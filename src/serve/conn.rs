use crate::{
    error::{AppErr, ErrorExt, IoErr},
    utils::get_mut,
};
use dashmap::DashMap;
use serde::{de::DeserializeOwned, Serialize};
use std::{sync::{Arc, atomic::{AtomicU8, Ordering}}, time::Duration};
use tokio::{
    net::TcpStream,
    sync::{mpsc, oneshot, Semaphore},
    time,
};

use super::{
    handler::handle_frame,
    manager::conn_remove, frame::{send::{SendFrame, ResponseFrame, RequestFrame}, recv::{RecvFrame}, write, read, BaseFrame, frame_type, make_type_seq}, api::{ConnInfo},
};


pub struct DeviceConn {
    stream: TcpStream,
    pub info: ConnInfo,

    seq: AtomicU8,

    exit_sem: Semaphore,
    write_tx: mpsc::Sender<SendFrame>,
    
    // type << 8 + seq
    res_mq: DashMap<u16, oneshot::Sender<RecvFrame>>,
}

pub type SharedConn = Arc<DeviceConn>;

impl DeviceConn {
    pub fn new(stream: TcpStream, info: ConnInfo) -> SharedConn {
        let (tx, rx) = mpsc::channel(32);
        let conn = DeviceConn {
            info,
            stream,
            seq: AtomicU8::new(0),
            exit_sem: Semaphore::new(0),
            write_tx: tx,
            res_mq: DashMap::new(),
        };
        let conn = Arc::new(conn);
        tokio::spawn(read_loop(conn.clone()));
        tokio::spawn(write_loop(conn.clone(), rx));
        conn
    }

    pub fn exit(&self) {
        self.exit_sem.add_permits(2);
    }

    pub async fn exec_simple_req<T: Serialize, R: DeserializeOwned>(
        &self,
        cmd: u8,
        value: &T
    ) -> Result<R, AppErr> {

        let seq = self.get_seq();
        let rx = self.create_resp(make_type_seq(frame_type::SIMPLE_RES, seq));
        self.write(SendFrame::SimpleReq(RequestFrame::new(seq, cmd, value)))?;
        let frame = time::timeout(Duration::from_secs(1), rx).await.wrap()?.wrap()?;
        let frame = frame.simple_res()?;
        let v = frame.parse()?;
        Ok(v)
    }

    pub async fn exec_req<T: Serialize, R: DeserializeOwned>(
        &self,
        cmd: u8,
        value: &T,
        timeout: Duration
    ) -> Result<R, AppErr> {
        let seq = self.get_seq();
        let ack_rx = self.create_resp(make_type_seq(frame_type::ACK, seq));
        let res_rx = self.create_resp(make_type_seq(frame_type::RES, seq));
        self.write( SendFrame::Req(RequestFrame::new(seq, cmd, value)) )?;

        let ack = time::timeout(Duration::from_secs(1), ack_rx).await.wrap()?.wrap()?;
        ack.ack()?;

        let frame = time::timeout(timeout, res_rx).await.wrap()?.wrap()?;
        let frame = frame.res()?;
        let r = frame.parse()?;
        Ok(r)
    }

    pub async fn exec_ping(&self) -> Result<(), AppErr> {
        let seq = self.get_seq();
        let rx = self.create_resp(make_type_seq(frame_type::PONG, seq));
        self.write(SendFrame::Ping(BaseFrame{ seq }))?;
        let frame = time::timeout(Duration::from_secs(1), rx).await.wrap()?.wrap()?;
        frame.pong()?;
        Ok(())
    }

    pub fn write(&self, frame: SendFrame) -> Result<(), AppErr> {
        self.write_tx.try_send(frame).wrap()?;
        Ok(())
    }

    pub fn ack(&self, seq: u8) -> Result<(), AppErr> {
        let frame = SendFrame::Ack(BaseFrame { seq });
        self.write(frame)
    }

    pub fn res<T: Serialize>(&self, seq: u8, cmd: u8, value: Result<T, AppErr>) -> Result<(), AppErr> {
        let frame = ResponseFrame::new(seq, cmd, value);
        self.write(SendFrame::Res(frame))
    }

    pub fn simple_res<T: Serialize>(&self, seq: u8, cmd: u8, value: Result<T, AppErr>) -> Result<(), AppErr> {
        let frame = ResponseFrame::new(seq, cmd, value);
        self.write(SendFrame::SimpleRes(frame))
    }

    fn get_seq(&self) -> u8 {
        self.seq.fetch_add(1, Ordering::SeqCst)
    }

    fn create_resp(&self, type_seq: u16) -> oneshot::Receiver<RecvFrame> {
        let (tx, rx) = oneshot::channel();
        self.res_mq.insert(type_seq, tx);
        rx
    }

    fn recv_resp(&self, type_seq: u16, frame: RecvFrame) {
        let tx = self.res_mq.remove(&type_seq);
        if let Some(tx) = tx {
            _ = tx.1.send(frame);
        }
    }

    async fn read_frame(&self) -> Result<RecvFrame, AppErr> {
        let stream = get_mut(&self.stream);
        let frame = read(stream).await?;
        Ok(frame)
    }

    async fn write_frame(&self, frame: &SendFrame) -> Result<(), IoErr> {
        let stream = get_mut(&self.stream);
        write(stream, frame).await
    }
}

async fn read_loop(conn: SharedConn) {
    loop {
        let ret = tokio::select! {
            ret = conn.read_frame() => ret,

            _ = conn.exit_sem.acquire() => {
                println!("read exit");
                break;
            }
        };

        let frame = match ret {
            Ok(ret) => ret,
            Err(e) => {
                println!("read err:{0}", e);
                break;
            }
        };

        match &frame {
            RecvFrame::Ack(f) => {
                conn.recv_resp(make_type_seq(frame_type::ACK, f.seq), frame);
            },
            RecvFrame::Pong(f) => {
                conn.recv_resp(make_type_seq(frame_type::PONG, f.seq), frame);
            },
            RecvFrame::Res(f) => {
                conn.recv_resp(make_type_seq(frame_type::RES, f.seq), frame);
            },
            RecvFrame::SimpleRes(f) => {
                conn.recv_resp(make_type_seq(frame_type::SIMPLE_RES, f.seq), frame);
            },
            _ => handle_frame(&conn, frame).await,
        };
        
    }
    conn.exit();
}

async fn write_loop(conn: SharedConn, mut rx: mpsc::Receiver<SendFrame>) {
    loop {
        let frame = tokio::select! {
            frame = rx.recv() => {
                frame
            }
            _ = conn.exit_sem.acquire() => {
                println!("write exit");
                break;
            }
        };
        let frame = match frame {
            None => {
                println!("write exit2");
                break;
            }
            Some(v) => v,
        };
        let ret = conn.write_frame(&frame).await;
        if let Err(e) = ret {
            println!("write err:{}", e);
            break;
        }
    }
    conn.exit();
    conn_remove(&conn);
}

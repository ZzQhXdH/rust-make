use crate::{
    error::{errors, proto_err, AppErr, ErrInfo, ErrorExt, IoErr},
    serve::proto::make_req,
    utils::{get_mut, rand_u8, Array},
};
use serde::{de::DeserializeOwned, Serialize};
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::{
    io::AsyncWriteExt,
    net::TcpStream,
    sync::{mpsc, oneshot, Mutex, Semaphore},
    time::timeout,
};

use super::{
    handler::handle_frame,
    io,
    manager::conn_remove,
    proto::{make_ping, ReadFrame},
};

pub struct DeviceConn {
    stream: TcpStream,
    pub addr: SocketAddr,

    exit_sem: Semaphore,
    write_tx: mpsc::Sender<Array<u8>>,

    master_lock: Mutex<()>,
    pub pong_tx: Mutex<Option<oneshot::Sender<()>>>,
    pub res_tx: Mutex<Option<oneshot::Sender<Array<u8>>>>,
}

pub type SharedConn = Arc<DeviceConn>;

impl DeviceConn {
    pub fn new(stream: TcpStream, addr: SocketAddr) -> SharedConn {
        let (tx, rx) = mpsc::channel(32);
        let conn = DeviceConn {
            addr,
            stream,
            exit_sem: Semaphore::new(0),
            write_tx: tx,
            pong_tx: Mutex::new(None),
            res_tx: Mutex::new(None),
            master_lock: Mutex::new(()),
        };
        let conn = Arc::new(conn);
        tokio::spawn(read_loop(conn.clone()));
        tokio::spawn(write_loop(conn.clone(), rx));
        conn
    }

    pub async fn ping(&self) -> Result<(), AppErr> {
        let _ = self.master_lock.lock().await;

        let (tx, rx) = oneshot::channel::<()>();
        {
            let mut pong_tx = self.pong_tx.lock().await;
            *pong_tx = Some(tx);
        }
        self.write(make_ping())?;
        timeout(Duration::from_secs(1), rx).await.wrap()?.wrap()?;
        Ok(())
    }

    pub async fn exec_req<T: Serialize, R: DeserializeOwned>(
        &self,
        cmd: u8,
        value: &T,
    ) -> Result<R, AppErr> {
        let seq = rand_u8();
        let buf = self.req(make_req(seq, cmd, value)).await?;
        let len = buf.len();
        if len < 3 {
            return proto_err("res len invalid");
        }
        let r_seq = buf[0];
        let r_cmd = buf[1];
        let ec = buf[2];
        if (r_seq != seq) || (r_cmd != cmd) {
            return proto_err("res seq or cmd invalid");
        }
        if ec != 0 {
            let err_info: ErrInfo = serde_cbor::from_slice(&buf[3..])?;
            return Err(AppErr::Custom(err_info));
        }
        let resp = serde_cbor::from_slice(&buf[3..])?;
        Ok(resp)
    }

    async fn req(&self, frame: Array<u8>) -> Result<Array<u8>, AppErr> {
        let _ = self.master_lock.lock().await;

        let (tx, rx) = oneshot::channel::<Array<u8>>();
        {
            let mut res_tx = self.res_tx.lock().await;
            *res_tx = Some(tx);
        }
        self.write(frame)?;
        let body = timeout(Duration::from_secs(1), rx).await.wrap()?.wrap()?;
        Ok(body)
    }

    pub fn write(&self, buf: Array<u8>) -> Result<(), AppErr> {
        self.write_tx.try_send(buf).wrap()?;
        Ok(())
    }

    pub fn exit(&self) {
        self.exit_sem.add_permits(2);
    }

    async fn read_frame(&self) -> Result<ReadFrame, AppErr> {
        let stream = get_mut(&self.stream);
        let frame = timeout(Duration::from_secs(10), io::read_frame(stream))
            .await
            .wrap()?
            .wrap()?;
        Ok(frame)
    }

    async fn write_frame(&self, buf: &[u8]) -> Result<(), IoErr> {
        let stream = get_mut(&self.stream);
        stream.write_all(buf).await
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

        handle_frame(&conn, frame).await;
    }
    conn.exit();
    conn_remove(&conn);
}

async fn write_loop(conn: SharedConn, mut rx: mpsc::Receiver<Array<u8>>) {
    loop {
        let buf = tokio::select! {
            buf = rx.recv() => {
                buf
            }
            _ = conn.exit_sem.acquire() => {
                println!("write exit");
                break;
            }
        };
        let buf = match buf {
            None => {
                println!("write exit2");
                break;
            }
            Some(v) => v,
        };
        let ret = conn.write_frame(&buf).await;
        if let Err(e) = ret {
            println!("write err:{}", e);
            break;
        }
    }
    conn.exit();
    conn_remove(&conn);
}

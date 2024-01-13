use self::{conn::DeviceConn, manager::conn_append};
use crate::config::DEVICE_ADDR;
use tokio::net::TcpListener;

mod api;
mod conn;
mod handler;
mod io;
mod manager;
mod other;
mod proto;
mod sync;

pub async fn run() {
    manager::init();
    println!("device serve:{}", DEVICE_ADDR);

    let serve = TcpListener::bind(DEVICE_ADDR).await.unwrap();
    tokio::spawn(inner_run(serve));
}

async fn inner_run(serve: TcpListener) {
    loop {
        let ret = serve.accept().await;
        match ret {
            Ok((stream, addr)) => {
                let conn = DeviceConn::new(stream, addr);
                conn_append(conn);
            }
            Err(e) => {
                println!("accept err:{}", e);
            }
        };
    }
}

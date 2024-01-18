use std::net::SocketAddr;

use self::{conn::DeviceConn, manager::conn_append, api::wait_login};
use crate::{config::DEVICE_ADDR, error::AppErr};
use tokio::net::{TcpListener, TcpStream};

mod api;
mod conn;
mod handler;
mod manager;
mod other;
mod frame;

pub use frame::Body;


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
                tokio::spawn(try_wait_login(stream, addr));
            }
            Err(e) => {
                println!("accept err:{}", e);
            }
        };
    }
}

async fn try_wait_login(stream: TcpStream, addr: SocketAddr) {

    if let Err(e) = do_login(stream, addr).await {
        println!("login:{}", e);
    };
}

async fn do_login(mut stream: TcpStream, addr: SocketAddr) -> Result<(), AppErr> {
    let info = wait_login(&mut stream, addr).await?;
    let conn = DeviceConn::new(stream, info);
    conn_append(conn);
    Ok(())
}



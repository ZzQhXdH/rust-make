use super::{
    api::handle_resp,
    conn::SharedConn,
    proto::{make_pong, ReadFrame},
};
use crate::error::ErrorExt;

pub async fn handle_frame(conn: &SharedConn, frame: ReadFrame) {
    match frame {
        ReadFrame::Ping => {
            conn.write(make_pong()).print_if_err();
        }

        ReadFrame::Pong => {
            let mut tx = conn.pong_tx.lock().await;
            let tx = tx.take();
            if let Some(tx) = tx {
                _ = tx.send(());
            }
        }

        ReadFrame::Resp(body) => {
            let mut tx = conn.res_tx.lock().await;
            let tx = tx.take();
            if let Some(tx) = tx {
                _ = tx.send(body);
            }
        }

        ReadFrame::Req(body) => {
            tokio::spawn(handle_resp(conn.clone(), body));
        }

        _ => {}
    };
}

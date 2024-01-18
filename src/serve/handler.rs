use super::{
    conn::SharedConn,
    frame::{recv::RecvFrame, send::SendFrame, BaseFrame}, api::handle_req,
};


pub async fn handle_frame(conn: &SharedConn, frame: RecvFrame) {
    
    match frame {
        RecvFrame::Ping(r) => {
            _ = conn.write(SendFrame::Pong(BaseFrame { seq: r.seq }));
            conn.info.ping();
        },

        RecvFrame::Req(r) => {
            _ = conn.write(SendFrame::Ack(BaseFrame { seq: r.seq }));
            tokio::spawn(handle_req(conn.clone(), r));
        },

        RecvFrame::NotifyAck(r) => {
            _ = conn.write(SendFrame::Ack(BaseFrame { seq: r.seq }));
        },

        _ => {}
    };
}




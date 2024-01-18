use crate::serve::frame::ToFrameBody;

use super::{conn::SharedConn, api::ConnInfo};
use dashmap::DashSet;
use std::mem::MaybeUninit;

pub struct Manager {
    hub: DashSet<SharedConn>,
}

static mut MANAGER: MaybeUninit<Manager> = MaybeUninit::uninit();

pub fn init() {
    let m = Manager {
        hub: DashSet::new(),
    };
    unsafe {
        MANAGER.write(m);
    }
}

fn get_manager() -> &'static Manager {
    unsafe { MANAGER.assume_init_ref() }
}

pub fn conn_append(conn: SharedConn) {
    let m = get_manager();
    m.hub.insert(conn);
}

pub fn conn_remove(conn: &SharedConn) {
    let m = get_manager();
    m.hub.remove(conn);
}

pub fn conn_infos() -> Vec<u8> {
    let manager = unsafe {
        MANAGER.assume_init_ref()  
    };
    let mut vec: Vec<SharedConn> = Vec::with_capacity(manager.hub.len());
    for conn in manager.hub.iter() {
        vec.push(conn.clone());
    }
    let is: Vec<&ConnInfo> = vec.iter().map(|c| &c.info).collect();
    is.to_vec()
}

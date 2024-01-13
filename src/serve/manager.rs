use super::conn::SharedConn;
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

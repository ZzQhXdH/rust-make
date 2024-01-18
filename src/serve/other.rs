use std::hash::Hash;

use super::conn::DeviceConn;

impl PartialEq for DeviceConn {
    fn eq(&self, other: &Self) -> bool {
        self.info.addr == other.info.addr
    }
}

impl Eq for DeviceConn {}

impl Hash for DeviceConn {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.info.addr.hash(state);
    }
}

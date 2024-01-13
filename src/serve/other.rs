use std::hash::Hash;

use super::conn::DeviceConn;

impl PartialEq for DeviceConn {
    fn eq(&self, other: &Self) -> bool {
        self.addr == other.addr
    }
}

impl Eq for DeviceConn {}

impl Hash for DeviceConn {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.addr.hash(state);
    }
}

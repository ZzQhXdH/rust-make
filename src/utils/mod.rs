use std::time::SystemTime;

pub type Array<T> = Box<[T]>;

pub fn current_timestamp() -> i64 {
    let now = SystemTime::now();
    now.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() as i64
}



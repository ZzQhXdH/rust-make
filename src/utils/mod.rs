use std::{ptr::NonNull, time::SystemTime};

use rand::Rng;

pub mod codec;

pub type Array<T> = Box<[T]>;

pub fn new_bytes(len: usize) -> Array<u8> {
    let mut buf = Vec::with_capacity(len);
    unsafe {
        buf.set_len(len);
    }
    buf.into_boxed_slice()
}

pub fn current_timestamp() -> i64 {
    let now = SystemTime::now();
    now.duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

pub fn get_mut<T>(value: &T) -> &mut T {
    unsafe {
        NonNull::new_unchecked(value as *const T as *mut T).as_mut()
    }
}

pub fn rand_u8() -> u8 {
    let mut rng = rand::thread_rng();
    rng.gen_range(0..=255)
}

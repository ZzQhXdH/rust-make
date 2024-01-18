
pub fn encode_u8(buf: &mut[u8], value: u8) {
    buf[0] = value;
}

pub fn encode_u16(buf: &mut[u8], value: u16) {
    buf[0] = (value >> 8) as u8;
    buf[1] = value as u8;
}

pub fn encode_u24(buf: &mut[u8], value: u32) {
    buf[0] = (value >> 16) as u8;
    buf[1] = (value >> 8) as u8;
    buf[2] = value as u8;
}

pub fn decode_u8(buf: &[u8]) -> u8 {
    buf[0]
}

pub fn decode_u16(buf: &[u8]) -> u16 {
    ((buf[0] as u16) >> 8) + (buf[1] as u16)
}

pub fn decode_u24(buf: &[u8]) -> u32 {
    ((buf[0] as u32) >> 16) + ((buf[1] as u32) >> 8) + (buf[2] as u32)
}

pub fn memcpy(dst: &mut [u8], src: &[u8]) {
    unsafe {
        std::ptr::copy_nonoverlapping(src.as_ptr(), dst.as_mut_ptr(), src.len());
    }
}


macro_rules! read_2_bytes {
    ($c: expr) => {
        $c.read_u16::<byteorder::BigEndian>().unwrap()
    };
}
macro_rules! read_4_bytes {
    ($c: expr) => {
        $c.read_u32::<byteorder::BigEndian>().unwrap()
    };
}
pub mod utils {}
pub(crate) use read_2_bytes;
pub(crate) use read_4_bytes;

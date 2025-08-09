#[derive(Clone, Copy)]
pub enum Endianness {
    LittleEndian,
    BigEndian,
}

pub trait FromBytes: Sized {
    fn from_le_bytes(bytes: &[u8]) -> Self;
    fn from_be_bytes(bytes: &[u8]) -> Self;
}

impl FromBytes for u16 {
    fn from_le_bytes(bytes: &[u8]) -> Self {
        Self::from_le_bytes(bytes.try_into().unwrap())
    }
    fn from_be_bytes(bytes: &[u8]) -> Self {
        Self::from_be_bytes(bytes.try_into().unwrap())
    }
}

impl FromBytes for u32 {
    fn from_le_bytes(bytes: &[u8]) -> Self {
        Self::from_le_bytes(bytes.try_into().unwrap())
    }
    fn from_be_bytes(bytes: &[u8]) -> Self {
        Self::from_be_bytes(bytes.try_into().unwrap())
    }
}

impl FromBytes for u64 {
    fn from_le_bytes(bytes: &[u8]) -> Self {
        Self::from_le_bytes(bytes.try_into().unwrap())
    }
    fn from_be_bytes(bytes: &[u8]) -> Self {
        Self::from_be_bytes(bytes.try_into().unwrap())
    }
}

pub fn get_value<T: FromBytes>(endianness: Endianness, bytes: &[u8]) -> T {
    assert_eq!(bytes.len(), size_of::<T>());

    if matches!(endianness, Endianness::LittleEndian) {
        return T::from_le_bytes(bytes);
    }

    return T::from_be_bytes(bytes);
}

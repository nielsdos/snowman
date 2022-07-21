use crate::emulator_accessor::EmulatorAccessor;

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        if cfg!(debug_assertions) {
            println!($($arg)*);
        }
    };
}

pub fn expect_magic<E>(value: u16, expected: u16, error: E) -> Result<(), E> {
    bool_to_result(value == expected, error)
}

pub fn bool_to_result<T>(result: bool, error: T) -> Result<(), T> {
    if result {
        Ok(())
    } else {
        Err(error)
    }
}

pub fn u16_from_slice(bytes: &[u8], offset: usize) -> Option<u16> {
    Some(u16::from_le_bytes(
        bytes
            .get(offset..offset + 2)?
            .try_into()
            .expect("slice is big enough"),
    ))
}

pub fn u16_from_array<const N: usize>(bytes: &[u8; N], offset: usize) -> Option<u16> {
    Some(u16::from_le_bytes(
        bytes
            .get(offset..offset + 2)?
            .try_into()
            .expect("slice is big enough"),
    ))
}

pub fn u32_from_array<const N: usize>(bytes: &[u8; N], offset: usize) -> Option<u32> {
    Some(u32::from_le_bytes(
        bytes
            .get(offset..offset + 4)?
            .try_into()
            .expect("slice is big enough"),
    ))
}

#[inline]
pub fn add_with_flags_16(a: u16, b: u16) -> (u16, bool, bool) {
    let (result, carry) = a.overflowing_add(b);
    let overflow = (a as i16).overflowing_add(b as i16).1;
    (result, carry, overflow)
}

#[inline]
pub fn sub_with_flags_16(a: u16, b: u16) -> (u16, bool, bool) {
    let (result, carry) = a.overflowing_sub(b);
    let overflow = (a as i16).overflowing_sub(b as i16).1;
    (result, carry, overflow)
}

#[inline]
pub fn add_with_flags_8(a: u8, b: u8) -> (u8, bool, bool) {
    let (result, carry) = a.overflowing_add(b);
    let overflow = (a as i8).overflowing_add(b as i8).1;
    (result, carry, overflow)
}

#[inline]
pub fn sub_with_flags_8(a: u8, b: u8) -> (u8, bool, bool) {
    let (result, carry) = a.overflowing_sub(b);
    let overflow = (a as i8).overflowing_sub(b as i8).1;
    (result, carry, overflow)
}

fn map_u16_u8(tuple: (u8, bool, bool)) -> (u16, bool, bool) {
    (tuple.0 as u16, tuple.1, tuple.2)
}

#[inline]
pub fn sub_with_flags<const N: usize>(a: u16, b: u16) -> (u16, bool, bool) {
    if N == 16 {
        sub_with_flags_16(a, b)
    } else if N == 8 {
        map_u16_u8(sub_with_flags_8(a as u8, b as u8))
    } else {
        unreachable!()
    }
}

#[inline]
pub fn add_with_flags<const N: usize>(a: u16, b: u16) -> (u16, bool, bool) {
    if N == 16 {
        add_with_flags_16(a, b)
    } else if N == 8 {
        map_u16_u8(add_with_flags_8(a as u8, b as u8))
    } else {
        unreachable!()
    }
}

pub fn debug_print_null_terminated_string(accessor: &EmulatorAccessor, mut address: u32) {
    print!("  > {:x} -> \"", address);
    let mut length = 0;
    loop {
        let data = accessor.memory().read_8(address).unwrap_or(0);
        if data == 0 {
            break;
        }
        print!("{}", data as char);
        address += 1;
        length += 1;
    }
    println!("\" [length {}]", length);
}

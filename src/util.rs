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

pub fn debug_print_null_terminated_string(accessor: &EmulatorAccessor, mut address: u32) {
    print!("  > ");
    loop {
        let data = accessor.memory().read_8(address).unwrap_or(0);
        if data == 0 {
            break;
        }
        print!("{}", data as char);
        address += 1;
    }
    println!();
}

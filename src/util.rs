use crate::emulator_accessor::EmulatorAccessor;

pub fn bool_to_result<T>(result: bool, error: T) -> Result<(), T> {
    if result {
        Ok(())
    } else {
        Err(error)
    }
}

pub fn u16_from_slice(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes(
        bytes[offset..offset + 2]
            .try_into()
            .expect("slice is big enough"),
    )
}

pub fn u32_from_slice(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(
        bytes[offset..offset + 4]
            .try_into()
            .expect("slice is big enough"),
    )
}

pub fn debug_print_null_terminated_string(accessor: &EmulatorAccessor, mut address: u32) {
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

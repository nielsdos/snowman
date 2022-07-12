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

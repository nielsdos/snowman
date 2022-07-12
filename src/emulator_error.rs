// TODO: proper errors
#[derive(Debug)]
pub enum EmulatorError {
    OutOfBounds,
    Exit,
    InvalidOpcode,
}

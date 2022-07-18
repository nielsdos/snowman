#[derive(Debug)]
pub enum EmulatorError {
    OutOfBounds,
    Exit,
    InvalidOpcode,
    DivideError,
}

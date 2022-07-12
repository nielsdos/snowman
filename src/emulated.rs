pub trait EmulatedComponentInformationProvider {
    fn argument_bytes_of_procedure(&self, procedure: u16) -> u16;
}

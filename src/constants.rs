use bitflags::bitflags;

pub const KERNEL_INT_VECTOR: u8 = 0xff;
pub const USER_INT_VECTOR: u8 = 0xfe;
pub const GDI_INT_VECTOR: u8 = 0xfd;
pub const KEYBOARD_INT_VECTOR: u8 = 0xfc;
pub const LOWEST_SYSCALL_INT_VECTOR: u8 = 0xfc;

bitflags! {
    #[allow(dead_code)]
    pub struct WinFlags: u16 {
        const WF_80X87 = 0x400;
        const WF_CPU286 = 0x2;
        const WF_CPU386 = 0x4;
        const WF_CPU486 = 0x8;
        const WF_ENHANCED = 0x20;
        const WF_PAGING = 0x800;
        const WF_PMODE = 0x1;
        const WF_STANDARD = 0x10;
    }
}

#[allow(dead_code)]
#[derive(Eq, PartialEq)]
pub enum MessageType {
    CREATE = 0x01,
    PAINT = 0x0f,
    QUIT = 0x12,
}

impl From<MessageType> for u16 {
    fn from(m: MessageType) -> Self {
        m as u16
    }
}

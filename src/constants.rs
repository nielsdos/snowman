use bitflags::bitflags;

pub const KERNEL_INT_VECTOR: u8 = 0xff;
pub const USER_INT_VECTOR: u8 = 0xfe;
pub const GDI_INT_VECTOR: u8 = 0xfd;
pub const KEYBOARD_INT_VECTOR: u8 = 0xfc;
pub const LOWEST_SYSCALL_INT_VECTOR: u8 = 0xfc;

bitflags! {
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

// Window messages
pub const WM_CREATE: u16 = 0x01;
pub const WM_PAINT: u16 = 0x0f;
pub const WM_QUIT: u16 = 0x12;

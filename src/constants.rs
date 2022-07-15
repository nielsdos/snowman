pub const KERNEL_INT_VECTOR: u8 = 0xff;
pub const USER_INT_VECTOR: u8 = 0xfe;
pub const GDI_INT_VECTOR: u8 = 0xfd;
pub const KEYBOARD_INT_VECTOR: u8 = 0xfc;
pub const LOWEST_SYSCALL_INT_VECTOR: u8 = 0xfc;

// GETWINFLAGS
pub const WF_80X87: u16 = 0x400;
pub const WF_CPU286: u16 = 0x2;
pub const WF_CPU386: u16 = 0x4;
pub const WF_CPU486: u16 = 0x8;
pub const WF_ENHANCED: u16 = 0x20;
pub const WF_PAGING: u16 = 0x800;
pub const WF_PMODE: u16 = 0x1;
pub const WF_STANDARD: u16 = 0x10;

// Window messages
pub const WM_CREATE: u16 = 0x01;
pub const WM_PAINT: u16 = 0x0f;
pub const WM_QUIT: u16 = 0x12;

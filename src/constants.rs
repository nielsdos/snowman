use bitflags::bitflags;
use num_derive::FromPrimitive;

pub const KERNEL_INT_VECTOR: u8 = 0xff;
pub const USER_INT_VECTOR: u8 = 0xfe;
pub const GDI_INT_VECTOR: u8 = 0xfd;
pub const KEYBOARD_INT_VECTOR: u8 = 0xfc;
pub const LOWEST_SYSCALL_INT_VECTOR: u8 = 0xfc;

bitflags! {
    #[allow(dead_code)]
    pub struct WinFlags: u32 {
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

bitflags! {
    #[allow(dead_code)]
    pub struct ClassStyles: u16 {
        const VREDRAW = 0x001;
        const HREDRAW = 0x002;
        const PARENT_DC = 0x080;
    }
}

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum MessageType {
    Create = 0x01,
    Size = 0x05,
    Paint = 0x0f,
    Quit = 0x12,
    EraseBkGnd = 0x14,
    ShowWindow = 0x18,
    GetMinMaxInfo = 0x24,
    WindowPosChanging = 0x46,
    NcCreate = 0x81,
    NcCalcSize = 0x83,
    Timer = 0x113,
}

impl From<MessageType> for u16 {
    fn from(m: MessageType) -> Self {
        m as u16
    }
}

#[allow(dead_code)]
#[derive(Eq, PartialEq, FromPrimitive)]
pub enum SystemColors {
    Scrollbar,
    Background,
    ActiveCaption,
    InactiveCaption,
    Menu,
    Window,
    WindowFrame,
    MenuText,
    WindowText,
    CaptionText,
    ActiveBorder,
    InactiveBorder,
    AppWorkspace,
    Highlight,
    HighlightText,
    ButtonFace,
    ButtonShadow,
    GrayText,
    ButtonText,
    InactiveCaptionText,
    ButtonHighlight,
}

#[allow(dead_code)]
#[derive(Eq, PartialEq)]
pub enum DeviceCapRequest {
    HorzSize = 4,
    VertSize = 6,
    HorzRes = 8,
    VertRes = 10,
}

impl From<DeviceCapRequest> for u16 {
    fn from(r: DeviceCapRequest) -> Self {
        r as u16
    }
}

#[allow(dead_code)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, FromPrimitive)]
pub enum RasterOp {
    Black = 1,
    NotMergePen = 2,
    MaskNotPen = 3,
    NotCopyPen = 4,
    MaskPenNot = 5,
    Not = 6,
    XorPen = 7,
    NotMaskPen = 8,
    MaskPen = 9,
    NotXorPen = 10,
    Nop = 11,
    MergeNotPen = 12,
    CopyPen = 13,
    MergePenNot = 14,
    MergePen = 15,
    White = 16,
}

impl From<RasterOp> for u16 {
    fn from(r: RasterOp) -> Self {
        r as u16
    }
}

use num_traits::PrimInt;

pub struct Registers {
    pub ip: u16,
    gpr: [u16; 8],
    // TODO: Last 2 unused until I figure out what to do with illegal instructions
    segments: [u16; 8],
    flags: u16,
}

#[allow(dead_code)]
impl Registers {
    pub const FLAG_CF: u16 = 1 << 11;
    pub const FLAG_OF: u16 = 1 << 0;
    pub const FLAG_SF: u16 = 1 << 7;
    pub const FLAG_ZF: u16 = 1 << 6;
    pub const FLAG_PF: u16 = 1 << 2;
    pub const FLAG_AF: u16 = 1 << 4;

    pub const REG_AX: u8 = 0;
    pub const REG_CX: u8 = 1;
    pub const REG_DX: u8 = 2;
    pub const REG_BX: u8 = 3;
    pub const REG_SP: u8 = 4;
    pub const REG_BP: u8 = 5;
    pub const REG_SI: u8 = 6;
    pub const REG_DI: u8 = 7;
    pub const REG_AL: u8 = 0;
    pub const REG_CL: u8 = 1;
    pub const REG_DL: u8 = 2;
    pub const REG_BL: u8 = 3;
    pub const REG_AH: u8 = 0;
    pub const REG_CH: u8 = 1;
    pub const REG_DH: u8 = 2;
    pub const REG_BH: u8 = 3;

    pub const REG_ES: u8 = 0;
    pub const REG_CS: u8 = 1;
    pub const REG_SS: u8 = 2;
    pub const REG_DS: u8 = 3;
    pub const REG_FS: u8 = 4;
    pub const REG_GS: u8 = 5;

    pub fn new(cs: u16, ip: u16) -> Self {
        let mut gpr = [0; 8];
        gpr[Self::REG_SP as usize] = 1024; // TODO
        let mut segments = [0; 8];
        segments[Self::REG_CS as usize] = cs;
        Self {
            ip,
            gpr,
            segments,
            flags: 0x200,
        }
    }

    pub fn flat_ip(&self) -> usize {
        self.ip as usize + ((self.read_segment(Registers::REG_CS) as usize) << 4)
    }

    pub fn flat_sp(&self) -> u32 {
        self.read_gpr_16(Registers::REG_SP) as u32
            + ((self.read_segment(Registers::REG_SS) as u32) << 4)
    }

    #[inline]
    pub fn flags(&self) -> u16 {
        self.flags
    }

    pub fn dec_sp(&mut self, amount: u16) {
        self.gpr[Self::REG_SP as usize] = self.gpr[Self::REG_SP as usize].wrapping_sub(amount);
    }

    pub fn inc_sp(&mut self, amount: u16) {
        self.gpr[Self::REG_SP as usize] = self.gpr[Self::REG_SP as usize].wrapping_add(amount);
    }

    #[inline]
    pub fn read_gpr_16(&self, index: u8) -> u16 {
        self.gpr[(index & 7) as usize]
    }

    #[inline]
    pub fn write_gpr_16(&mut self, index: u8, data: u16) {
        self.gpr[(index & 7) as usize] = data;
    }

    #[inline]
    pub fn read_segment(&self, index: u8) -> u16 {
        self.segments[(index & 7) as usize]
    }

    #[inline]
    pub fn write_segment(&mut self, index: u8, data: u16) {
        self.segments[(index & 7) as usize] = data;
    }

    #[inline]
    pub fn read_gpr_lo_8(&self, index: u8) -> u8 {
        debug_assert!(index < 4);
        self.gpr[(index & 7) as usize] as u8
    }

    #[inline]
    pub fn read_gpr_hi_8(&self, index: u8) -> u8 {
        debug_assert!(index < 4);
        (self.gpr[(index & 7) as usize] >> 8) as u8
    }

    pub fn read_gpr_8(&self, register_destination: u8) -> u8 {
        if register_destination <= 3 {
            self.read_gpr_lo_8(register_destination)
        } else {
            self.read_gpr_hi_8(register_destination - 4)
        }
    }

    pub fn read_gpr<const N: usize>(&self, index: u8) -> u16 {
        if N == 8 {
            self.read_gpr_8(index) as u16
        } else if N == 16 {
            self.read_gpr_16(index)
        } else {
            unreachable!()
        }
    }

    pub fn write_gpr<const N: usize>(&mut self, index: u8, data: u16) {
        if N == 8 {
            self.write_gpr_8(index, data as u8);
        } else if N == 16 {
            self.write_gpr_16(index, data);
        } else {
            unreachable!();
        }
    }

    #[inline]
    pub fn write_gpr_lo_8(&mut self, index: u8, data: u8) {
        debug_assert!(index < 4);
        let index = (index & 7) as usize;
        self.gpr[index] = (self.gpr[index] & 0xff00) | (data as u16);
    }

    #[inline]
    pub fn write_gpr_hi_8(&mut self, index: u8, data: u8) {
        debug_assert!(index < 4);
        let index = (index & 7) as usize;
        self.gpr[index] = (self.gpr[index] & 0x00ff) | ((data as u16) << 8);
    }

    pub fn write_gpr_8(&mut self, register_destination: u8, data: u8) {
        if register_destination <= 3 {
            self.write_gpr_lo_8(register_destination, data);
        } else {
            self.write_gpr_hi_8(register_destination - 4, data);
        }
    }

    pub fn handle_bitwise_result(&mut self, result: u16) {
        // Clear OF & CF, and the flags we can set here
        self.flags &=
            !(Self::FLAG_CF | Self::FLAG_OF | Self::FLAG_SF | Self::FLAG_ZF | Self::FLAG_PF);
        // Set SF, ZF, PF flags according to result
        if result == 0 {
            self.flags |= Self::FLAG_ZF | Self::FLAG_PF;
        } else if result & (1 << 15) > 0 {
            self.flags |= Self::FLAG_SF;
            if (result.count_ones() & 1) == 0 {
                self.flags |= Self::FLAG_PF;
            }
        }
    }

    pub fn handle_arithmetic_result_u_generic<const N: usize, R: PrimInt>(&mut self, result: u16) {
        // TODO: support CF, OF, AF

        // Clear the flags we can set here
        self.flags &= !(Self::FLAG_CF
            | Self::FLAG_OF
            | Self::FLAG_SF
            | Self::FLAG_ZF
            | Self::FLAG_PF
            | Self::FLAG_AF);
        // Set flags according to result
        if result == 0 {
            self.flags |= Self::FLAG_ZF | Self::FLAG_PF;
        } else {
            let highest_bit_flag = if N == 8 { 1 << 7 } else { 1 << 15 };

            if result & highest_bit_flag > 0 {
                self.flags |= Self::FLAG_SF;
                if (result.count_ones() & 1) == 0 {
                    self.flags |= Self::FLAG_PF;
                }
            }
        }
    }

    pub fn handle_arithmetic_result_u16(&mut self, result: u16) {
        self.handle_arithmetic_result_u_generic::<16, u16>(result)
    }

    pub fn handle_arithmetic_result_u8(&mut self, result: u8) {
        self.handle_arithmetic_result_u_generic::<8, u8>(result as u16)
    }

    pub fn flag_zero(&self) -> bool {
        (self.flags & Self::FLAG_ZF) > 0
    }
}

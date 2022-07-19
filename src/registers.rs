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
    pub const FLAG_DF: u16 = 1 << 10;

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

    pub fn new(ds: u16, cs: u16, ip: u16) -> Self {
        let mut gpr = [0; 8];
        gpr[Self::REG_SP as usize] = 0xF000; // TODO
        let mut segments = [0; 8];
        segments[Self::REG_CS as usize] = cs;
        segments[Self::REG_DS as usize] = ds;
        segments[Self::REG_SS as usize] = ds;
        Self {
            ip,
            gpr,
            segments,
            flags: 0x200,
        }
    }

    pub fn flat_ip(&self) -> u32 {
        self.ip as u32 + ((self.read_segment(Registers::REG_CS) as u32) << 4)
    }

    pub fn flat_sp(&self) -> u32 {
        self.flat_reg(Registers::REG_SS, Registers::REG_SP)
    }

    pub fn flat_reg(&self, segment: u8, reg: u8) -> u32 {
        self.flat_address(segment, self.read_gpr_16(reg))
    }

    pub fn flat_address(&self, segment: u8, offset: u16) -> u32 {
        offset as u32 + ((self.read_segment(segment) as u32) << 4)
    }

    #[inline]
    pub fn flags(&self) -> u16 {
        self.flags
    }

    #[inline]
    pub fn set_direction_flag(&mut self, flag: bool) {
        if flag {
            self.flags |= Self::FLAG_DF;
        } else {
            self.flags &= !Self::FLAG_DF;
        }
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

    /// Set SF, ZF, PF flags according to result
    fn set_zf_pf_sf<const N: usize>(&mut self, result: u16) {
        let highest_bit_flag = if N == 8 { 1 << 7 } else { 1 << 15 };
        if result == 0 {
            self.flags |= Self::FLAG_ZF | Self::FLAG_PF;
        } else if result & highest_bit_flag > 0 {
            self.flags |= Self::FLAG_SF;
            if (result.count_ones() & 1) == 0 {
                self.flags |= Self::FLAG_PF;
            }
        }
    }

    pub fn handle_bitwise_result_u_generic<const N: usize>(
        &mut self,
        result_did_carry: bool,
        result: u16,
    ) {
        // Clear OF & CF, and the flags we can set here
        self.flags &=
            !(Self::FLAG_CF | Self::FLAG_OF | Self::FLAG_SF | Self::FLAG_ZF | Self::FLAG_PF);
        self.set_zf_pf_sf::<N>(result);
        if result_did_carry {
            self.flags |= Self::FLAG_CF;
        }
    }

    pub fn handle_bitwise_result_u16(&mut self, result_did_carry: bool, result: u16) {
        self.handle_bitwise_result_u_generic::<16>(result_did_carry, result)
    }

    pub fn handle_bitwise_result_u8(&mut self, result_did_carry: bool, result: u16) {
        self.handle_bitwise_result_u_generic::<8>(result_did_carry, result)
    }

    pub fn handle_arithmetic_result_u_generic<const N: usize>(
        &mut self,
        result: u16,
        result_did_carry: bool,
        affect_cf: bool,
    ) {
        // TODO: support OF, AF

        // Clear the flags we can set here
        if affect_cf {
            self.flags &= !Self::FLAG_CF;
            if result_did_carry {
                self.flags |= Self::FLAG_CF;
            }
        }
        self.flags &=
            !(Self::FLAG_OF | Self::FLAG_SF | Self::FLAG_ZF | Self::FLAG_PF | Self::FLAG_AF);
        self.set_zf_pf_sf::<N>(result);
    }

    pub fn handle_imul_result_u16(&mut self, result_did_carry: bool) {
        if result_did_carry {
            self.flags |= Self::FLAG_OF | Self::FLAG_CF;
        } else {
            self.flags &= !(Self::FLAG_OF | Self::FLAG_CF);
        }
    }

    pub fn handle_arithmetic_result_u16(
        &mut self,
        result: u16,
        result_did_carry: bool,
        affect_cf: bool,
    ) {
        self.handle_arithmetic_result_u_generic::<16>(result, result_did_carry, affect_cf)
    }

    pub fn handle_arithmetic_result_u8(
        &mut self,
        result: u8,
        result_did_carry: bool,
        affect_cf: bool,
    ) {
        self.handle_arithmetic_result_u_generic::<8>(result as u16, result_did_carry, affect_cf)
    }

    pub fn flag_zero(&self) -> bool {
        (self.flags & Self::FLAG_ZF) > 0
    }

    pub fn flag_carry(&self) -> bool {
        (self.flags & Self::FLAG_CF) > 0
    }

    pub fn flag_overflow(&self) -> bool {
        (self.flags & Self::FLAG_OF) > 0
    }

    pub fn flag_sign(&self) -> bool {
        (self.flags & Self::FLAG_SF) > 0
    }
}

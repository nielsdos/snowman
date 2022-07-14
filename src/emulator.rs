use crate::constants::{
    GDI_INT_VECTOR, KERNEL_INT_VECTOR, KEYBOARD_INT_VECTOR, LOWEST_SYSCALL_INT_VECTOR,
    USER_INT_VECTOR,
};
use crate::emulated_gdi::EmulatedGdi;
use crate::emulated_kernel::EmulatedKernel;
use crate::emulated_keyboard::EmulatedKeyboard;
use crate::emulator_accessor::EmulatorAccessor;
use crate::emulator_error::EmulatorError;
use crate::memory::Memory;
use crate::mod_rm::ModRM;
use crate::registers::Registers;
use crate::{debug, EmulatedUser};

pub struct Emulator<'a> {
    regs: Registers,
    memory: Memory,
    emulated_kernel: EmulatedKernel,
    emulated_user: EmulatedUser<'a>,
    emulated_gdi: EmulatedGdi<'a>,
    emulated_keyboard: EmulatedKeyboard,
}

impl<'a> Emulator<'a> {
    pub fn new(
        memory: Memory,
        ds: u16,
        cs: u16,
        ip: u16,
        emulated_kernel: EmulatedKernel,
        emulated_user: EmulatedUser<'a>,
        emulated_gdi: EmulatedGdi<'a>,
        emulated_keyboard: EmulatedKeyboard,
    ) -> Self {
        Self {
            regs: Registers::new(ds, cs, ip),
            memory,
            emulated_kernel,
            emulated_user,
            emulated_gdi,
            emulated_keyboard,
        }
    }

    fn push_value_16(&mut self, data: u16) -> Result<(), EmulatorError> {
        self.regs.dec_sp(2);
        self.memory.write_16(
            self.regs.flat_reg(Registers::REG_SS, Registers::REG_SP),
            data,
        )?;
        Ok(())
    }

    fn pop_value_16(&mut self) -> Result<u16, EmulatorError> {
        let data = self
            .memory
            .read_16(self.regs.flat_reg(Registers::REG_SS, Registers::REG_SP))?;
        self.regs.inc_sp(2);
        Ok(data)
    }

    fn push_ip(&mut self) -> Result<(), EmulatorError> {
        self.push_value_16(self.regs.ip)
    }

    fn push_cs(&mut self) -> Result<(), EmulatorError> {
        self.push_value_16(self.regs.read_segment(Registers::REG_CS))
    }

    fn push_gpr_16(&mut self, register: u8) -> Result<(), EmulatorError> {
        self.push_value_16(self.regs.read_gpr_16(register))
    }

    fn pop_gpr_16(&mut self, register: u8) -> Result<(), EmulatorError> {
        let data = self.pop_value_16()?;
        self.regs.write_gpr_16(register, data);
        Ok(())
    }

    fn push_segment_16(&mut self, segment: u8) -> Result<(), EmulatorError> {
        self.push_value_16(self.regs.read_segment(segment))
    }

    fn pop_segment_16(&mut self, segment: u8) -> Result<(), EmulatorError> {
        let data = self.pop_value_16()?;
        self.regs.write_segment(segment, data);
        Ok(())
    }

    pub fn read_ip_u8(&mut self) -> Result<u8, EmulatorError> {
        let byte = self.memory.read_8(self.regs.flat_ip())?;
        self.regs.ip = self.regs.ip.wrapping_add(1);
        Ok(byte)
    }

    pub fn read_ip_i8(&mut self) -> Result<i8, EmulatorError> {
        self.read_ip_u8().map(|data| data as i8)
    }

    pub fn read_ip_mod_rm(&mut self) -> Result<ModRM, EmulatorError> {
        self.read_ip_u8().map(ModRM)
    }

    pub fn read_ip_u16(&mut self) -> Result<u16, EmulatorError> {
        let byte = self.memory.read_16(self.regs.flat_ip())?;
        self.regs.ip += 2;
        Ok(byte)
    }

    pub fn read_ip_u_generic<const N: usize>(&mut self) -> Result<u16, EmulatorError> {
        if N == 16 {
            self.read_ip_u16()
        } else if N == 8 {
            self.read_ip_u8().map(|data| data as u16)
        } else {
            Err(EmulatorError::OutOfBounds)
        }
    }

    fn write_memory_ds<const N: usize>(
        &mut self,
        offset: u16,
        data: u16,
    ) -> Result<(), EmulatorError> {
        let address = self.regs.flat_address(Registers::REG_DS, offset);
        self.memory.write::<N>(address, data)
    }

    fn read_memory_ds<const N: usize>(&mut self, offset: u16) -> Result<u16, EmulatorError> {
        let address = self.regs.flat_address(Registers::REG_DS, offset);
        self.memory.read::<N>(address)
    }

    pub fn call_far_with_32b_displacement(&mut self) -> Result<(), EmulatorError> {
        let address = self.read_ip_u16()?;
        let segment = self.read_ip_u16()?;

        self.push_cs()?;
        self.push_ip()?;

        self.regs.write_segment(Registers::REG_CS, segment);
        self.regs.ip = address;

        Ok(())
    }

    fn calculate_mod_rm_address<const N: usize>(
        &mut self,
        mod_rm: ModRM,
    ) -> Result<u16, EmulatorError> {
        match mod_rm.addressing_mode() {
            0 => match mod_rm.rm() {
                0 => Ok(self
                    .regs
                    .read_gpr_16(Registers::REG_BX)
                    .wrapping_add(self.regs.read_gpr_16(Registers::REG_SI))),
                1 => Ok(self
                    .regs
                    .read_gpr_16(Registers::REG_BX)
                    .wrapping_add(self.regs.read_gpr_16(Registers::REG_DI))),
                2 => Ok(self
                    .regs
                    .read_gpr_16(Registers::REG_BP)
                    .wrapping_add(self.regs.read_gpr_16(Registers::REG_SI))),
                3 => Ok(self
                    .regs
                    .read_gpr_16(Registers::REG_BP)
                    .wrapping_add(self.regs.read_gpr_16(Registers::REG_DI))),
                4 => Ok(self.regs.read_gpr_16(Registers::REG_SI)),
                5 => Ok(self.regs.read_gpr_16(Registers::REG_DI)),
                6 => self.read_ip_u16(),
                7 => Ok(self.regs.read_gpr_16(Registers::REG_BX)),
                _ => unreachable!(),
            },
            1 | 2 => {
                let displacement = if mod_rm.addressing_mode() == 1 {
                    self.read_ip_i8()? as u16
                } else {
                    self.read_ip_u16()?
                };

                let double_register = |register1: u8, register2: u8| {
                    Ok(self
                        .regs
                        .read_gpr_16(register1)
                        .wrapping_add(self.regs.read_gpr_16(register2))
                        .wrapping_add(displacement))
                };

                let single_register =
                    |register: u8| Ok(self.regs.read_gpr_16(register).wrapping_add(displacement));

                match mod_rm.rm() {
                    0 => double_register(Registers::REG_BX, Registers::REG_SI),
                    1 => double_register(Registers::REG_BX, Registers::REG_DI),
                    2 => double_register(Registers::REG_BP, Registers::REG_SI),
                    3 => double_register(Registers::REG_BP, Registers::REG_DI),
                    4 => single_register(Registers::REG_SI),
                    5 => single_register(Registers::REG_DI),
                    6 => single_register(Registers::REG_BP),
                    7 => single_register(Registers::REG_BX),
                    _ => unreachable!(),
                }
            }
            _ => Err(EmulatorError::InvalidOpcode),
        }
    }

    fn read_mod_rm<const N: usize>(&mut self, mod_rm: ModRM) -> Result<u16, EmulatorError> {
        match mod_rm.addressing_mode() {
            0 | 1 | 2 => {
                let offset = self.calculate_mod_rm_address::<N>(mod_rm)?;
                self.read_memory_ds::<N>(offset)
            }
            3 => Ok(self.regs.read_gpr::<N>(mod_rm.rm())),
            _ => unreachable!(),
        }
    }

    fn read_mod_rm_8(&mut self, mod_rm: ModRM) -> Result<u8, EmulatorError> {
        self.read_mod_rm::<8>(mod_rm).map(|data| data as u8)
    }

    fn read_mod_rm_16(&mut self, mod_rm: ModRM) -> Result<u16, EmulatorError> {
        self.read_mod_rm::<16>(mod_rm)
    }

    fn write_mod_rm<const N: usize>(
        &mut self,
        mod_rm: ModRM,
        data: u16,
    ) -> Result<(), EmulatorError> {
        match mod_rm.addressing_mode() {
            0 | 1 | 2 => {
                let offset = self.calculate_mod_rm_address::<N>(mod_rm)?;
                self.write_memory_ds::<N>(offset, data)
            }
            3 => {
                self.regs.write_gpr::<N>(mod_rm.rm(), data);
                Ok(())
            }
            _ => unreachable!(),
        }
    }

    fn write_mod_rm_16(&mut self, mod_rm: ModRM, data: u16) -> Result<(), EmulatorError> {
        self.write_mod_rm::<16>(mod_rm, data)
    }

    fn write_mod_rm_8(&mut self, mod_rm: ModRM, data: u16) -> Result<(), EmulatorError> {
        self.write_mod_rm::<8>(mod_rm, data)
    }

    fn or_r16_rm16(&mut self) -> Result<(), EmulatorError> {
        let mod_rm = self.read_ip_mod_rm()?;
        let result =
            self.read_mod_rm_16(mod_rm)? | self.regs.read_gpr_16(mod_rm.register_destination());
        self.regs
            .write_gpr_16(mod_rm.register_destination(), result);
        self.regs.handle_bitwise_result_u16(result);
        Ok(())
    }

    fn or_rm16_r16(&mut self) -> Result<(), EmulatorError> {
        let mod_rm = self.read_ip_mod_rm()?;
        let old_ip = self.regs.ip;
        let result =
            self.read_mod_rm_16(mod_rm)? | self.regs.read_gpr_16(mod_rm.register_destination());
        self.regs.ip = old_ip;
        self.write_mod_rm_16(mod_rm, result)?;
        self.regs.handle_bitwise_result_u16(result);
        Ok(())
    }

    fn xor_r_rm_generic<const N: usize>(&mut self) -> Result<(), EmulatorError> {
        // TODO: generalise with the above OR function
        let mod_rm = self.read_ip_mod_rm()?;
        let result =
            self.read_mod_rm::<N>(mod_rm)? ^ self.regs.read_gpr::<N>(mod_rm.register_destination());
        self.regs
            .write_gpr::<N>(mod_rm.register_destination(), result);
        self.regs.handle_bitwise_result_u_generic::<N>(result);
        Ok(())
    }

    fn xor_rm_r_generic<const N: usize>(&mut self) -> Result<(), EmulatorError> {
        // TODO: generalise with the above OR function
        let mod_rm = self.read_ip_mod_rm()?;
        let result =
            self.read_mod_rm::<N>(mod_rm)? ^ self.regs.read_gpr::<N>(mod_rm.register_destination());
        self.regs.write_gpr::<N>(mod_rm.register_destination(), result);
        self.regs.handle_bitwise_result_u_generic::<N>(result);
        Ok(())
    }

    fn mov_r16_rm16(&mut self) -> Result<(), EmulatorError> {
        let mod_rm = self.read_ip_mod_rm()?;
        let result = self.read_mod_rm_16(mod_rm)?;
        self.regs
            .write_gpr_16(mod_rm.register_destination(), result);
        Ok(())
    }

    fn mov_r8_rm8(&mut self) -> Result<(), EmulatorError> {
        let mod_rm = self.read_ip_mod_rm()?;
        let result = self.read_mod_rm_8(mod_rm)?;
        self.regs.write_gpr_8(mod_rm.register_destination(), result);
        Ok(())
    }

    fn mov_rm8_r8(&mut self) -> Result<(), EmulatorError> {
        let mod_rm = self.read_ip_mod_rm()?;
        let result = self.regs.read_gpr_8(mod_rm.register_destination());
        self.write_mod_rm_8(mod_rm, result as u16)
    }

    fn mov_rm16_r16(&mut self) -> Result<(), EmulatorError> {
        let mod_rm = self.read_ip_mod_rm()?;
        self.write_mod_rm_16(mod_rm, self.regs.read_gpr_16(mod_rm.register_destination()))
    }

    fn mov_rm_imm_generic<const N: usize>(&mut self) -> Result<(), EmulatorError> {
        let mod_rm = self.read_ip_mod_rm()?;
        let data = self.read_ip_u_generic::<N>()?;
        self.write_mod_rm::<N>(mod_rm, data)
    }

    fn jcc(&mut self, condition: bool) -> Result<(), EmulatorError> {
        let destination_offset = self.read_ip_i8()?;
        if condition {
            self.regs.ip = self.regs.ip.wrapping_add(destination_offset as u16);
        }
        Ok(())
    }

    fn op_0xf6_0xf7_generic<const N: usize>(&mut self) -> Result<(), EmulatorError> {
        let mod_rm = self.read_ip_mod_rm()?;
        match mod_rm.register_destination() {
            0 => {
                let data = self.read_mod_rm_16(mod_rm)?;
                let imm = self.read_ip_u_generic::<N>()?;
                self.regs.handle_bitwise_result_u_generic::<N>(data & imm);
            }
            _ => {
                debug!("[cpu] {}", mod_rm.register_destination());
                unreachable!()
            }
        }
        Ok(())
    }

    fn op_0xf6(&mut self) -> Result<(), EmulatorError> {
        self.op_0xf6_0xf7_generic::<8>()
    }

    fn op_0xf7(&mut self) -> Result<(), EmulatorError> {
        self.op_0xf6_0xf7_generic::<16>()
    }

    fn jmp_rel8(&mut self) -> Result<(), EmulatorError> {
        let destination_offset = self.read_ip_i8()?;
        self.regs.ip = self.regs.ip.wrapping_add(destination_offset as u16);
        Ok(())
    }

    fn jmp_rel16(&mut self) -> Result<(), EmulatorError> {
        let destination_offset = self.read_ip_u16()?;
        self.regs.ip = self.regs.ip.wrapping_add(destination_offset as u16);
        Ok(())
    }

    fn call_near_rel16(&mut self) -> Result<(), EmulatorError> {
        let destination_offset = self.read_ip_u16()? as i16;
        self.push_ip()?;
        self.regs.ip = self.regs.ip.wrapping_add(destination_offset as u16);
        Ok(())
    }

    fn mov_al_imm8(&mut self) -> Result<(), EmulatorError> {
        let data = self.read_ip_u8()?;
        self.regs.write_gpr_lo_8(Registers::REG_AL, data);
        Ok(())
    }

    fn mov_ah_imm8(&mut self) -> Result<(), EmulatorError> {
        let data = self.read_ip_u8()?;
        self.regs.write_gpr_hi_8(Registers::REG_AH, data);
        Ok(())
    }

    fn mov_r16_imm16(&mut self, index: u8) -> Result<(), EmulatorError> {
        let data = self.read_ip_u16()?;
        self.regs.write_gpr_16(index, data);
        Ok(())
    }

    fn ret_near_without_pop(&mut self) -> Result<(), EmulatorError> {
        self.regs.ip = self.pop_value_16()?;
        Ok(())
    }

    fn ret_near_with_pop(&mut self) -> Result<(), EmulatorError> {
        let amount = self.read_ip_u16()?;
        self.ret_near_without_pop()?;
        self.regs.inc_sp(amount);
        Ok(())
    }

    fn ret_far_with_pop(&mut self) -> Result<(), EmulatorError> {
        let amount = self.read_ip_u16()?;
        self.ret_far_without_pop()?;
        self.regs.inc_sp(amount);
        Ok(())
    }

    fn ret_far_without_pop(&mut self) -> Result<(), EmulatorError> {
        self.regs.ip = self.pop_value_16()?;
        let cs = self.pop_value_16()?;
        self.regs.write_segment(Registers::REG_CS, cs);
        Ok(())
    }

    fn int(&mut self) -> Result<(), EmulatorError> {
        let nr = self.read_ip_u8()?;
        debug!(
            "[cpu] interrupt {:x}{}",
            nr,
            if nr >= LOWEST_SYSCALL_INT_VECTOR {
                " (thunk into emulated module)"
            } else {
                ""
            }
        );
        if nr == 0x21 {
            let ah = self.regs.read_gpr_hi_8(Registers::REG_AH);
            if ah == 0x4C {
                debug!(
                    "[cpu] Exit with {}",
                    self.regs.read_gpr_lo_8(Registers::REG_AL)
                );
            } else if ah == 0 {
                debug!("[cpu] Exit with {}", 0);
            } else if ah == 0x30 {
                // Get DOS version, fake MS-DOS 5.0
                // TODO: only al and ah are set right now
                self.regs.write_gpr_16(Registers::REG_AX, 0x0050);
                return Ok(());
            }
            Err(EmulatorError::Exit)
        } else if nr >= LOWEST_SYSCALL_INT_VECTOR {
            // System call handler
            let function = self.regs.read_gpr_16(Registers::REG_AX);
            let accessor = EmulatorAccessor::new(&mut self.memory, &mut self.regs);
            if nr == KERNEL_INT_VECTOR {
                self.emulated_kernel.syscall(function, accessor)
            } else if nr == USER_INT_VECTOR {
                self.emulated_user.syscall(function, accessor)
            } else if nr == GDI_INT_VECTOR {
                self.emulated_gdi.syscall(function, accessor)
            } else if nr == KEYBOARD_INT_VECTOR {
                self.emulated_keyboard.syscall(function, accessor)
            } else {
                Err(EmulatorError::Exit)
            }
        } else {
            Err(EmulatorError::Exit)
        }
    }

    fn mov_segment(&mut self) -> Result<(), EmulatorError> {
        let mod_rm = self.read_ip_mod_rm()?;
        self.write_mod_rm_16(
            mod_rm,
            self.regs.read_segment(mod_rm.register_destination()),
        )?;
        Ok(())
    }

    fn op_0xff(&mut self) -> Result<(), EmulatorError> {
        let mod_rm = self.read_ip_mod_rm()?;
        println!(
            "{:?} {} {} {}",
            mod_rm,
            mod_rm.addressing_mode(),
            mod_rm.register_destination(),
            mod_rm.rm()
        );
        match mod_rm.register_destination() {
            0 => {
                // inc ...
                let old_ip = self.regs.ip;
                let data = self.read_mod_rm_16(mod_rm)?;
                self.regs.ip = old_ip; // Because src = dest for MOD/RM
                let result = data.wrapping_add(1);
                self.regs.handle_arithmetic_result_u16(result);
                self.write_mod_rm_16(mod_rm, result)
            }
            3 => {
                // call far ...
                if mod_rm.addressing_mode() == 3 {
                    Err(EmulatorError::InvalidOpcode)
                } else {
                    let offset = self.calculate_mod_rm_address::<16>(mod_rm)?;
                    let segment = self.read_memory_ds::<16>(offset + 2)?;
                    let offset_within_segment = self.read_memory_ds::<16>(offset)?;
                    self.push_cs()?;
                    self.push_ip()?;
                    self.regs.write_segment(Registers::REG_CS, segment);
                    self.regs.ip = offset_within_segment;
                    Ok(())
                }
            }
            6 => {
                // push ...
                let data = self.read_mod_rm_16(mod_rm)?;
                self.push_value_16(data)
            }
            _ => unreachable!(),
        }
    }

    fn op_0x83(&mut self) -> Result<(), EmulatorError> {
        let mod_rm = self.read_ip_mod_rm()?;
        match mod_rm.register_destination() {
            0 => {
                let data = self.read_ip_i8()?;
                let result = self.read_mod_rm_16(mod_rm)?.wrapping_add(data as u16);
                self.write_mod_rm_16(mod_rm, result)?;
                self.regs.handle_arithmetic_result_u16(result);
            }
            5 => {
                let data = self.read_ip_i8()?;
                let result = self.read_mod_rm_16(mod_rm)?.wrapping_sub(data as u16);
                self.write_mod_rm_16(mod_rm, result)?;
                self.regs.handle_arithmetic_result_u16(result);
            }
            7 => {
                let result = self.read_mod_rm_16(mod_rm)?;
                let data = self.read_ip_i8()?;
                let result = result.wrapping_sub(data as u16);
                self.regs.handle_arithmetic_result_u16(result);
            }
            _ => {
                debug!("[cpu] {}", mod_rm.register_destination());
                unreachable!()
            }
        }
        Ok(())
    }

    fn cmp_r_rm<const N: usize>(&mut self) -> Result<(), EmulatorError> {
        let mod_rm = self.read_ip_mod_rm()?;
        let result = self
            .regs
            .read_gpr::<N>(mod_rm.register_destination())
            .wrapping_sub(self.read_mod_rm::<N>(mod_rm)?);
        self.regs.handle_arithmetic_result_u_generic::<N>(result);
        Ok(())
    }

    fn cmp_r8_rm8(&mut self) -> Result<(), EmulatorError> {
        self.cmp_r_rm::<8>()
    }

    fn cmp_r16_rm16(&mut self) -> Result<(), EmulatorError> {
        self.cmp_r_rm::<16>()
    }

    fn cmp_r_imm<const N: usize>(&mut self, reg: u8) -> Result<(), EmulatorError> {
        let immediate = self.read_ip_u_generic::<N>()?;
        let result = self.regs.read_gpr::<N>(reg).wrapping_sub(immediate);
        self.regs.handle_arithmetic_result_u_generic::<N>(result);
        Ok(())
    }

    fn cmp_r8_imm8(&mut self, reg: u8) -> Result<(), EmulatorError> {
        self.cmp_r_imm::<8>(reg)
    }

    fn cmp_r16_imm16(&mut self, reg: u8) -> Result<(), EmulatorError> {
        self.cmp_r_imm::<16>(reg)
    }

    fn sub_r_rm<const N: usize>(&mut self) -> Result<(), EmulatorError> {
        let mod_rm = self.read_ip_mod_rm()?;
        let result = self
            .regs
            .read_gpr::<N>(mod_rm.register_destination())
            .wrapping_sub(self.read_mod_rm::<N>(mod_rm)?);
        self.regs
            .write_gpr::<N>(mod_rm.register_destination(), result);
        self.regs.handle_arithmetic_result_u_generic::<N>(result);
        Ok(())
    }

    fn sub_r8_rm8(&mut self) -> Result<(), EmulatorError> {
        self.sub_r_rm::<8>()
    }

    fn sub_r16_rm16(&mut self) -> Result<(), EmulatorError> {
        self.sub_r_rm::<16>()
    }

    fn add_rm_r<const N: usize>(&mut self) -> Result<(), EmulatorError> {
        let mod_rm = self.read_ip_mod_rm()?;
        let old_ip = self.regs.ip;
        let result = self
            .regs
            .read_gpr::<N>(mod_rm.register_destination())
            .wrapping_add(self.read_mod_rm::<N>(mod_rm)?);
        self.regs.ip = old_ip; // Because src = dest for MOD/RM
        self.write_mod_rm::<N>(mod_rm, result)?;
        self.regs.handle_arithmetic_result_u_generic::<N>(result);
        Ok(())
    }

    fn add_rm8_r8(&mut self) -> Result<(), EmulatorError> {
        self.add_rm_r::<8>()
    }

    fn add_rm16_r16(&mut self) -> Result<(), EmulatorError> {
        self.add_rm_r::<16>()
    }

    fn nop(&self) -> Result<(), EmulatorError> {
        Ok(())
    }

    fn set_direction_flag(&mut self, flag: bool) -> Result<(), EmulatorError> {
        self.regs.set_direction_flag(flag);
        Ok(())
    }

    fn stosb(&mut self) -> Result<(), EmulatorError> {
        todo!("STOSB");
    }

    fn rep(&mut self) -> Result<(), EmulatorError> {
        todo!("REP");
    }

    fn lea(&mut self) -> Result<(), EmulatorError> {
        let mod_rm = self.read_ip_mod_rm()?;
        let data = self.calculate_mod_rm_address::<16>(mod_rm)?;
        self.regs.write_gpr_16(mod_rm.register_destination(), data);
        Ok(())
    }

    fn mov_moffs16_ax(&mut self) -> Result<(), EmulatorError> {
        let offset = self.read_ip_u16()?;
        self.write_memory_ds::<16>(offset, self.regs.read_gpr_16(Registers::REG_AX))
    }

    fn mov_ax_moffs16(&mut self) -> Result<(), EmulatorError> {
        let offset = self.read_ip_u16()?;
        let data = self.read_memory_ds::<16>(offset)?;
        self.regs.write_gpr_16(Registers::REG_AX, data);
        Ok(())
    }

    fn mov_moffs8_al(&mut self) -> Result<(), EmulatorError> {
        let offset = self.read_ip_u16()?;
        self.write_memory_ds::<8>(offset, self.regs.read_gpr_lo_8(Registers::REG_AL) as u16)
    }

    fn cwd(&mut self) -> Result<(), EmulatorError> {
        self.regs.write_gpr_16(
            Registers::REG_DX,
            self.regs.read_gpr_16(Registers::REG_AX) >> 15,
        );
        Ok(())
    }

    fn push_imm8(&mut self) -> Result<(), EmulatorError> {
        let data = self.read_ip_u8()?;
        self.push_value_16(data as u16)
    }

    fn push_imm16(&mut self) -> Result<(), EmulatorError> {
        let data = self.read_ip_u16()?;
        self.push_value_16(data)
    }

    pub fn read_opcode(&mut self) -> Result<(), EmulatorError> {
        match self.read_ip_u8()? {
            0x01 => self.add_rm16_r16(),
            0x06 => self.push_segment_16(Registers::REG_ES),
            0x07 => self.pop_segment_16(Registers::REG_ES),
            0x09 => self.or_rm16_r16(),
            0x0B => self.or_r16_rm16(),
            0x0E => self.push_segment_16(Registers::REG_CS),
            0x16 => self.push_segment_16(Registers::REG_SS),
            0x1E => self.push_segment_16(Registers::REG_DS),
            0x2A => self.sub_r8_rm8(),
            0x2B => self.sub_r16_rm16(),
            0x3B => self.cmp_r16_rm16(),
            0x3C => self.cmp_r8_rm8(),
            0x3D => self.cmp_r16_imm16(Registers::REG_AX),
            0x31 => self.xor_rm_r_generic::<8>(),
            0x32 => self.xor_r_rm_generic::<8>(),
            0x33 => self.xor_r_rm_generic::<16>(),
            0x50 => self.push_gpr_16(Registers::REG_AX),
            0x51 => self.push_gpr_16(Registers::REG_CX),
            0x52 => self.push_gpr_16(Registers::REG_DX),
            0x53 => self.push_gpr_16(Registers::REG_BX),
            0x54 => self.push_gpr_16(Registers::REG_SP),
            0x55 => self.push_gpr_16(Registers::REG_BP),
            0x56 => self.pop_gpr_16(Registers::REG_SI),
            0x57 => self.pop_gpr_16(Registers::REG_DI),
            0x58 => self.pop_gpr_16(Registers::REG_AX),
            0x5D => self.pop_gpr_16(Registers::REG_BP),
            0x5E => self.pop_gpr_16(Registers::REG_SI),
            0x5F => self.pop_gpr_16(Registers::REG_DI),
            0x68 => self.push_imm16(),
            0x6a => self.push_imm8(),
            0x72 => self.jcc(self.regs.flag_carry()),
            0x73 => self.jcc(!self.regs.flag_carry()),
            0x74 => self.jcc(self.regs.flag_zero()),
            0x75 => self.jcc(!self.regs.flag_zero()),
            0x7E => self
                .jcc(self.regs.flag_zero() | (self.regs.flag_sign() ^ self.regs.flag_overflow())),
            0x83 => self.op_0x83(),
            0x8B => self.mov_r16_rm16(),
            0x89 => self.mov_rm16_r16(),
            0x88 => self.mov_rm8_r8(),
            0x8A => self.mov_r8_rm8(),
            0x8C => self.mov_segment(),
            0x8D => self.lea(),
            0x90 => self.nop(),
            0x99 => self.cwd(),
            0x9A => self.call_far_with_32b_displacement(),
            0xAA => self.stosb(),
            0xA1 => self.mov_ax_moffs16(),
            0xA2 => self.mov_moffs8_al(),
            0xA3 => self.mov_moffs16_ax(),
            0xB0 => self.mov_al_imm8(),
            0xB4 => self.mov_ah_imm8(),
            0xB8 => self.mov_r16_imm16(Registers::REG_AX),
            0xB9 => self.mov_r16_imm16(Registers::REG_CX),
            0xBA => self.mov_r16_imm16(Registers::REG_DX),
            0xBF => self.mov_r16_imm16(Registers::REG_DI),
            0xC2 => self.ret_near_with_pop(),
            0xC3 => self.ret_near_without_pop(),
            0xC6 => self.mov_rm_imm_generic::<8>(),
            0xC7 => self.mov_rm_imm_generic::<16>(),
            0xCA => self.ret_far_with_pop(),
            0xCB => self.ret_far_without_pop(),
            0xCD => self.int(),
            0xE9 => self.jmp_rel16(),
            0xEB => self.jmp_rel8(),
            0xE8 => self.call_near_rel16(),
            0xF3 => self.rep(),
            0xF6 => self.op_0xf6(),
            0xF7 => self.op_0xf7(),
            0xFC => self.set_direction_flag(false),
            0xFF => self.op_0xff(),
            nr => {
                debug!("[cpu] unknown opcode {:x}", nr);
                Err(EmulatorError::InvalidOpcode)
            }
        }
    }

    pub fn step(&mut self) {
        debug!(
            "[cpu] Currently at {:x}:{:x}, AX={:x}, BX={:x}, CX={:x}, DX={:x}, SP={:x}, BP={:x}, FLAGS={:016b}",
            self.regs.read_segment(Registers::REG_CS),
            self.regs.ip,
            self.regs.read_gpr_16(Registers::REG_AX),
            self.regs.read_gpr_16(Registers::REG_BX),
            self.regs.read_gpr_16(Registers::REG_CX),
            self.regs.read_gpr_16(Registers::REG_DX),
            self.regs.read_gpr_16(Registers::REG_SP),
            self.regs.read_gpr_16(Registers::REG_BP),
            self.regs.flags(),
        );
        self.read_opcode().expect("todo");
    }

    pub fn run(&mut self) {
        loop {
            self.step();
        }
    }
}

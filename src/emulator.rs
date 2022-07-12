use crate::emulated_kernel::EmulatedKernel;
use crate::emulator_error::EmulatorError;
use crate::memory::Memory;
use crate::mod_rm::ModRM;
use crate::registers::Registers;
use crate::u16_from_slice;
use num_traits::PrimInt;

pub struct Emulator {
    regs: Registers,
    memory: Memory,
    emulated_kernel: EmulatedKernel,
}

impl Emulator {
    pub fn new(memory: Memory, cs: u16, ip: u16, emulated_kernel: EmulatedKernel) -> Self {
        Self {
            regs: Registers::new(cs, ip),
            memory,
            emulated_kernel,
        }
    }

    fn push_value_16(&mut self, data: u16) -> Result<(), EmulatorError> {
        println!("push {:x}", data);
        // TODO: keep in mind the stack segment, because this is wrong now
        self.regs.dec_sp(2);
        self.memory
            .write_16(self.regs.read_gpr_16(Registers::REG_SP) as u32, data)?;
        Ok(())
    }

    fn pop_value_16(&mut self) -> Result<u16, EmulatorError> {
        // TODO: keep in mind the stack segment, because this is wrong now
        let data = self
            .memory
            .read_16(self.regs.read_gpr_16(Registers::REG_SP) as u32)?;
        println!("pop {:x}", data);
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

    fn read_u8_at(&self, offset: usize) -> Result<u8, EmulatorError> {
        // TODO: cast is ugly
        self.memory.read_8(offset as u32)
    }

    fn read_u16_at(&self, offset: usize) -> Result<u16, EmulatorError> {
        // TODO: cast is ugly
        self.memory.read_16(offset as u32)
    }

    fn flat_ip(&self) -> usize {
        //println!("{:x}:{:x}", self.regs.read_segment(Registers::REG_CS), self.regs.ip);
        self.regs.ip as usize + ((self.regs.read_segment(Registers::REG_CS) as usize) << 4)
    }

    pub fn read_ip_u8(&mut self) -> Result<u8, EmulatorError> {
        let byte = self.read_u8_at(self.flat_ip())?;
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
        let byte = self.read_u16_at(self.flat_ip())?;
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

    pub fn call_far_with_32b_displacement(&mut self) -> Result<(), EmulatorError> {
        let address = self.read_ip_u16()?;
        let segment = self.read_ip_u16()?;

        self.push_cs()?;
        self.push_ip()?;

        self.regs.write_segment(Registers::REG_CS, segment);
        self.regs.ip = address;

        Ok(())
    }

    fn read_mod_rm<const N: usize>(&mut self, mod_rm: ModRM) -> Result<u16, EmulatorError> {
        match mod_rm.addressing_mode() {
            0 => match mod_rm.rm() {
                6 => {
                    let disp16 = self.read_ip_u16()?;
                    // TODO: keep segment in mind
                    self.memory.read::<N>(disp16 as u32)
                }
                _ => {
                    assert!(false);
                    Ok(0)
                }
            },
            2 => {
                assert!(false);
                Ok(0)
            }
            1 => match mod_rm.rm() {
                // TODO
                6 => {
                    // [bp + disp8]
                    let address = self
                        .regs
                        .read_gpr_16(Registers::REG_BP)
                        .wrapping_add((self.read_ip_i8()?) as u16);
                    // TODO: keep in mind segment ig
                    self.memory.read::<N>(address as u32)
                }
                _ => unreachable!(),
            },
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

    fn write_mod_rm_16(&mut self, mod_rm: ModRM, data: u16) -> Result<(), EmulatorError> {
        match mod_rm.addressing_mode() {
            0 => match mod_rm.rm() {
                6 => {
                    let disp16 = self.read_ip_u16()?;
                    // TODO: keep in mind data segment(?)
                    self.memory.write_16(disp16 as u32, data)
                }
                _ => {
                    assert!(false);
                    Ok(())
                }
            },
            1 => match mod_rm.rm() {
                6 => {
                    let disp8 = self.read_ip_i8()?;
                    let address = self
                        .regs
                        .read_gpr_16(Registers::REG_BP)
                        .wrapping_add(disp8 as u16);
                    // TODO: keep in mind data segment(?)
                    self.memory.write_16(address as u32, data)
                }

                _ => {
                    println!(
                        "{} {} {}",
                        mod_rm.addressing_mode(),
                        mod_rm.register_destination(),
                        mod_rm.rm()
                    );
                    assert!(false);
                    Ok(())
                }
            },
            2 => {
                println!(
                    "{} {} {}",
                    mod_rm.addressing_mode(),
                    mod_rm.register_destination(),
                    mod_rm.rm()
                );
                assert!(false);
                Ok(())
            }
            3 => {
                self.regs.write_gpr_16(mod_rm.rm(), data);
                Ok(())
            }
            _ => unreachable!(),
        }
    }

    fn or_r16(&mut self) -> Result<(), EmulatorError> {
        let mod_rm = self.read_ip_mod_rm()?;
        let result =
            self.read_mod_rm_16(mod_rm)? | self.regs.read_gpr_16(mod_rm.register_destination());
        self.regs
            .write_gpr_16(mod_rm.register_destination(), result);
        self.regs.handle_bitwise_result(result);
        Ok(())
    }

    fn xor_r16(&mut self) -> Result<(), EmulatorError> {
        // TODO: generalise with the above OR function
        let mod_rm = self.read_ip_mod_rm()?;
        let result =
            self.read_mod_rm_16(mod_rm)? ^ self.regs.read_gpr_16(mod_rm.register_destination());
        self.regs
            .write_gpr_16(mod_rm.register_destination(), result);
        self.regs.handle_bitwise_result(result);
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

    fn mov_rm16_r16(&mut self) -> Result<(), EmulatorError> {
        let mod_rm = self.read_ip_mod_rm()?;
        self.write_mod_rm_16(mod_rm, self.regs.read_gpr_16(mod_rm.register_destination()))?;
        Ok(())
    }

    fn mov_rm16_imm16(&mut self) -> Result<(), EmulatorError> {
        let mod_rm = self.read_ip_mod_rm()?;
        let data = self.read_ip_u16()?;
        self.write_mod_rm_16(mod_rm, data)?;
        Ok(())
    }

    fn jz(&mut self) -> Result<(), EmulatorError> {
        let destination_offset = self.read_ip_i8()?;
        if self.regs.flag_zero() {
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
                self.regs.handle_bitwise_result(data & imm);
            }
            _ => {
                println!("{}", mod_rm.register_destination());
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

    fn jmp(&mut self) -> Result<(), EmulatorError> {
        let destination_offset = self.read_ip_i8()?;
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

    fn mov_ax_imm16(&mut self) -> Result<(), EmulatorError> {
        let data = self.read_ip_u16()?;
        self.regs.write_gpr_16(Registers::REG_AX, data);
        Ok(())
    }

    fn mov_dx_imm16(&mut self) -> Result<(), EmulatorError> {
        let data = self.read_ip_u16()?;
        self.regs.write_gpr_16(Registers::REG_DX, data);
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
        if nr == 0x21 {
            if self.regs.read_gpr_hi_8(Registers::REG_AH) == 0x4C {
                println!("Exit with {}", self.regs.read_gpr_lo_8(Registers::REG_AL));
            }
            Err(EmulatorError::Exit)
        } else if nr == 0xff {
            // System call handler
            self.emulated_kernel.syscall(&mut self.regs);
            Ok(())
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
            6 => {
                let data = self.read_mod_rm_16(mod_rm)?;
                self.push_value_16(data)?;
            }
            _ => unreachable!(),
        }
        Ok(())
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
                println!("{}", mod_rm.register_destination());
                unreachable!()
            }
        }
        Ok(())
    }

    fn sub_rn_rmn<const N: usize, R: PrimInt>(&mut self) -> Result<(), EmulatorError> {
        let mod_rm = self.read_ip_mod_rm()?;
        let result = self
            .regs
            .read_gpr::<N>(mod_rm.register_destination())
            .wrapping_sub(self.read_mod_rm::<N>(mod_rm)?);
        self.regs
            .write_gpr::<N>(mod_rm.register_destination(), result);
        self.regs.handle_arithmetic_result_u_generic::<N, R>(result);
        Ok(())
    }

    fn sub_r8_rm8(&mut self) -> Result<(), EmulatorError> {
        self.sub_rn_rmn::<8, u8>()
    }

    fn sub_r16_rm16(&mut self) -> Result<(), EmulatorError> {
        self.sub_rn_rmn::<16, u16>()
    }

    fn nop(&self) -> Result<(), EmulatorError> {
        Ok(())
    }

    pub fn read_opcode(&mut self) -> Result<(), EmulatorError> {
        let op = self.read_ip_u8()?;
        println!("  op: {:x}", op);
        match op {
            0x0B => self.or_r16(),
            0x1E => self.push_segment_16(Registers::REG_DS),
            0x2A => self.sub_r8_rm8(),
            0x2B => self.sub_r16_rm16(),
            0x33 => self.xor_r16(),
            0x50 => self.push_gpr_16(Registers::REG_AX),
            0x52 => self.push_gpr_16(Registers::REG_DX),
            0x55 => self.push_gpr_16(Registers::REG_BP),
            0x58 => self.pop_gpr_16(Registers::REG_AX),
            0x5D => self.pop_gpr_16(Registers::REG_BP),
            0x74 => self.jz(),
            0x83 => self.op_0x83(),
            0x8B => self.mov_r16_rm16(),
            0x89 => self.mov_rm16_r16(),
            0x8A => self.mov_r8_rm8(),
            0x8C => self.mov_segment(),
            0x90 => self.nop(),
            0x9A => self.call_far_with_32b_displacement(),
            0xB0 => self.mov_al_imm8(),
            0xB4 => self.mov_ah_imm8(),
            0xB8 => self.mov_ax_imm16(),
            0xBA => self.mov_dx_imm16(),
            0xC2 => self.ret_near_with_pop(),
            0xC3 => self.ret_near_without_pop(),
            0xC7 => self.mov_rm16_imm16(),
            0xCA => self.ret_far_with_pop(),
            0xCB => self.ret_far_without_pop(),
            0xCD => self.int(),
            0xEB => self.jmp(),
            0xE8 => self.call_near_rel16(),
            0xF6 => self.op_0xf6(),
            0xF7 => self.op_0xf7(),
            0xFF => self.op_0xff(),
            other => Err(EmulatorError::InvalidOpcode),
        }
    }

    pub fn step(&mut self) {
        println!(
            "Currently at {:x}:{:x}, AX={:x}, BX={:x}, CX={:x}, DX={:x}, SP={:x}, BP={:x}, FLAGS={:016b}",
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

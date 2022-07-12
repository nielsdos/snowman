use std::collections::HashMap;
use crate::emulated_kernel::EmulatedKernel;
use crate::emulated_user::EmulatedUser;
use crate::emulator::Emulator;
use crate::emulator_error::EmulatorError;
use crate::executable::{Executable, ExecutableFormatError};
use crate::memory::Memory;
use crate::module::{KernelModule, Module, UserModule};
use crate::util::{bool_to_result, u16_from_slice};

mod constants;
mod emulated_kernel;
mod emulated_user;
mod emulator;
mod emulator_accessor;
mod emulator_error;
mod executable;
mod memory;
mod mod_rm;
mod module;
mod registers;
mod util;

struct MZResult {
    pub ne_header_offset: usize,
}

fn main() {
    let mut bytes = std::fs::read("../vms/WINVER.EXE").expect("test file should exist");
    let mut executable = Executable::new(bytes.as_mut_slice());
    println!("{:?}", process_file(&mut executable));
}

fn process_file_mz(executable: &Executable) -> Result<MZResult, ExecutableFormatError> {
    executable.validate_magic_id(0, b"MZ")?;
    // TODO: check MZ checksum
    let ne_header_offset = executable.read_u16(0x3C)? as usize;
    Ok(MZResult { ne_header_offset })
}

fn validate_application_flags(executable: &Executable) -> Result<(), ExecutableFormatError> {
    bool_to_result(
        (executable.read_u8(0x0D)? & 0b11101000) == 0,
        ExecutableFormatError::ApplicationFlags,
    )
}

fn validate_target_operating_system(executable: &Executable) -> Result<(), ExecutableFormatError> {
    let byte = executable.read_u8(0x36)?;
    bool_to_result(
        byte == 0 || byte == 2 || byte == 4,
        ExecutableFormatError::OperatingSystem,
    )
}

#[derive(Debug)]
struct ImportOrdinalRelocation {
    index_into_module_reference_table: u16,
    procedure_ordinal_number: u16,
}

#[derive(Debug)]
struct InternalRefRelocation {
    parameter: u16,
    segment_number: u8,
}

#[derive(Debug)]
enum RelocationType {
    ImportOrdinal(ImportOrdinalRelocation),
    InternalRef(InternalRefRelocation),
}

#[derive(Debug)]
struct Relocation {
    relocation_type: RelocationType,
    locations: Vec<u16>,
    offset_within_segment_from_source_chain: u16,
    source_type: u8,
}

#[derive(Debug)]
struct Segment {
    pub logical_sector_offset: u32,
    pub length_of_segment_in_file: u32,
    pub minimum_allocation_size: u32,
    pub relocations: Option<Vec<Relocation>>,
}

type SegmentTable = Vec<Segment>;

fn process_segment_table(
    executable: &mut Executable,
    offset_to_segment_table: usize,
    segment_count: usize,
    file_alignment_size_shift: usize,
) -> Result<SegmentTable, ExecutableFormatError> {
    let segment_table_cursor = executable.seek_from_here(offset_to_segment_table)?;

    let mut segments = Vec::with_capacity(segment_count);

    for segment_index in 0..segment_count {
        let byte_offset = segment_index * 8;

        let map_zero_to_64k = |data: u16| {
            if data == 0 {
                65536
            } else {
                data as u32
            }
        };

        let logical_sector_offset =
            (executable.read_u16(byte_offset)? as u32) << file_alignment_size_shift;
        let length_of_segment_in_file = map_zero_to_64k(executable.read_u16(byte_offset + 2)?);
        let flags = executable.read_u16(byte_offset + 4)? as u32;

        // Read relocation data
        let relocations = if (flags & 0x100) == 0x100 {
            let relocation_old_cursor = executable.seek_from_start(
                logical_sector_offset as usize + length_of_segment_in_file as usize,
            )?;
            let relocation_count = executable.read_u16(0)?;

            let mut relocations = Vec::with_capacity(relocation_count as usize);

            for relocation_index in 0..relocation_count {
                let byte_offset = 2 + relocation_index as usize * 8;

                let source_type = executable.read_u8(byte_offset)?;
                let flags = executable.read_u8(byte_offset + 1)?;
                let offset_within_segment_from_source_chain =
                    executable.read_u16(byte_offset + 2)?;

                let old_cursor = executable.seek_from_start(logical_sector_offset as usize)?;
                let mut relocation_locations = Vec::new();

                // Walk the linked list of the offsets
                let mut offset_cursor = offset_within_segment_from_source_chain;
                // TODO: avoid loops in the linked list
                loop {
                    relocation_locations.push(offset_cursor);
                    let pointer = executable.read_u16(offset_cursor as usize)?;

                    if pointer == 0xffff {
                        break;
                    }

                    if (flags & 4) == 0 {
                        // Additive flag is not set
                        offset_cursor = pointer;
                    } else {
                        // Additive flag is set
                        if pointer == 0 {
                            break;
                        }
                        offset_cursor += pointer;
                    }
                }

                executable.restore_cursor(old_cursor);

                match flags & 3 {
                    // Internal ref
                    0 => {
                        let segment_number = executable.read_u8(byte_offset + 4)?;
                        let parameter = executable.read_u16(byte_offset + 6)?;
                        relocations.push(Relocation {
                            relocation_type: RelocationType::InternalRef(
                                InternalRefRelocation {
                                    segment_number,
                                    parameter,
                                },
                            ),
                            locations: relocation_locations,
                            offset_within_segment_from_source_chain,
                            source_type,
                        });
                    }
                    // Import ordinal
                    1 => {
                        let index_into_module_reference_table =
                            executable.read_u16(byte_offset + 4)?;
                        let procedure_ordinal_number = executable.read_u16(byte_offset + 6)?;
                        relocations.push(Relocation {
                            relocation_type: RelocationType::ImportOrdinal(
                                ImportOrdinalRelocation {
                                    index_into_module_reference_table,
                                    procedure_ordinal_number,
                                },
                            ),
                            locations: relocation_locations,
                            offset_within_segment_from_source_chain,
                            source_type,
                        });
                    }
                    // Import name
                    2 => {
                        println!("import name");
                    }
                    // OS fixup
                    3 => {
                        println!("OS fixup");
                    }
                    _ => unreachable!(),
                }
            }

            executable.restore_cursor(relocation_old_cursor);

            Some(relocations)
        } else {
            None
        };

        segments.push(Segment {
            logical_sector_offset,
            length_of_segment_in_file,
            minimum_allocation_size: map_zero_to_64k(executable.read_u16(byte_offset + 6)?),
            relocations,
        });
    }

    executable.restore_cursor(segment_table_cursor);

    Ok(segments)
}

fn validate_segment_index_and_offset(
    segment_table: &SegmentTable,
    segment: u16,
    offset: u16,
) -> Result<(), ExecutableFormatError> {
    // TODO: handle segment 0
    assert!(segment >= 1);

    bool_to_result(
        (segment as usize - 1) < segment_table.len(),
        ExecutableFormatError::SegmentIndex,
    )?;
    bool_to_result(
        (offset as u32) < segment_table[segment as usize - 1].minimum_allocation_size,
        ExecutableFormatError::SegmentOffset,
    )?;
    Ok(())
}

struct ModuleReferenceTable {
    modules: Vec<Box<dyn Module>>,
}

impl ModuleReferenceTable {
    pub fn module(&self, index: u16) -> Result<&dyn Module, EmulatorError> {
        if index >= 1 && (index as usize) <= self.modules.len() {
            Ok(&*self.modules[index as usize - 1])
        } else {
            Err(EmulatorError::OutOfBounds)
        }
    }
}

fn process_module_reference_table(
    executable: &Executable,
    offset_to_module_reference_table: usize,
    module_reference_count: u16,
) -> Result<ModuleReferenceTable, ExecutableFormatError> {
    let offset_to_imported_name_table = executable.read_u16(0x2A)? as usize;

    let mut module_reference_table = ModuleReferenceTable {
        modules: Vec::with_capacity(module_reference_count as usize),
    };

    for module_index in 0..module_reference_count {
        let module_name_offset_in_imported_name_table =
            executable.read_u16(offset_to_module_reference_table + (module_index * 2) as usize)?;
        let start_offset =
            offset_to_imported_name_table + module_name_offset_in_imported_name_table as usize;
        let module_name_length = executable.read_u8(start_offset)?;
        let module_name = executable.slice(start_offset + 1, module_name_length as usize)?;

        if module_name == b"KERNEL" {
            module_reference_table
                .modules
                .push(Box::new(KernelModule::new(0x10 * 0x1000))); // TODO: better address
        } else if module_name == b"USER" {
            module_reference_table
                .modules
                .push(Box::new(UserModule::new(0x10 * 0x2000))); // TODO: better address
        }
    }

    Ok(module_reference_table)
}

#[derive(Debug, Copy, Clone)]
struct EntryTableEntry {
    pub offset: u16,
    pub segment_number: u8,
}

struct EntryTable {
    entries: HashMap<u16, EntryTableEntry>,
}

impl EntryTable {
    pub fn get(&self, ordinal_index: u16) -> Option<EntryTableEntry> {
        self.entries.get(&ordinal_index).copied()
    }
}

fn process_entry_table(executable: &mut Executable, offset_to_entry_table: usize, entry_table_bytes: usize) -> Result<EntryTable, ExecutableFormatError> {
    let old_cursor = executable.seek_from_here(offset_to_entry_table)?;

    let mut entry_table = EntryTable {
        entries: HashMap::new(),
    };

    let mut offset = 0;
    let mut ordinal_index = 1u16;
    while offset < entry_table_bytes {
        let number_of_entries = executable.read_u8(offset + 0)?;
        if number_of_entries == 0 {
            break;
        }
        let segment_indicator = executable.read_u8(offset + 1)?;
        offset += 2;
        if segment_indicator == 0 {
            ordinal_index += number_of_entries as u16;
            continue;
        }

        for _ in 0..number_of_entries {
            let flag = executable.read_u8(offset)?;

            if segment_indicator == 0xff {
                // TODO: expect 16-bit number 0x3FCD here
                let magic = executable.read_u16(offset + 1)?;
                let segment_number = executable.read_u8(offset + 3)?;
                let offset_within_segment_to_entry_point = executable.read_u16(offset + 4)?;
                println!("movable segment {:x} {:x} {:x}", magic, segment_number, offset_within_segment_to_entry_point);

                entry_table.entries.insert(ordinal_index, EntryTableEntry {
                    segment_number,
                    offset: offset_within_segment_to_entry_point,
                });

                offset += 6;
            } else {
                // TODO: fixed segment
                let offset_within_segment_to_entry_point = executable.read_u16(offset + 1)?;
                println!("fixed segment {:x}", offset_within_segment_to_entry_point);
                offset += 3;
            }

            ordinal_index += 1;
        }
    }

    executable.restore_cursor(old_cursor);
    Ok(entry_table)
}

fn perform_relocations(
    memory: &mut Memory,
    ip: u32,
    module_reference_table: &ModuleReferenceTable,
    entry_table: &EntryTable,
    code_segment: &Segment,
) -> Result<(), EmulatorError> {
    if let Some(relocations) = code_segment.relocations.as_ref() {
        for relocation in relocations {
            match &relocation.relocation_type {
                RelocationType::ImportOrdinal(import) => {
                    // Relocate kernel system call
                    let module =
                        module_reference_table.module(import.index_into_module_reference_table)?;
                    let segment_and_offset = module.base_module().procedure(
                        memory,
                        import.procedure_ordinal_number,
                        module.argument_bytes_of_procedure(import.procedure_ordinal_number),
                    )?;

                    for &offset in &relocation.locations {
                        let flat_address = ip + offset as u32;
                        if relocation.source_type == 3 {
                            memory.write_16(flat_address, segment_and_offset.offset)?;
                            memory.write_16(flat_address + 2, segment_and_offset.segment)?;
                        } else {
                            // TODO
                        }
                    }
                }
                RelocationType::InternalRef(internal_ref) => {
                    println!("internal ref {:?}", internal_ref);

                    let (segment, offset_within_segment) = if internal_ref.segment_number == 0xff {
                        let ordinal_index_into_entry_table = internal_ref.parameter;
                        let entry = entry_table.get(ordinal_index_into_entry_table).ok_or(EmulatorError::OutOfBounds)?;
                        println!("relocation entry {:?}", entry);

                        // TODO: this is hardcoded
                        let segment = if entry.segment_number == 1 { 0u16 } else {
                            todo!()
                        };
                        (segment, entry.offset)
                    } else {
                        todo!()
                    };

                    for &offset in &relocation.locations {
                        let flat_address = ip + offset as u32;

                        if relocation.source_type == 2 {
                            memory.write_16(flat_address, segment)?;
                        } else if relocation.source_type == 5 {
                            memory.write_16(flat_address, offset_within_segment)?;
                        } else {
                            // TODO: invalid?
                        }

                        println!("relocate at {:x}, {}, {:x}:{:x}", flat_address, relocation.source_type, segment, offset_within_segment);
                    }
                }
                _ => {
                    // TODO
                }
            }
        }
    }

    Ok(())
}

fn process_file_ne(
    executable: &mut Executable,
    ne_header_offset: usize,
) -> Result<(), ExecutableFormatError> {
    let old_cursor = executable.seek_from_start(ne_header_offset)?;
    executable.validate_magic_id(0, b"NE")?;
    validate_application_flags(executable)?;
    validate_target_operating_system(executable)?;

    let offset_to_entry_table = executable.read_u16(0x04)? as usize;
    let entry_table_bytes = executable.read_u16(0x06)? as usize;
    let segment_table_segment_count = executable.read_u16(0x1C)? as usize;
    let module_reference_count = executable.read_u16(0x1E)?;
    let offset_to_segment_table = executable.read_u16(0x22)? as usize;
    let offset_to_module_reference_table = executable.read_u16(0x28)? as usize;
    let file_alignment_size_shift = {
        let shift = executable.read_u16(0x32)?;
        if shift == 0 {
            9
        } else {
            shift as usize
        }
    };

    let module_reference_table = process_module_reference_table(
        executable,
        offset_to_module_reference_table,
        module_reference_count,
    )?;

    let entry_table = process_entry_table(executable, offset_to_entry_table, entry_table_bytes)?;

    println!(
        "Expected Windows version: {}.{}",
        executable.read_u8(0x3F)?,
        executable.read_u8(0x3E)?
    );

    let cs = executable.read_u16(0x16)?;
    let ip = executable.read_u16(0x14)?;
    let ds = executable.read_u16(0x0E)?;
    let ss = executable.read_u16(0x1A)?;
    let sp = executable.read_u16(0x18)?;

    println!("CS:IP data: {:x} {:x}", cs, ip);
    println!("SS:SP data: {:x} {:x}", ss, sp);
    println!("DS: {:x}", ds);

    let segment_table = process_segment_table(
        executable,
        offset_to_segment_table,
        segment_table_segment_count,
        file_alignment_size_shift,
    )?;

    validate_segment_index_and_offset(&segment_table, cs, ip)?;
    validate_segment_index_and_offset(&segment_table, ss, sp)?;

    executable.restore_cursor(old_cursor);

    let mut memory = Memory::new();

    // TODO: handle all segments (including 0 segment rules)
    let data_segment = &segment_table[ds as usize - 1];
    let code_segment = &segment_table[cs as usize - 1];
    let code_bytes = executable.slice(
        code_segment.logical_sector_offset as usize,
        code_segment.length_of_segment_in_file as usize,
    )?;
    let data_bytes = executable.slice(
        data_segment.logical_sector_offset as usize,
        data_segment.length_of_segment_in_file as usize,
    )?;
    println!("{:?}", data_bytes);
    memory
        .copy_from(code_bytes, 0x4000)
        .map_err(|_| ExecutableFormatError::HeaderSize)?; // TODO: code offset & segment
    memory
        .copy_from(data_bytes, 0x123*0x10)
        .map_err(|_| ExecutableFormatError::HeaderSize)?; // TODO: data offset & segment
    let emulated_kernel = EmulatedKernel::new();
    let emulated_user = EmulatedUser::new();
    perform_relocations(&mut memory, 0x4000, &module_reference_table, &entry_table, code_segment)
        .map_err(|_| ExecutableFormatError::HeaderSize)?; // TODO: also other relocations necessary

    // TODO: don't do this here, I'm just testing stuff. Also don't hardcode this!
    let mut emulator = Emulator::new(memory, 0x123, 0, ip + 0x4000, emulated_kernel, emulated_user);
    emulator.run();

    // TODO: validate CRC32
    Ok(())
}

fn process_file(executable: &mut Executable) -> Result<(), ExecutableFormatError> {
    let mz_result = process_file_mz(executable)?;
    process_file_ne(executable, mz_result.ne_header_offset)
}

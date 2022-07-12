use crate::emulator::Emulator;
use crate::executable::{Executable, ExecutableFormatError};
use crate::util::{bool_to_result, u16_from_slice};

mod emulator;
mod emulator_error;
mod executable;
mod memory;
mod mod_rm;
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

fn process_file_mz(bytes: &Executable) -> Result<MZResult, ExecutableFormatError> {
    bytes.validate_magic_id(0, b"MZ")?;
    // TODO: check MZ checksum
    let ne_header_offset = bytes.read_u16(0x3C)? as usize;
    Ok(MZResult { ne_header_offset })
}

fn validate_application_flags(bytes: &Executable) -> Result<(), ExecutableFormatError> {
    bool_to_result(
        (bytes.read_u8(0x0D)? & 0b11101000) == 0,
        ExecutableFormatError::ApplicationFlags,
    )
}

fn validate_target_operating_system(bytes: &Executable) -> Result<(), ExecutableFormatError> {
    let byte = bytes.read_u8(0x36)?;
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
enum RelocationType {
    ImportOrdinal(ImportOrdinalRelocation),
}

#[derive(Debug)]
struct Relocation {
    relocation_type: RelocationType,
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
    bytes: &mut Executable,
    offset_to_segment_table: usize,
    segment_count: usize,
    file_alignment_size_shift: usize,
) -> Result<SegmentTable, ExecutableFormatError> {
    let segment_table_cursor = bytes.seek_from_here(offset_to_segment_table)?;

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
            (bytes.read_u16(byte_offset)? as u32) << file_alignment_size_shift;
        let length_of_segment_in_file = map_zero_to_64k(bytes.read_u16(byte_offset + 2)?);
        let flags = bytes.read_u16(byte_offset + 4)? as u32;

        // Read relocation data
        let relocations = if (flags & 0x100) == 0x100 {
            let relocation_old_cursor = bytes.seek_from_start(
                logical_sector_offset as usize + length_of_segment_in_file as usize,
            )?;
            let relocation_count = bytes.read_u16(0)?;

            let mut relocations = Vec::with_capacity(relocation_count as usize);

            for relocation_index in 0..relocation_count {
                println!("relocation index {}", relocation_index);

                let byte_offset = 2 + relocation_index as usize * 8;

                let source_type = bytes.read_u8(byte_offset)?;
                let flags = bytes.read_u8(byte_offset + 1)?;
                let offset_within_segment_from_source_chain = bytes.read_u16(byte_offset + 2)?;

                let old_cursor = bytes.seek_from_start(logical_sector_offset as usize)?;

                // Walk the linked list of the offsets
                let mut offset_cursor = offset_within_segment_from_source_chain;
                // TODO: avoid loops in the linked list
                loop {
                    println!("relocate at {}", offset_cursor);
                    let pointer = bytes.read_u16(offset_cursor as usize)?;

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

                bytes.restore_cursor(old_cursor);

                match flags & 3 {
                    // Internal ref
                    0 => {
                        println!("internal ref");
                    }
                    // Import ordinal
                    1 => {
                        let index_into_module_reference_table = bytes.read_u16(byte_offset + 4)?;
                        let procedure_ordinal_number = bytes.read_u16(byte_offset + 6)?;
                        relocations.push(Relocation {
                            relocation_type: RelocationType::ImportOrdinal(
                                ImportOrdinalRelocation {
                                    index_into_module_reference_table,
                                    procedure_ordinal_number,
                                },
                            ),
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

            bytes.restore_cursor(relocation_old_cursor);

            Some(relocations)
        } else {
            None
        };

        segments.push(Segment {
            logical_sector_offset,
            length_of_segment_in_file,
            minimum_allocation_size: map_zero_to_64k(bytes.read_u16(byte_offset + 6)?),
            relocations,
        });
    }

    bytes.restore_cursor(segment_table_cursor);

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

fn process_module_reference_table(bytes: &Executable, offset_to_module_reference_table: usize, module_reference_count: usize) -> Result<(), ExecutableFormatError> {
    let offset_to_imported_name_table = bytes.read_u16(0x2A)? as usize;

    for module_index in 0..module_reference_count {
        let module_name_offset_in_imported_name_table = bytes.read_u16(offset_to_module_reference_table + module_index * 2)?;
        let start_offset = offset_to_imported_name_table + module_name_offset_in_imported_name_table as usize;
        let module_name_length = bytes.read_u8(start_offset)?;
        println!("module {} name: {:?}", module_index, bytes.slice(start_offset + 1, module_name_length as usize)?);
    }

    Ok(())
}

fn process_file_ne(
    bytes: &mut Executable,
    ne_header_offset: usize,
) -> Result<(), ExecutableFormatError> {
    let old_cursor = bytes.seek_from_start(ne_header_offset)?;
    bytes.validate_magic_id(0, b"NE")?;
    validate_application_flags(bytes)?;
    validate_target_operating_system(bytes)?;

    let segment_table_segment_count = bytes.read_u16(0x1C)? as usize;
    let module_reference_count = bytes.read_u16(0x1E)? as usize;
    let offset_to_segment_table = bytes.read_u16(0x22)? as usize;
    let offset_to_module_reference_table = bytes.read_u16(0x28)? as usize;
    let file_alignment_size_shift = {
        let shift = bytes.read_u16(0x32)?;
        if shift == 0 {
            9
        } else {
            shift as usize
        }
    };

    process_module_reference_table(bytes, offset_to_module_reference_table, module_reference_count)?;

    println!(
        "Expected Windows version: {}.{}",
        bytes.read_u8(0x3F)?,
        bytes.read_u8(0x3E)?
    );

    let cs = bytes.read_u16(0x16)?;
    let ip = bytes.read_u16(0x14)?;
    let ss = bytes.read_u16(0x1A)?;
    let sp = bytes.read_u16(0x18)?;

    println!("CS:IP data: {:x} {:x}", cs, ip);
    println!("SS:SP data: {:x} {:x}", ss, sp);

    let segment_table = process_segment_table(
        bytes,
        offset_to_segment_table,
        segment_table_segment_count,
        file_alignment_size_shift,
    )?;
    println!("{:#?}", segment_table);

    validate_segment_index_and_offset(&segment_table, cs, ip)?;
    validate_segment_index_and_offset(&segment_table, ss, sp)?;

    bytes.restore_cursor(old_cursor);

    // TODO: don't do this here, I'm just testing stuff. Also don't hardcode this!
    let code_segment = &segment_table[cs as usize - 1]; // TODO: handle 0 segment
    let code_bytes = bytes.slice(
        code_segment.logical_sector_offset as usize,
        code_segment.length_of_segment_in_file as usize,
    )?;
    let mut emulator = Emulator::new(code_bytes, ip);
    emulator.run();

    // TODO: validate CRC32
    Ok(())
}

// TODO: rename "bytes"
fn process_file(bytes: &mut Executable) -> Result<(), ExecutableFormatError> {
    let mz_result = process_file_mz(bytes)?;
    process_file_ne(bytes, mz_result.ne_header_offset)
}

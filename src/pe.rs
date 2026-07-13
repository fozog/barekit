/*
    SPDX-License-Identifier: Mozilla Public License 2.0
*/

#[repr(C)]
pub struct DosHeader {
    pub e_magic: u16,
    pub e_cblp: u16,
    pub e_cp: u16,
    pub e_crlc: u16,
    pub e_cparhdr: u16,
    pub e_minalloc: u16,
    pub e_maxalloc: u16,
    pub e_ss: u16,
    pub e_sp: u16,
    pub e_csum: u16,
    pub e_ip: u16,
    pub e_cs: u16,
    pub e_lfarlc: u16,
    pub e_ovno: u16,
    pub e_res: [u16; 4],
    pub e_oemid: u16,
    pub e_oeminfo: u16,
    pub e_res2: [u16; 10],
    pub e_lfanew: i32,
}

#[repr(C)]
pub struct FileHeader {
    pub machine: u16,
    pub num_of_sections: u16,
    pub time_date_stamp: u32,
    pub ptr_to_symbol_table: u32,
    pub num_of_symbols: u32,
    pub size_of_optional_header: u16,
    pub characteristics: u16,
}

#[repr(C)]
pub struct DataDirectory {
    pub virtual_address: u32,
    pub size: u32,
}

#[repr(C)]
pub struct OptionalHeader64 {
    pub magic: u16,
    pub major_linker_version: u8,
    pub minor_linker_version: u8,
    pub size_of_code: u32,
    pub size_of_initialized_data: u32,
    pub size_of_uninitialized_data: u32,
    pub address_of_entry_point: u32,
    pub base_of_code: u32,
    pub image_base: u64,
    pub section_alignment: u32,
    pub file_alignment: u32,
    pub major_operating_system_version: u16,
    pub minor_operating_system_version: u16,
    pub major_image_version: u16,
    pub minor_image_version: u16,
    pub major_subsystem_version: u16,
    pub minor_subsystem_version: u16,
    pub win32_version_value: u32,
    pub size_of_image: u32,
    pub size_of_headers: u32,
    pub check_sum: u32,
    pub subsystem: u16,
    pub dll_characteristics: u16,
    pub size_of_stack_reserve: u64,
    pub size_of_stack_commit: u64,
    pub size_of_heap_reserve: u64,
    pub size_of_heap_commit: u64,
    pub loader_flags: u32,
    pub number_of_rva_and_sizes: u32,
    pub data_directory: [DataDirectory; 16],
}

#[repr(C)]
pub struct NtHeader {
    pub signature: u32,
    pub file_header: FileHeader,
    pub optional_header: OptionalHeader64,
}

#[repr(C)]
pub struct SectionHeader {
    pub name: [u8; 8],
    pub virtual_size: u32,
    pub virtual_address: u32,
    pub raw_data_size: u32,
    pub raw_data_address: u32,
    pub ptr_to_relocations: u32,
    pub ptr_to_line_numbers: u32,
    pub num_of_relocations: u16,
    pub num_of_line_numbers: u16,
    pub characteristics: u32,
}

#[allow(dead_code)]
pub enum SectionFlags {
    CntInitData = 0x0000_0040,
    Write = 0x8000_0000,
    Read = 0x4000_0000,
    Discardable = 0x0200_0000,
}

#[repr(C)]
pub struct RelocationHead {
    pub page_rva: u32,
    pub block_size: u32,
}
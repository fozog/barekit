/* 
    SPDX-License-Identifier: Mozilla Public License 2.0
    Copyrigth (c) 2022-2023 François-Frédéric Ozog
    
*/

use peview::dir::{RelocationHead};
use peview::header::{DosHeader, NtHeader, SectionHeader, FileHeader, SectionFlags};


/*
CRITICAL INFORMATION ON PE/COFF to understand memory layout and information..

First lets look at concrete examples on section information from both sources:
/map file and objdump

1) start.rs:TTY_BUFFER set to 16384

1.a) Information from /map:barekit.map
 Start         Length     Name                   Class
 0001:00000000 00018124H .text                   CODE
 0001:00018124 0000033cH .text$unlikely          CODE
 0002:00000000 00004b90H .rdata                  DATA
 0003:00000000 00000018H .data                   DATA
 0003:00000018 00004018H .bss                    DATA
 0005:00000000 0000b0a4H .debug_abbrev           DATA
 0006:00000000 00008110H .debug_aranges          DATA
 0007:00000000 00076f8fH .debug_info             DATA
 0008:00000000 00026cccH .debug_line             DATA
 0009:00000000 000021bbH .debug_loc              DATA
 000a:00000000 00023d83H .debug_pubnames         DATA
 000b:00000000 000280f1H .debug_pubtypes         DATA
 000c:00000000 0000ccc0H .debug_ranges           DATA
 000d:00000000 000844fdH .debug_str              DATA

 1.b) objdump sections:
 Sections:
Idx Name            Size     VMA              Type
  0 .text           00018460 0000000000001000 TEXT
  1 .rdata          00004c1a 000000000001a000 DATA
  2 .data           00001000 000000000001f000 DATA
  3 .reloc          00000314 0000000000024000 DATA
  4 .debug_abbrev   0000b0a4 0000000000025000 DATA, DEBUG
  5 .debug_aranges  00008110 0000000000031000 DATA, DEBUG
  6 .debug_info     00076f8f 000000000003a000 DATA, DEBUG
  7 .debug_line     00026ccc 00000000000b1000 DATA, DEBUG
  8 .debug_loc      000021bb 00000000000d8000 DATA, DEBUG
  9 .debug_pubnames 00023d83 00000000000db000 DATA, DEBUG
 10 .debug_pubtypes 000280f1 00000000000ff000 DATA, DEBUG
 11 .debug_ranges   0000ccc0 0000000000128000 DATA, DEBUG
 12 .debug_str      000844fd 0000000000135000 DATA, DEBUG

 2) start.rs:TTY_BUFFER set to 256

 2.a) Information from /map:barekit.map
  Start         Length     Name                   Class
 0001:00000000 00018124H .text                   CODE
 0001:00018124 0000033cH .text$unlikely          CODE
 0002:00000000 00004b90H .rdata                  DATA
 0003:00000000 00000018H .data                   DATA
 0003:00000018 00000118H .bss                    DATA
 0005:00000000 0000b0a4H .debug_abbrev           DATA
 0006:00000000 00008110H .debug_aranges          DATA
 0007:00000000 00076f8fH .debug_info             DATA
 0008:00000000 00026cdbH .debug_line             DATA
 0009:00000000 000021bbH .debug_loc              DATA
 000a:00000000 00023d83H .debug_pubnames         DATA
 000b:00000000 000280f1H .debug_pubtypes         DATA
 000c:00000000 0000ccc0H .debug_ranges           DATA
 000d:00000000 000844fdH .debug_str              DATA

  2.b) objdump sections:
Sections:
Idx Name            Size     VMA              Type
  0 .text           00018460 0000000000001000 TEXT
  1 .rdata          00004c1a 000000000001a000 DATA
  2 .data           00000130 000000000001f000 DATA
  3 .reloc          00000314 0000000000020000 DATA
  4 .debug_abbrev   0000b0a4 0000000000021000 DATA, DEBUG
  5 .debug_aranges  00008110 000000000002d000 DATA, DEBUG
  6 .debug_info     00076f8f 0000000000036000 DATA, DEBUG
  7 .debug_line     00026cdb 00000000000ad000 DATA, DEBUG
  8 .debug_loc      000021bb 00000000000d4000 DATA, DEBUG
  9 .debug_pubnames 00023d83 00000000000d7000 DATA, DEBUG
 10 .debug_pubtypes 000280f1 00000000000fb000 DATA, DEBUG
 11 .debug_ranges   0000ccc0 0000000000124000 DATA, DEBUG
 12 .debug_str      000844fd 0000000000131000 DATA, DEBUG

 3) analysis of the above two cases

 Looking at the /map excerpt (at the end of the file):
 0003:00000000       _ZN7barekit4heap4HEAP17hb64a84bab3c288d4E 000000000001f000     barekit-466377300eb9f42e.4cai9g4yawjrhmu8.rcgu.o
 0003:00000018       _ZN7barekit5start10TTY_BUFFER17he39844c892c5ddfeE 000000000001f018     barekit-466377300eb9f42e.13zjuzn704wqzv9s.rcgu.o
 0003:00000118       _ZN7barekit3log10TTY_TARGET17h803239ee42015579E 000000000001f118     barekit-466377300eb9f42e.1hirjepxl0gm1v02.rcgu.o
 0003:00000128       _ZN7barekit4heap11ALLOC_COUNT17h43aa2a4b76d461f1E 000000000001f128     barekit-466377300eb9f42e.4cai9g4yawjrhmu8.rcgu.o

 we see that .data section is actually built of all "static"
 variables, initialized or not.
 HEAP (HEAP::size_of() == 0x18) is initialized
 .bss "pseudo" section (pseudo as it is not listed per say in the PE/COFF file) 
 starts right afterward with TTY_BUFFER, TTY_TARGET, ALLOC_COUNT.

 The size of the .data maxes at 0x1000 (section alignment) in the file, yet the .reloc
 section starts away enough to accomodate the pseudo .bss size and the area past the .data 
 file part need to be zeroed to match the specification.


 The optional headers' fields for both cases are not reflecting the expected values to discover .BSS at runtime:
mSizeOfInitializedData: 0x19c000
mSizeOfUninitializedData: 0 


*/


pub unsafe fn relocate(load_address: usize, _upper_limit: usize) {
    
    //TODO: make sure we don't go beyond limits, in particular beyond the image itself

    // 1) find the sections table

    let dos_header: &DosHeader = &*(load_address as * const DosHeader);

    //early_print!("PE header offset = {:#04x}\n", dos_header.e_lfanew);
    let nt_header: &NtHeader = &*((load_address + (dos_header.e_lfanew as usize)) as * const NtHeader);
    let mut start_of_sections = load_address + dos_header.e_lfanew as usize;
    start_of_sections += core::mem::size_of::<u32>() as usize; // signature
    start_of_sections += core::mem::size_of::<FileHeader>() as usize;
    start_of_sections += nt_header.file_header.size_of_optional_header as usize;

    //early_prints!("First section header offset = %\n", start_of_sections as u64);
    
    // 2) look for the .reloc section
    let mut reloc_section_offset:  u32 = 0;
    let mut reloc_section_size: u32 = 0;
    //println!("Checking {} sections", efi_header.file_header.num_of_sections );
    let reloc_name: [u8;8] = [46, 114, 101, 108, 111, 99, 0, 0];

    // browse sections from the end to allow moving of the sections without overlapping
    for s in (0..nt_header.file_header.num_of_sections).rev() { 
        let current_section = start_of_sections + (s as usize) * core::mem::size_of::<SectionHeader>();
        let section : &SectionHeader = &*(current_section as * const SectionHeader);
        let mut move_section = false;
        if  section.characteristics & SectionFlags::Discardable as u32 == 0 {
            /*
            early_prints!("Section %\n", section.characteristics as u64);
            early_prints!("    VM start %\n", section.virtual_address as u64);
            early_prints!("    VM size %\n", section.virtual_size as u64);
            early_prints!("    File start %\n", section.raw_data_address as u64);
            early_prints!("    File size %\n", section.raw_data_size as u64);
            */
            if section.raw_data_address != 0 && section.raw_data_address != section.virtual_address {
                move_section = true;
            }
        }
        else {
            if section.name == reloc_name {
                /*
                early_prints!("Relocation %\n", section.characteristics as u64);
                early_prints!("    VM start %\n", section.virtual_address as u64);
                early_prints!("    VM size %\n", section.virtual_size as u64);
                early_prints!("    File start %\n", section.raw_data_address as u64);
                early_prints!("    File size %\n", section.raw_data_size as u64);
                */
                if section.virtual_address != section.raw_data_address {
                    move_section = true;
                }
                reloc_section_offset =  section.virtual_address;
                reloc_section_size =  section.virtual_size;
            }
        }
        
        if move_section {
            //early_prints!("Moving section to %\n", load_address as u64 + section.virtual_address as u64);
            for i in 0..section.raw_data_size {
                let src =  (load_address as u64 + section.raw_data_address as u64 + i as u64) as *const u8;
                let dst = (load_address as u64 + section.virtual_address as u64 + i as u64) as *mut u8;
                *dst = *src;
            }
        }

        let mask = SectionFlags::Write as u32 | SectionFlags::Read as u32 | SectionFlags::CntInitData as u32;
        if section.characteristics == mask {
            // this is the .data section
            // lets zero the .BSS part
            //early_prints!("Zeroing BSS start at %\n", load_address as u64 + section.virtual_address as u64 + section.raw_data_size as u64);
            for i in section.raw_data_size..section.virtual_size {
                let dst = (load_address as u64 + section.virtual_address as u64 + i as u64) as *mut u8;
                *dst = 0;
            }

        } 
    }

    // 3) handle relocations if present
    if reloc_section_offset != 0  {

        let mut block_address = load_address + (reloc_section_offset as usize);
        let block_end = block_address + reloc_section_size as usize;

        //early_prints!("handling relocations at offset = %\n", block_address as u64);
        //early_prints!("relocations size = %\n", reloc_section_size as u64);
        // loop through relocation blocks: essentially one block per page that contain relocations
        
        let image_base = nt_header.optional_header.image_base as u64;

        while block_address < block_end {

            let block : &RelocationHead = &*(block_address as * const RelocationHead);
            
            let page_rva =  block.page_rva as usize;
            let reloc_in_block: u32 = (block.block_size - (core::mem::size_of::<RelocationHead>() as u32))/ (core::mem::size_of::<u16>() as u32);

            //early_prints!("Relocating for page %\n", (load_address +block.page_rva as usize) as u64);
            //early_prints!("     relocations blocksize = %\n", block.block_size as u64);

            //let mut reloc_tag_offset : usize = load_address + (reloc_section_offset as usize) + core::mem::size_of::<RelocationHead>();
            let mut reloc_tag_offset : usize = block_address +core::mem::size_of::<u32>()*2
             ;
            // do the relocation in the block
            for _r in 0..reloc_in_block {
                
                let relocation_tag : u16 =  *(reloc_tag_offset as * const u16);
                let offset = (relocation_tag & 0xFFF) as usize;
                let relocation_type = relocation_tag >> 12 ;
                
                let target_address = load_address + page_rva + offset;
                //early_prints!("    target=%: ", target_address as u64);
                let target = target_address as *mut u64;
                if relocation_type == 10 {
                    //early_prints!("Dir64 *target=%\n", *target);
                    let newvalue =  *target+(load_address as u64) - image_base;
                    *target = newvalue;
                        
                    //println!("Apply relocation in {:#x}: {:#x} -> {:#x}", target_address, *target, newvalue);
                }
                else if relocation_type == 11 {
                    let newvalue =  *target+(load_address as u64) - image_base;
                    //early_prints!("!!! *target=%\n", *target);
                    *target = newvalue;
                }
                else if relocation_type == 0 {
                    //early_prints!("PAD\n", 0);
                }
                else {
                    //early_prints!("? type=%\n", relocation_type as u64);
                }
                // next reloc in block
                reloc_tag_offset += core::mem::size_of::<u16>();
            }

            block_address += block.block_size as usize;
        }
        
    }
}


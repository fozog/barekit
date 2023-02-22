/* 
    SPDX-License-Identifier: Mozilla Public License 2.0
    Copyrigth (c) 2022-2023 François-Frédéric Ozog
    
*/

use core::arch::asm;


/*
  304 |        |        |
      +--------+--------+
  288 | old_fp | return | ; stack frame (old_fp == x29, return = elr_el1)
      +--------+--------+
  272 | esr_el1|        |
  256 | elr_el1|spsr_el1|
      +--------+--------+
  240 |   x30  | old_sp |
      +--------+--------+
  224 |   x28  |   x29  |
  208 |   x26  |   x27  |
  192 |   x24  |   x25  |
  176 |   x22  |   x23  |
  160 |   x20  |   x21  |
  144 |   x18  |   x19  |
  128 |   x16  |   x17  |
  112 |   x14  |   x15  |
   96 |   x12  |   x13  |
   80 |   x10  |   x11  |
   64 |   x8   |   x9   |
   48 |   x6   |   x7   |
   32 |   x4   |   x5   |
   16 |   x2   |   x3   |
    0 |   x0   |   x1   |
      +--------+--------+
 */


 #[repr(C)]
 pub struct GPRegisters {
    pub x : [u64; 31],
    pub sp : u64
}

#[repr(C)]
pub struct StackFrame {
    pub old_fp : u64,
    pub return_address : u64
}


#[repr(C)]
pub struct ExceptionFrame {
    pub gp_regs : GPRegisters,
    pub elr: u64,
    pub spsr : u64,
    pub esr : u64,
    pub padding: u64,
    pub stack_frame: StackFrame
}

pub fn get_current_el() -> u8 
{
    let mut current_el : u64;
    unsafe {
        asm!("mrs {}, CurrentEL", out(reg) current_el);
    }
    current_el = (current_el >> 2 ) & 3;
    current_el as u8
}

pub fn get_vbar() -> u64 {
    let current_el = get_current_el();
    let value : u64;
    unsafe {
        match  current_el {
            1 => asm!("mrs {}, VBAR_EL1", out(reg) value),
            2 => asm!("mrs {}, VBAR_EL2", out(reg) value),
            3 => asm!("mrs {}, VBAR_EL3", out(reg) value),
            0 | 4..=u8::MAX => panic!("Invalid EL retrieved: {:#x}", current_el)
        }
    };
    return value;
}


pub fn get_elr() -> u64 {
    let current_el = get_current_el();
    let value : u64;
    unsafe {
        match  current_el {
            1 => asm!("mrs {}, ELR_EL1", out(reg) value),
            2 => asm!("mrs {}, ELR_EL2", out(reg) value),
            3 => asm!("mrs {}, ELR_EL3", out(reg) value),
            0 | 4..=u8::MAX => panic!("Invalid EL retrieved: {:#x}", current_el)
        }
    };
    return value;
}

pub fn get_tcr() -> u64 {
    let current_el = get_current_el();
    let value : u64;
    unsafe {
        match  current_el {
            1 => asm!("mrs {}, TCR_EL1", out(reg) value),
            2 => asm!("mrs {}, TCR_EL2", out(reg) value),
            3 => asm!("mrs {}, TCR_EL3", out(reg) value),
            0 | 4..=u8::MAX => panic!("Invalid EL retrieved: {:#x}", current_el)
        }
    };
    return value;
}

pub fn set_vbar(vbar : u64) {
    let current_el = get_current_el();
    unsafe {
        match  current_el {
            1 => asm!("msr VBAR_EL1, {}", in(reg) vbar),
            2 => asm!("msr VBAR_EL2, {}", in(reg) vbar),
            3 => asm!("msr VBAR_EL3, {}", in(reg) vbar),
            0 | 4..=u8::MAX => panic!("Invalid EL retrieved: {:#x}", current_el)
        }
    };
}





#[allow(dead_code)]
pub const TRANSLATION_5_LEVELS:u8  =  0;
#[allow(dead_code)]
pub const TRANSLATION_4_LEVELS:u8  =  1;
#[allow(dead_code)]
pub const TRANSLATION_3_LEVELS:u8  =  2;
#[allow(dead_code)]
pub const TRANSLATION_2_LEVELS:u8  =  3;
#[allow(dead_code)]
pub const TRANSLATION_1_LEVEL:u8   =  4;


pub fn page_index_at_bits(va: u64, bits: u8) -> usize
{
    ((va as usize) >> bits) & 0x1FF
}

pub fn page_is_present(pageinfo: u64) -> bool
{
    pageinfo & 1 != 0
}

pub fn page_is_block(pageinfo: u64, bits: u8) -> bool
{
    bits > 12 && pageinfo & 2 == 0
}

pub fn page_is_table(pageinfo: u64, bits: u8) -> bool
{
    bits > 12 && pageinfo & 2 == 2
}

pub fn page_target_at_index(pageinfo: u64, index: usize) -> *const u64
{
    // this is either a page or a block
    let mut mask: u64 = 0xFFF;
    let bits: u8 = BITS_AT_LEVEL[index];
    if bits > 12 {
        mask |= (1 << BITS_AT_LEVEL[index + 1]) -1;
    }
    mask |= 0xFFFF << 48;
    (pageinfo & !mask) as *const u64
}

pub fn table_target_at_index(pageinfo: u64, _index: usize) -> *const u64
{
    // this is either a page or a block
    let mut mask: u64 = 0xFFF;
    mask |= 0xFFFF << 48;
    (pageinfo & !mask) as *const u64
}

pub static BITS_AT_LEVEL : [u8; 5] = [
    /* index 0 */    9+9+9+9+12,        /* level -1 as per Arm doc */
    /* index 1 */    9+9+9+12,          /* level 0  as per Arm doc */
    /* index 2 */    9+9+12,            /* level 1  as per Arm doc */
    /* index 3 */    9+12,              /* level 2  as per Arm doc */
    /* index 4 */    12,                /* level 3  as per Arm doc */
    ];

/*
TODO: handle t0sz != t1sz and different ELs
 */
pub fn paging_get_low_mem_paging() -> (u8, usize)
{
    let tcr = get_tcr();
    //TODO: sense t1sz...
    let tsz: u8 = (tcr & 0x3f) as u8;
    let level : u8;
    // let's assume 4K granule (table D5-14)
    match tsz {
        12..16 => level = TRANSLATION_5_LEVELS,
        16..25 => level = TRANSLATION_4_LEVELS,
        25..34 => level = TRANSLATION_3_LEVELS,
        34..43 => level = TRANSLATION_2_LEVELS,
        43..49 => level = TRANSLATION_1_LEVEL,
        0..12 | 49..=u8::MAX => panic!("Invalid T0SZ retrieved {:#x} from TCR= {:#x}", tsz, tcr)
    }
    let size: u64 = 1_u64 << (64 - tsz);
    return (level, size as usize);
}

/*
Returns the page table anchor for a VA
the returned value can be u64::MAX when it points to unaddressable space
return value 0 does not mean anything special: it may be the case that TTBRx_ELy is just set to 0
which may be acutally a page table or nothing. I guess it is not advisable to install the page table
at address 0. So a 0 value is practically equivalent to nothing set
 */
pub fn get_anchor_for(va: u64) -> u64
{
    let current_el = get_current_el();
    let value : u64;
    unsafe { 
        match  current_el {
            1 => {
                let tcr_el1 : u64;
                asm!("mrs {}, tcr_el1", out(reg)tcr_el1);
                let t0sz = tcr_el1 & 0x3f;
                let t1sz =(tcr_el1 >> 16) & 0x3f;
                let end_low_mem = 1_u64 << (64 - t0sz);
                let start_high_mem = u64::MAX - (1_u64 << (64 - t1sz)) +1;
                if va < end_low_mem {
                    asm!("mrs {}, TTBR0_EL1", out(reg) value);
                } else if va < start_high_mem {
                    value = u64::MAX;
                } else {
                    asm!("mrs {}, TTBR1_EL1", out(reg) value);
                }
            },
            2 => asm!("mrs {}, TTBR0_EL2", out(reg) value),
            3 => asm!("mrs {}, TTBR0_EL3", out(reg) value),
            0 | 4..=u8::MAX => panic!("Invalid EL retrieved: {:#x}", current_el)
        }
    }
    // the TTBRx_ELy bit 0 can mean something but is not part of the address
    value & !1
}

/*
* retrieves the physical address, containing directory entry 
* (parameters as results) and page size (return value) from a pointer
* only valid for the virtual address range from 0-128TB 
* if the returned value is None, then the translation has not been possible
* or the page is not present
*/
pub fn paging_virtual_info_ex(anchor: u64, start_index: usize, location: u64) -> Option<(u64, *const u64, *const u64)>
{
	let mut pageinfo : u64;
	let mut entry: *const u64;
	let mut index : usize;
	let mut current_index : usize= start_index;

	pageinfo = anchor;

	while current_index < 5
	{
        unsafe {
            let bits = BITS_AT_LEVEL[current_index];
            index = page_index_at_bits(location, bits);
            let table = table_target_at_index(pageinfo, current_index);
            entry = table.add(index);
            //println!("Checking at current_index {}: {:#x}[{}] ({:#x})= {:#x}",current_index, table as u64, index, entry as u64, *entry);
            pageinfo = *entry;
            if page_is_block(pageinfo, bits)|| !page_is_present(pageinfo) || current_index == 4 {
                return Some((1 << bits, page_target_at_index(pageinfo, current_index), entry));
            }
        }
        current_index += 1;
	}

	return None;

}

pub fn paging_virtual_info(location: u64) -> Option<(u64, *const u64, *const u64)>
{
    let anchor = get_anchor_for(location);
    let info = paging_get_low_mem_paging();
    return paging_virtual_info_ex(anchor, info.0 as usize, location);
}

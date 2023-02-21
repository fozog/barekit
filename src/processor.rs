/* 
    SPDX-License-Identifier: Mozilla Public License 2.0
    Copyrigth (c) 2022-2023 François-Frédéric Ozog
    
*/

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

use core::arch::asm;

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

pub fn get_current_vbar() -> u64 {
    let mut current_el : u64;
    let vbar : u64;
    unsafe {
        asm!("mrs {}, CurrentEL", out(reg) current_el);
        current_el = (current_el >> 2 ) & 3;
        match  current_el {
            1 => asm!(
                "mrs {}, VBAR_EL1",
                out(reg) vbar
            ),
            2 => asm!(
                "mrs {}, VBAR_EL2",
                out(reg) vbar
            ),
            3 => asm!(
                "mrs {}, VBAR_EL3",
                out(reg) vbar
            ),
            0_u64 | 4_u64..=u64::MAX => panic!("Invalid EL retrieved: {:#x}", current_el)
        }
    };
    return vbar;
}

pub fn get_current_elr() -> u64 {
    let mut current_el : u64;
    let vbar : u64;
    unsafe {
        asm!("mrs {}, CurrentEL", out(reg) current_el);
        current_el = (current_el >> 2 ) & 3;
        match  current_el {
            1 => asm!(
                "mrs {}, ELR_EL1",
                out(reg) vbar
            ),
            2 => asm!(
                "mrs {}, ELR_EL2",
                out(reg) vbar
            ),
            3 => asm!(
                "mrs {}, ELR_EL3",
                out(reg) vbar
            ),
            0_u64 | 4_u64..=u64::MAX => panic!("Invalid EL retrieved: {:#x}", current_el)
        }
    };
    return vbar;
}

pub fn set_current_vbar(vbar : u64) {
    let mut current_el : u64;
    unsafe {
        asm!("mrs {}, CurrentEL", out(reg) current_el);
        current_el = (current_el >> 2 ) & 3;
        match  current_el {
            1 => asm!(
                "msr VBAR_EL1, {}",
                in(reg) vbar
            ),
            2 => asm!(
                "msr VBAR_EL2, {}",
                in(reg) vbar
            ),
            3 => asm!(
                "msr VBAR_EL3, {}",
                in(reg) vbar
            ),
            0_u64 | 4_u64..=u64::MAX => panic!("Invalid EL retrieved: {:#x}", current_el)
        }
    };
}
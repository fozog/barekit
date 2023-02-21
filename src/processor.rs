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
    pub elr_el1: u64,
    pub spsr_el1 : u64,
    pub esr_el1 : u64,
    pub padding: u64,
    pub stack_frame: StackFrame
}

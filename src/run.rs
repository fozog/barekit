/* 
    SPDX-License-Identifier: Mozilla Public License 2.0
    Copyrigth (c) 2022-2023 François-Frédéric Ozog
    
*/

use alloc::boxed::Box;
use core::arch::asm;
use core::arch::global_asm;
use alloc::vec::Vec;

use crate::PlatformOperations;

use crate::println;

use crate::early_prints;
#[cfg(feature = "early_print")]
use crate::print::_early_print_s;

/*
  320 |        |        |
      +--------+--------+
  304 | old_fp | return | ; stack frame (old_fp == x29, return = elr_el1)
      +--------+--------+
  288 |        |        |
  272 |        |        |
      +--------+--------+
  256 | esr_el1|spsr_el1|
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
global_asm!("
exception_table:
    sub     sp, sp, #0x150
    stp     x0, x1, [sp]
    isb
    mov     w0, #0x9000000
    mov     w1, #0x21
    strb    w1, [x0]
    dsb sy
    isb
    mov     x1, #0
    b       trampoline
    
. = exception_table + 0x200
    sub     sp, sp, #0x150
    stp     x0, x1, [sp]
    isb
    mov     w0, #0x9000000
    mov     w1, #0x22
    //strb    w1, [x0]
    dsb sy
    isb
    mov     x1, #1
    b       trampoline


. = exception_table + 0x800

reloc_offset:
    .quad   0

trampoline:
    stp     x2, x3, [sp, #16]
    stp     x4, x5, [sp, #32]
    stp     x6, x7, [sp, #48]
    stp     x8, x9, [sp, #64]
    stp     x10, x11, [sp, #80]
    stp     x12, x13, [sp, #96]
    stp     x14, x15, [sp, #112]
    stp     x16, x17, [sp, #128]
    stp     x18, x19, [sp, #144]
    stp     x20, x21, [sp, #160]
    stp     x22, x23, [sp, #176]
    stp     x24, x25, [sp, #192]
    stp     x26, x27, [sp, #208]
    stp     x28, x29, [sp, #224]

    # x21 = old_sp
    add     x21, sp, #0x150
    stp     x30, x21, [sp, #240]
    
    # preserve flags and return address
    mrs     x22, elr_el1
    mrs     x23, spsr_el1
    stp     x22, x23, [sp, #256]

    # make a new stack frame for backtrace to work in the future
    stp     x29, x22, [sp, #304]                                                                                               
    add     x29, sp, #304

    mov     w0, #0x9000000
    add     x1, x1, #0x30
    strb    w1, [x0]
    dsb sy
    isb

    adr     x8, handle_exception
    adr     x3, reloc_offset
    ldr     x3, [x3]
    sub     x8, x8, x3

    br       x8

    msr     daifset, #0xf

    #restore returning environment
    ldp     x21, x22, [sp, #256]
    # override return address with value returned by handle_exception
    msr     elr_el1, x0
    msr     spsr_el1, x23

    ldp     x0, x1, [sp]
    ldp     x2, x3, [sp, #16]
    ldp     x4, x5, [sp, #32]
    ldp     x6, x7, [sp, #48]
    ldp     x8, x9, [sp, #64]
    ldp     x10, x11, [sp, #80]
    ldp     x12, x13, [sp, #96]
    ldp     x14, x15, [sp, #112]
    ldp     x16, x17, [sp, #128]
    ldp     x18, x19, [sp, #144]
    ldp     x20, x21, [sp, #160]  
    ldp     x22, x23, [sp, #176]  
    ldp     x24, x25, [sp, #192]  
    ldp     x26, x27, [sp, #208]  
    ldp     x28, x29, [sp, #224]  
    ldr     x30, [sp, #240]

    add     sp, sp, #0x150

    eret

test:
adr     x2, handle_exception
adr     x3, reloc_offset
ldr     x3, [x3]
sub     x0, x2, x3
ret

");

extern "C" {
    fn exception_table() -> !;
    fn reloc_offset() -> !;
    fn test() -> u64;
}

static mut previous_vbar: u64 = 0;

#[export_name = "handle_exception"]
extern "C" fn handle_exception() -> u64 {
    let mut elr_el1 : u64;
    unsafe {
        asm!("msr VBAR_EL1, {}", in(reg) previous_vbar);
        asm!("mrs {}, ELR_EL1", out(reg) elr_el1);
    }
    return elr_el1;
}

#[repr(align(0x1000))]
struct AlignedBuffer {
    buffer : [u8; 0xa00]
}

static mut  RW_EXCEPTION_TABLE : AlignedBuffer = AlignedBuffer{buffer : [0; 0xa00]};

pub fn run(platform:&Box<dyn PlatformOperations>) -> i64 {
    println!("ID registers at startup\n");
    
    let mut barekit_vbar : u64;

    unsafe {
        asm!("mrs {}, VBAR_EL1", out(reg) previous_vbar);
        barekit_vbar = exception_table as u64;
    }


    unsafe {

        // kvmtool sets VBAR to a special value 
        //TODO: kvmtool to set it to (cached value)
        if previous_vbar != 0 && previous_vbar != 0xf0000000 {
            
            let offset = reloc_offset as *mut u64;
            *offset = RW_EXCEPTION_TABLE.buffer.as_ptr() as u64 - barekit_vbar;
            println!("reloc_offset set to {:#x}", *offset);

                let mut source = previous_vbar as *const u64;
            let mut target =  RW_EXCEPTION_TABLE.buffer.as_mut_ptr() as *mut u64;
            println!("Copy barekit handler from {:#x} to {:#x}", source as u64, target as u64);
            let mut i:u32 = 0;
            while i < (0x800 / 8)
            {
                *target = *source;
                target = target.add(1);
                source = source.add(1);
                i+=1;
            }
            // now point source to trampoline code
            source = exception_table as *const u64;
            source = source.add(0x800/8);
            i = 0;
            while i < 0x200 / 8
            {
                *target = *source;
                target = target.add(1);
                source = source.add(1);
                i+=1;
            }

            // now copy the barekit syncchronous handlers
            source = exception_table as *const u64;
            target =  RW_EXCEPTION_TABLE.buffer.as_mut_ptr() as *mut u64;

            i=0;
            while i < 0x80 / 8
            {
                *target = *source;
                target = target.add(1);
                source = source.add(1);
                i+=1;
            }
            target = target.add(0x200 /8);
            source = source.add(0x200 /8);
            i=0;
            while i < 0x80 / 8
            {
                *target = *source;
                target = target.add(1);
                source = source.add(1);
                i+=1;
            }

            barekit_vbar = RW_EXCEPTION_TABLE.buffer.as_mut_ptr() as u64;
        
        }

        println!("Setting VBAR_EL1 to {:#x}", barekit_vbar);
        asm!("msr VBAR_EL1, {}", in(reg) barekit_vbar);

        println!("test={:#x}", test());
        println!("handle_exception={:#x}", handle_exception as u64);

    }


    unsafe {
            let value: u64;
            asm!("mrs {}, CCSIDR_EL1", out(reg) value);
            println!("CCSIDR_EL1={:#x}", value);
        }

    unsafe {
            let value: u64;
            asm!("mrs {}, CLIDR_EL1", out(reg) value);
            println!("CLIDR_EL1={:#x}", value);
        }

    unsafe {
            let value: u64;
            asm!("mrs {}, CTR_EL0", out(reg) value);
            println!("CTR_EL0={:#x}", value);
        }

    unsafe {
            let value: u64;
            asm!("mrs {}, ID_AA64AFR0_EL1", out(reg) value);
            println!("ID_AA64AFR0_EL1={:#x}", value);
        }

    unsafe {
            let value: u64;
            asm!("mrs {}, ID_AA64AFR1_EL1", out(reg) value);
            println!("ID_AA64AFR1_EL1={:#x}", value);
        }

    unsafe {
            let value: u64;
            asm!("mrs {}, ID_AA64DFR0_EL1", out(reg) value);
            println!("ID_AA64DFR0_EL1={:#x}", value);
        }

    unsafe {
            let value: u64;
            asm!("mrs {}, ID_AA64DFR1_EL1", out(reg) value);
            println!("ID_AA64DFR1_EL1={:#x}", value);
        }

    unsafe {
            let value: u64;
            asm!("mrs {}, ID_AA64ISAR0_EL1", out(reg) value);
            println!("ID_AA64ISAR0_EL1={:#x}", value);
        }

    unsafe {
            let value: u64;
            asm!("mrs {}, ID_AA64ISAR1_EL1", out(reg) value);
            println!("ID_AA64ISAR1_EL1={:#x}", value);
        }

    unsafe {
            let value: u64;
            asm!("mrs {}, ID_AA64ISAR2_EL1", out(reg) value);
            println!("ID_AA64ISAR2_EL1={:#x}", value);
        }

    unsafe {
            let value: u64;
            asm!("mrs {}, ID_AA64MMFR0_EL1", out(reg) value);
            println!("ID_AA64MMFR0_EL1={:#x}", value);
        }

    unsafe {
            let value: u64;
            asm!("mrs {}, ID_AA64MMFR1_EL1", out(reg) value);
            println!("ID_AA64MMFR1_EL1={:#x}", value);
        }

    unsafe {
            let value: u64;
            asm!("mrs {}, ID_AA64MMFR2_EL1", out(reg) value);
            println!("ID_AA64MMFR2_EL1={:#x}", value);
        }


    unsafe {
            let value: u64;
            asm!("mrs {}, ID_AA64PFR0_EL1", out(reg) value);
            println!("ID_AA64PFR0_EL1={:#x}", value);
        }

    unsafe {
            let value: u64;
            asm!("mrs {}, ID_AA64PFR1_EL1", out(reg) value);
            println!("ID_AA64PFR1_EL1={:#x}", value);
        }

    unsafe {
            let value: u64;
            asm!("mrs {}, ID_AFR0_EL1", out(reg) value);
            println!("ID_AFR0_EL1={:#x}", value);
        }

    unsafe {
            let value: u64;
            asm!("mrs {}, ID_AFR0_EL1", out(reg) value);
            println!("ID_AFR0_EL1={:#x}", value);
        }

    unsafe {
            let value: u64;
            asm!("mrs {}, ID_DFR0_EL1", out(reg) value);
            println!("ID_DFR0_EL1={:#x}", value);
        }

    unsafe {
            let value: u64;
            asm!("mrs {}, ID_ISAR0_EL1", out(reg) value);
            println!("ID_ISAR0_EL1={:#x}", value);
        }

    unsafe {
            let value: u64;
            asm!("mrs {}, ID_ISAR1_EL1", out(reg) value);
            println!("ID_ISAR1_EL1={:#x}", value);
        }

    unsafe {
            let value: u64;
            asm!("mrs {}, ID_ISAR2_EL1", out(reg) value);
            println!("ID_ISAR2_EL1={:#x}", value);
        }

    unsafe {
            let value: u64;
            asm!("mrs {}, ID_ISAR3_EL1", out(reg) value);
            println!("ID_ISAR3_EL1={:#x}", value);
        }

    unsafe {
            let value: u64;
            asm!("mrs {}, ID_ISAR4_EL1", out(reg) value);
            println!("ID_ISAR4_EL1={:#x}", value);
        }

    unsafe {
            let value: u64;
            asm!("mrs {}, ID_ISAR5_EL1", out(reg) value);
            println!("ID_ISAR5_EL1={:#x}", value);
        }


    unsafe {
            let value: u64;
            asm!("mrs {}, ID_MMFR0_EL1", out(reg) value);
            println!("ID_MMFR0_EL1={:#x}", value);
        }

    unsafe {
            let value: u64;
            asm!("mrs {}, ID_MMFR1_EL1", out(reg) value);
            println!("ID_MMFR1_EL1={:#x}", value);
        }

    unsafe {
            let value: u64;
            asm!("mrs {}, ID_MMFR2_EL1", out(reg) value);
            println!("ID_MMFR2_EL1={:#x}", value);
        }

    unsafe {
            let value: u64;
            asm!("mrs {}, ID_MMFR3_EL1", out(reg) value);
            println!("ID_MMFR3_EL1={:#x}", value);
        }

    unsafe {
            let value: u64;
            asm!("mrs {}, ID_MMFR4_EL1", out(reg) value);
            println!("ID_MMFR4_EL1={:#x}", value);
        }

    unsafe {
            let value: u64;
            asm!("mrs {}, ID_MMFR5_EL1", out(reg) value);
            println!("ID_MMFR5_EL1={:#x}", value);
        }

    unsafe {
            let value: u64;
            asm!("mrs {}, ID_PFR0_EL1", out(reg) value);
            println!("ID_PFR0_EL1={:#x}", value);
        }

    unsafe {
            let value: u64;
            asm!("mrs {}, ID_PFR1_EL1", out(reg) value);
            println!("ID_PFR1_EL1={:#x}", value);
        }

    unsafe {
            let value: u64;
            asm!("mrs {}, MIDR_EL1", out(reg) value);
            println!("MIDR_EL1={:#x}", value);
        }

    unsafe {
            let value: u64;
            asm!("mrs {}, MPIDR_EL1", out(reg) value);
            println!("MPIDR_EL1={:#x}", value);
        }

    unsafe {
            let value: u64;
            asm!("mrs {}, MVFR0_EL1", out(reg) value);
            println!("MVFR0_EL1={:#x}", value);
        }

    unsafe {
            let value: u64;
            asm!("mrs {}, MVFR1_EL1", out(reg) value);
            println!("MVFR1_EL1={:#x}", value);
        }

    unsafe {
            let value: u64;
            asm!("mrs {}, MVFR2_EL1", out(reg) value);
            println!("MVFR2_EL1={:#x}", value);
        }

    unsafe {
            let value: u64;
            asm!("mrs {}, PMCEID0_EL0", out(reg) value);
            println!("PMCEID0_EL0={:#x}", value);
        }

    unsafe {
            let value: u64;
            asm!("mrs {}, PMCEID1_EL0", out(reg) value);
            println!("PMCEID1_EL0={:#x}", value);
        }

    unsafe {
            let value: u64;
            asm!("mrs {}, PMCR_EL0", out(reg) value);
            println!("PMCR_EL0={:#x}", value);
        }

    unsafe {
            let value: u64;
            asm!("mrs {}, REVIDR_EL1", out(reg) value);
            println!("REVIDR_EL1={:#x}", value);
        }

    unsafe {
            let value: u64;
            asm!("mrs {}, RVBAR_EL1", out(reg) value);
            println!("RVBAR_EL1={:#x}", value);
        }

    unsafe {
            let value: u64;
            asm!("mrs {}, RVBAR_EL2", out(reg) value);
            println!("RVBAR_EL2={:#x}", value);
        }

    unsafe {
            let value: u64;
            asm!("mrs {}, RVBAR_EL3", out(reg) value);
            println!("RVBAR_EL3={:#x}", value);
        }

    unsafe {
            let value: u64;
            asm!("mrs {}, SCTLR_EL3", out(reg) value);
            println!("SCTLR_EL3={:#x}", value);
        }


    unsafe {
            let value: u64;
            asm!("mrs {}, TPIDR_EL3", out(reg) value);
            println!("TPIDR_EL3={:#x}", value);
        }

    unsafe {
            let value: u64;
            asm!("mrs {}, VMPIDR_EL2", out(reg) value);
            println!("VMPIDR_EL2={:#x}", value);
        }

    unsafe {
            let value: u64;
            asm!("mrs {}, VPIDR_EL2", out(reg) value);
            println!("VPIDR_EL2={:#x}", value);
        }
        
    unsafe {
        // this is for EFI to properly execute run/boot time services
        println!("Restoring VBAR_EL1 to {:#x}", previous_vbar);
        asm!("msr VBAR_EL1, {}", in(reg) previous_vbar);
    }
    return 0;
}

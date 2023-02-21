/* 
    SPDX-License-Identifier: Mozilla Public License 2.0
    Copyrigth (c) 2022-2023 François-Frédéric Ozog
    
*/

use alloc::boxed::Box;
use core::arch::asm;
use core::arch::global_asm;

use crate::PlatformOperations;

use crate::println;

use crate::processor::ExceptionFrame;


global_asm!("
exception_table:

    sub     sp, sp, #304
    stp     x0, x1, [sp]
    stp     x2, x3, [sp, #16]

    adr     x0, reloc_offset
    ldr     x0, [x0]
    adr     x1, sync_excetion_same_el_sp0
    sub     x2, x1, x0
    adr     x1, trampoline
    sub     x1, x1, x0

    br      x1 // trampoline
    
reloc_offset:
    .quad   0

. = exception_table + 0x200
    sub     sp, sp, #304
    stp     x0, x1, [sp]
    stp     x2, x3, [sp, #16]

    adr     x0, reloc_offset
    ldr     x0, [x0]
    adr     x1, sync_excetion_same_el_spx
    sub     x2, x1, x0
    adr     x1, trampoline
    sub     x1, x1, x0

    br      x1 // trampoline


. = exception_table + 0x800

trampoline:
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
    add     x21, sp, #304
    stp     x30, x21, [sp, #240]
    
    # preserve flags, return address and syndrome
    mrs     x22, elr_el1
    mrs     x23, spsr_el1
    stp     x22, x23, [sp, #256]
    mrs     x24, esr_el1
    str     x24, [sp, #272]

    # make a new stack frame for backtrace to work in the future
    stp     x29, x22, [sp, #288]                                                                                               
    add     x29, sp, #288


    mov     x8, x2               // get the handler address in x8
    mov     x0, sp               // get Exception in x0

    blr     x8

    msr     daifset, #0xf

    #restore returning environment
    ldp     x22, x23, [sp, #256]
    msr     elr_el1, x22
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

    add     sp, sp, #304

    eret

");

extern "C" {
    fn exception_table() -> !;
    fn reloc_offset() -> !;
}

static mut PREVIOUS_VBAR: u64 = 0;

#[export_name = "sync_excetion_same_el_sp0"]
extern "C" fn sync_excetion_same_el_sp0( ef : &mut ExceptionFrame) -> u64 {
    let ec = (ef.esr_el1 >> 26) & 0x3f;
    panic!("Unsupported sync_excetion_same_el_sp0 {:#x} at {:#x}", ec, ef.elr_el1);
}

#[export_name = "sync_excetion_same_el_spx"]
extern "C" fn sync_excetion_same_el_spx( ef : &mut ExceptionFrame) -> u64 {
    let ec = (ef.esr_el1 >> 26) & 0x3f;
    if ec == 0 {
        // it means the register can't be red from current EL or is not implemented
        ef.elr_el1 += 4;
    }
    else {
        panic!("Unsupported sync_excetion_same_el_spx {:#x} at {:#x}", ec, ef.elr_el1);
    }
    return 0;
}

pub fn run(_platform:&Box<dyn PlatformOperations>) -> i64 {
    let current_el : u64;
    unsafe {
        asm!("mrs {}, currentEL", out(reg)current_el);
    }
    println!("ID registers at startup {:#x}\n", (current_el >> 2 ) &3);

    let  barekit_vbar : u64;

    unsafe {
        asm!("mrs {}, VBAR_EL1", inout(reg) PREVIOUS_VBAR);
        barekit_vbar = exception_table as u64;
    }


    unsafe {
        // kvmtool sets VBAR to a special value 
        //TODO: kvmtool to set it to (cached value)
        if PREVIOUS_VBAR != 0 && PREVIOUS_VBAR != 0xf0000000 {

            let offset = reloc_offset as *mut u64;
            *offset = PREVIOUS_VBAR - barekit_vbar;
            println!("reloc_offset set to {:#x}", PREVIOUS_VBAR - barekit_vbar);

            let mut target = PREVIOUS_VBAR as *mut u64;
            let mut source =  barekit_vbar as *const u64;
            println!("Copy barekit handler from {:#x} to {:#x}", source as u64, target as u64);
            let mut i:u32 = 0;
            while i < (0x80 / 8)
            {
                *target = *source;
                target = target.add(1);
                source = source.add(1);
                i+=1;
            }
            source = exception_table as *const u64;
            source = source.add(0x200/8);
            target = PREVIOUS_VBAR as *mut u64;
            target = target.add(0x200/8);
            println!("Copy barekit handler from {:#x} to {:#x}", source as u64, target as u64);
            i = 0;
            while i < 0x80 / 8
            {
                *target = *source;
                target = target.add(1);
                source = source.add(1);
                i+=1;
            }
        }
        else {
            println!("Setting VBAR_EL1 to {:#x}", barekit_vbar);
            asm!("msr VBAR_EL1, {}", in(reg) barekit_vbar);
        }

    }


    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, CCSIDR_EL1", inout(reg) value);
        println!("CCSIDR_EL1={:#x}", value);
    }

    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, CLIDR_EL1", inout(reg) value);
        println!("CLIDR_EL1={:#x}", value);
    }

    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, CTR_EL0", inout(reg) value);
        println!("CTR_EL0={:#x}", value);
    }

    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, ID_AA64AFR0_EL1", inout(reg) value);
        println!("ID_AA64AFR0_EL1={:#x}", value);
    }

    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, ID_AA64AFR1_EL1", inout(reg) value);
        println!("ID_AA64AFR1_EL1={:#x}", value);
    }

    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, ID_AA64DFR0_EL1", inout(reg) value);
        println!("ID_AA64DFR0_EL1={:#x}", value);
    }

    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, ID_AA64DFR1_EL1", inout(reg) value);
        println!("ID_AA64DFR1_EL1={:#x}", value);
    }

    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, ID_AA64ISAR0_EL1", inout(reg) value);
        println!("ID_AA64ISAR0_EL1={:#x}", value);
    }

    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, ID_AA64ISAR1_EL1", inout(reg) value);
        println!("ID_AA64ISAR1_EL1={:#x}", value);
    }

    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, ID_AA64ISAR2_EL1", inout(reg) value);
        println!("ID_AA64ISAR2_EL1={:#x}", value);
    }

    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, ID_AA64MMFR0_EL1", inout(reg) value);
        println!("ID_AA64MMFR0_EL1={:#x}", value);
    }

    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, ID_AA64MMFR1_EL1", inout(reg) value);
        println!("ID_AA64MMFR1_EL1={:#x}", value);
    }

    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, ID_AA64MMFR2_EL1", inout(reg) value);
        println!("ID_AA64MMFR2_EL1={:#x}", value);
    }


    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, ID_AA64PFR0_EL1", inout(reg) value);
        println!("ID_AA64PFR0_EL1={:#x}", value);
    }

    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, ID_AA64PFR1_EL1", inout(reg) value);
        println!("ID_AA64PFR1_EL1={:#x}", value);
    }

    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, ID_AFR0_EL1", inout(reg) value);
        println!("ID_AFR0_EL1={:#x}", value);
    }

    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, ID_AFR0_EL1", inout(reg) value);
        println!("ID_AFR0_EL1={:#x}", value);
    }

    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, ID_DFR0_EL1", inout(reg) value);
        println!("ID_DFR0_EL1={:#x}", value);
    }

    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, ID_ISAR0_EL1", inout(reg) value);
        println!("ID_ISAR0_EL1={:#x}", value);
    }

    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, ID_ISAR1_EL1", inout(reg) value);
        println!("ID_ISAR1_EL1={:#x}", value);
    }

    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, ID_ISAR2_EL1", inout(reg) value);
        println!("ID_ISAR2_EL1={:#x}", value);
    }

    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, ID_ISAR3_EL1", inout(reg) value);
        println!("ID_ISAR3_EL1={:#x}", value);
    }

    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, ID_ISAR4_EL1", inout(reg) value);
        println!("ID_ISAR4_EL1={:#x}", value);
    }

    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, ID_ISAR5_EL1", inout(reg) value);
        println!("ID_ISAR5_EL1={:#x}", value);
    }


    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, ID_MMFR0_EL1", inout(reg) value);
        println!("ID_MMFR0_EL1={:#x}", value);
    }

    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, ID_MMFR1_EL1", inout(reg) value);
        println!("ID_MMFR1_EL1={:#x}", value);
    }

    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, ID_MMFR2_EL1", inout(reg) value);
        println!("ID_MMFR2_EL1={:#x}", value);
    }

    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, ID_MMFR3_EL1", inout(reg) value);
        println!("ID_MMFR3_EL1={:#x}", value);
    }

    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, ID_MMFR4_EL1", inout(reg) value);
        println!("ID_MMFR4_EL1={:#x}", value);
    }

    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, ID_MMFR5_EL1", inout(reg) value);
        println!("ID_MMFR5_EL1={:#x}", value);
    }

    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, ID_PFR0_EL1", inout(reg) value);
        println!("ID_PFR0_EL1={:#x}", value);
    }

    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, ID_PFR1_EL1", inout(reg) value);
        println!("ID_PFR1_EL1={:#x}", value);
    }

    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, MIDR_EL1", inout(reg) value);
        println!("MIDR_EL1={:#x}", value);
    }

    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, MPIDR_EL1", inout(reg) value);
        println!("MPIDR_EL1={:#x}", value);
    }

    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, MVFR0_EL1", inout(reg) value);
        println!("MVFR0_EL1={:#x}", value);
    }

    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, MVFR1_EL1", inout(reg) value);
        println!("MVFR1_EL1={:#x}", value);
    }

    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, MVFR2_EL1", inout(reg) value);
        println!("MVFR2_EL1={:#x}", value);
    }

    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, PMCEID0_EL0", inout(reg) value);
        println!("PMCEID0_EL0={:#x}", value);
    }

    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, PMCEID1_EL0", inout(reg) value);
        println!("PMCEID1_EL0={:#x}", value);
    }

    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, PMCR_EL0", inout(reg) value);
        println!("PMCR_EL0={:#x}", value);
    }

    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, REVIDR_EL1", inout(reg) value);
        println!("REVIDR_EL1={:#x}", value);
    }


    unsafe {
        let mut value: u64 = 0;
        let elr_el1: u64;
        let addr : u64;
        asm!(
            "mrs {a}, RVBAR_EL2", 
            "adr {c}, 0",
            "mrs {b}, elr_el1",
            a = inout(reg) value,
            b = out(reg) elr_el1,
            c = out(reg) addr
        );
        if elr_el1 != addr {
            println!("RVBAR_EL2={:#x}   {:#x}   {:#x}", value, elr_el1, addr);
        }
        else {
            println!("// could not access RVBAR_EL2");
        }
    }


    unsafe {
        let mut value: u64 = 0;
        let elr_el1: u64;
        let addr : u64;
        asm!(
            "mrs {a}, RVBAR_EL3", 
            "adr {c}, 0",
            "mrs {b}, elr_el1",
            a = inout(reg) value,
            b = out(reg) elr_el1,
            c = out(reg) addr
        );
        if elr_el1 != addr {
            println!("RVBAR_EL3={:#x}   {:#x}   {:#x}", value, elr_el1, addr);
        }
        else {
            println!("// could not access RVBAR_EL3");
        }
    }

    unsafe {
        let mut value: u64 = 0;
        let elr_el1: u64;
        let addr : u64;
        asm!(
            "mrs {a}, SCTLR_EL3", 
            "adr {c}, 0",
            "mrs {b}, elr_el1",
            a = inout(reg) value,
            b = out(reg) elr_el1,
            c = out(reg) addr
        );
        if elr_el1 != addr {
            println!("SCTLR_EL3={:#x}   {:#x}   {:#x}", value, elr_el1, addr);
        }
        else {
            println!("// could not access SCTLR_EL3");
        }
    }
    
    unsafe {
        let mut value: u64 = 0;
        let elr_el1: u64;
        let addr : u64;
        asm!(
            "mrs {a}, TPIDR_EL3", 
            "adr {c}, 0",
            "mrs {b}, elr_el1",
            a = inout(reg) value,
            b = out(reg) elr_el1,
            c = out(reg) addr
        );
        if elr_el1 != addr {
            println!("TPIDR_EL3={:#x}   {:#x}   {:#x}", value, elr_el1, addr);
        }
        else {
            println!("// could not access TPIDR_EL3");
        }
    }

    unsafe {
        let mut value: u64 = 0;
        let elr_el1: u64;
        let addr : u64;
        asm!(
            "mrs {a}, VMPIDR_EL2", 
            "adr {c}, 0",
            "mrs {b}, elr_el1",
            a = inout(reg) value,
            b = out(reg) elr_el1,
            c = out(reg) addr
        );
        if elr_el1 != addr {
            println!("VMPIDR_EL2={:#x}   {:#x}   {:#x}", value, elr_el1, addr);
        }
        else {
            println!("// could not access VMPIDR_EL2");
        }
    }

    unsafe {
        let mut value: u64 = 0;
        let elr_el1: u64;
        let addr : u64;
        asm!(
            "mrs {a}, VPIDR_EL2", 
            "adr {c}, 0",
            "mrs {b}, elr_el1",
            a = inout(reg) value,
            b = out(reg) elr_el1,
            c = out(reg) addr
        );
        if elr_el1 != addr {
            println!("VPIDR_EL2={:#x}   {:#x}   {:#x}", value, elr_el1, addr);
        }
        else {
            println!("// could not access VPIDR_EL2");
        }
    }

    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, RVBAR_EL1", inout(reg) value);
        println!("RVBAR_EL1={:#x}", value);
    }


    unsafe {
        // this is for EFI to properly execute run/boot time services
        println!("Restoring VBAR_EL1 to {:#x}", PREVIOUS_VBAR);
        asm!("msr VBAR_EL1, {}", in(reg) PREVIOUS_VBAR);
    }
    return 0;
}

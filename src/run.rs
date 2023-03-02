/* 
    SPDX-License-Identifier: Mozilla Public License 2.0
    Copyrigth (c) 2022-2023 François-Frédéric Ozog
    
*/

use alloc::boxed::Box;
use core::arch::asm;
use core::arch::global_asm;

use crate::PlatformOperations;

use crate::println;
use crate::print;

use crate::processor;
use crate::processor::ExceptionFrame;


global_asm!("
.align 11
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


global_asm!("
.align 11
exception_table_el2:

    sub     sp, sp, #304
    stp     x0, x1, [sp]
    stp     x2, x3, [sp, #16]

    adr     x0, reloc_offset_el2
    ldr     x0, [x0]
    adr     x1, sync_excetion_same_el_sp0
    sub     x2, x1, x0
    adr     x1, trampoline_el2
    sub     x1, x1, x0

    br      x1 // trampoline
    
reloc_offset_el2:
    .quad   0

. = exception_table_el2 + 0x200
    sub     sp, sp, #304
    stp     x0, x1, [sp]
    stp     x2, x3, [sp, #16]

    adr     x0, reloc_offset_el2
    ldr     x0, [x0]
    adr     x1, sync_excetion_same_el_spx
    sub     x2, x1, x0
    adr     x1, trampoline_el2
    sub     x1, x1, x0

    br      x1 // trampoline

    . = exception_table_el2 + 0x400
    sub     sp, sp, #304
    stp     x0, x1, [sp]
    stp     x2, x3, [sp, #16]

    adr     x0, reloc_offset_el2
    ldr     x0, [x0]
    adr     x1, sync_excetion_lower_el_aarch64
    sub     x2, x1, x0
    adr     x1, trampoline_el2
    sub     x1, x1, x0

    br      x1 // trampoline

    . = exception_table_el2 + 0x600
    sub     sp, sp, #304
    stp     x0, x1, [sp]
    stp     x2, x3, [sp, #16]

    adr     x0, reloc_offset_el2
    ldr     x0, [x0]
    adr     x1, sync_excetion_lower_el_aarch32
    sub     x2, x1, x0
    adr     x1, trampoline_el2
    sub     x1, x1, x0

    br      x1 // trampoline

. = exception_table_el2 + 0x800

trampoline_el2:
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
    mrs     x22, elr_el2
    mrs     x23, spsr_el2
    stp     x22, x23, [sp, #256]
    mrs     x24, esr_el2
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
    msr     elr_el2, x22
    msr     spsr_el2, x23

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


global_asm!("
.align 11
exception_table_el3:

    sub     sp, sp, #304
    stp     x0, x1, [sp]
    stp     x2, x3, [sp, #16]

    adr     x0, reloc_offset_el3
    ldr     x0, [x0]
    adr     x1, sync_excetion_same_el_sp0
    sub     x2, x1, x0
    adr     x1, trampoline_el3
    sub     x1, x1, x0

    br      x1 // trampoline
    
reloc_offset_el3:
    .quad   0

. = exception_table_el3 + 0x200
    sub     sp, sp, #304
    stp     x0, x1, [sp]
    stp     x2, x3, [sp, #16]

    adr     x0, reloc_offset_el3
    ldr     x0, [x0]
    adr     x1, sync_excetion_same_el_spx
    sub     x2, x1, x0
    adr     x1, trampoline_el3
    sub     x1, x1, x0

    br      x1 // trampoline

    . = exception_table_el3 + 0x400
    sub     sp, sp, #304
    stp     x0, x1, [sp]
    stp     x2, x3, [sp, #16]

    adr     x0, reloc_offset_el3
    ldr     x0, [x0]
    adr     x1, sync_excetion_lower_el_aarch64
    sub     x2, x1, x0
    adr     x1, trampoline_el3
    sub     x1, x1, x0

    br      x1 // trampoline

    . = exception_table_el3 + 0x600
    sub     sp, sp, #304
    stp     x0, x1, [sp]
    stp     x2, x3, [sp, #16]

    adr     x0, reloc_offset_el3
    ldr     x0, [x0]
    adr     x1, sync_excetion_lower_el_aarch32
    sub     x2, x1, x0
    adr     x1, trampoline_el3
    sub     x1, x1, x0

    br      x1 // trampoline

. = exception_table_el3 + 0x800

trampoline_el3:
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
    mrs     x22, elr_el3
    mrs     x23, spsr_el3
    stp     x22, x23, [sp, #256]
    mrs     x24, esr_el3
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
    msr     elr_el3, x22
    msr     spsr_el3, x23

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
    fn exception_table_el2() -> !;
    fn reloc_offset_el2() -> !;
    fn exception_table_el3() -> !;
    fn reloc_offset_el3() -> !;
}





#[export_name = "sync_excetion_same_el_sp0"]
extern "C" fn sync_excetion_same_el_sp0( ef : &mut ExceptionFrame) -> u64 {
    let ec = (ef.esr >> 26) & 0x3f;
    panic!("Unsupported sync_excetion_same_el_sp0 {:#x} at {:#x}", ec, ef.elr);
}

#[export_name = "sync_excetion_same_el_spx"]
extern "C" fn sync_excetion_same_el_spx( ef : &mut ExceptionFrame) -> u64 {
    let ec = (ef.esr >> 26) & 0x3f;
    //println!("sync_excetion_same_el_spx {:#x} at {:#x}", ec, ef.elr);
    if ec == 0 {
        // it means the register can't be red from current EL or is not implemented
        ef.elr += 4;
    }
    else {
        panic!("Unsupported sync_excetion_same_el_spx {:#x} at {:#x}", ec, ef.elr);
    }
    return 0;
}

#[export_name = "sync_excetion_lower_el_aarch64"]
extern "C" fn sync_excetion_lower_el_aarch64( ef : &mut ExceptionFrame) -> u64 {
    let ec = (ef.esr >> 26) & 0x3f;
    //println!("sync_excetion_lower_el_aarch64 {:#x} at {:#x}", ec, ef.elr);
    if ec == 0 {
        // it means the register can't be red from current EL or is not implemented
        ef.elr += 4;
    }
    else {
        panic!("Unsupported sync_excetion_same_el_spx {:#x} at {:#x}", ec, ef.elr);
    }
    return 0;
}

#[export_name = "sync_excetion_lower_el_aarch32"]
extern "C" fn sync_excetion_lower_el_aarch32( ef : &mut ExceptionFrame) -> u64 {
    let ec = (ef.esr >> 26) & 0x3f;
    panic!("Unsupported sync_excetion_lower_el_aarch32 {:#x} at {:#x}", ec, ef.elr);
}




#[allow(dead_code)]
fn dump_paging_step(anchor : u64, base : u64, level: usize, level_size: usize) {

    if level > 4 {
        return;
    }
    //println!("dump_paging_step(anchor={:#x}, base={:#x}, level={}, level_size={}", anchor, base, level, level_size);
    unsafe { 
        let bits: u8 = processor::BITS_AT_LEVEL[level];
        let size:usize = 1 << bits;
        let mut count = level_size / size;
        if count > 512 { count = 512};
        let mut i : usize = 0;
        let mut table = anchor as *const u64;
        while i < count {
            let va_start: u64 = base + (size * i) as u64;
            let va_end : u64 = va_start + (size - 1) as u64;
            if  processor::page_is_present(*table) {
                if processor::page_is_table(*table, bits) {
                    let target = processor::table_target_at_index(*table , level);
                    println!("l{}_{:#016x}[{}]={:#016x}: table @ {:#016x} for VA {:#016x} - {:#016x}", 
                        level - 1, anchor, i, 
                        *table, 
                        target as u64, va_start, va_end
                    );
                    dump_paging_step(target as u64, va_start, level+1, size)
                }
                else {
                    let target =  processor::page_target_at_index(*table , level);
                    let ro = (*table & (1 << 7)) !=0;
                    println!("l{}_{:#016x}[{}]={:#016x}: mapping (ro={}) for  VA {:#016x} - {:#016x} ->  PA {:#016x} - {:#016x}",
                        level - 1, anchor, i, 
                        *table, ro, 
                        va_start, va_end, target as u64, target as u64+ (size -1) as u64
                        );
                }
                
            }
            i += 1;
            table = table.add(1);
        }
    }
}


#[allow(dead_code)]
fn dump_paging() {
    let anchor = processor::get_anchor_for(0);
    let info = processor::paging_get_low_mem_paging();
    dump_paging_step(anchor, 0, info.0 as usize, info.1);
}


static mut PREVIOUS_VBAR: u64 = 0;


fn generate_cpu_vobj() {
    print!("\n");
    print!("-vobj 'CPU#name=\"\";");


    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, MIDR_EL1", inout(reg) value);
        if value != 0 {
            print!("MIDR_EL1={:#x};", value);
        }
    }

    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, MPIDR_EL1", inout(reg) value);
        if value != 0 {
            print!("MPIDR_EL1={:#x};", value);
        }
    }

    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, REVIDR_EL1", inout(reg) value);
        if value != 0 {
            print!("REVIDR_EL1={:#x};", value);
        }
    }
    
    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, RVBAR_EL1", inout(reg) value);
        if value != 0 {
            print!("RVBAR_EL1={:#x};", value);
        }
    }

    unsafe {
        let mut value: u64 = 0;
        asm!(
            "mrs {a}, RVBAR_EL2", 
            a = inout(reg) value
        );
        if value != 0 {
            print!("RVBAR_EL2={:#x};", value);
        }
    }


    unsafe {
        let mut value: u64 = 0;
        let addr : u64;
        asm!(
            "mrs {a}, RVBAR_EL3", 
            "adr {b}, 0",
            a = inout(reg) value,
            b = out(reg) addr
        );
        if processor::get_elr() != addr {
            if value != 0 {
                print!("RVBAR_EL3={:#x};", value);
            }
        }
    }

/* 
    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, CCSIDR_EL1", inout(reg) value);
        if value != 0 {
            print!("CCSIDR_EL1={:#x};", value);
        }
    }

    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, CLIDR_EL1", inout(reg) value);
        if value != 0 {
            print!("CLIDR_EL1={:#x};", value);
        }
    }
*/
    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, ID_AA64AFR0_EL1", inout(reg) value);
        if value != 0 {
            print!("ID_AA64AFR0_EL1={:#x};", value);
        }
    }

    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, ID_AA64AFR1_EL1", inout(reg) value);
        if value != 0 {
            print!("ID_AA64AFR1_EL1={:#x};", value);
        }
    }

    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, ID_AA64DFR0_EL1", inout(reg) value);
        if value != 0 {
            print!("ID_AA64DFR0_EL1={:#x};", value);
        }
    }

    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, ID_AA64DFR1_EL1", inout(reg) value);
        if value != 0 {
            print!("ID_AA64DFR1_EL1={:#x};", value);
        }
    }

    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, ID_AA64ISAR0_EL1", inout(reg) value);
        if value != 0 {
            print!("ID_AA64ISAR0_EL1={:#x};", value);
        }
    }

    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, ID_AA64ISAR1_EL1", inout(reg) value);
        if value != 0 {
            print!("ID_AA64ISAR1_EL1={:#x};", value);
        }
    }

    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, ID_AA64ISAR2_EL1", inout(reg) value);
        if value != 0 {
            print!("ID_AA64ISAR2_EL1={:#x};", value);
        }
    }

    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, ID_AA64MMFR0_EL1", inout(reg) value);
        if value != 0 {
            print!("ID_AA64MMFR0_EL1={:#x};", value);
        }
    }

    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, ID_AA64MMFR1_EL1", inout(reg) value);
        if value != 0 {
            print!("ID_AA64MMFR1_EL1={:#x};", value);
        }
    }

    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, ID_AA64MMFR2_EL1", inout(reg) value);
        if value != 0 {
            print!("ID_AA64MMFR2_EL1={:#x};", value);
        }
    }


    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, ID_AA64PFR0_EL1", inout(reg) value);
        if value != 0 {
            print!("ID_AA64PFR0_EL1={:#x};", value);
        }
    }

    unsafe {
        let mut value: u64 = 0;
        let addr : u64;
        asm!(
            "mrs {a}, ID_AA64PFR1_EL1", 
            "adr {b}, 0",
            a = inout(reg) value,
            b = out(reg) addr
        );
        if processor::get_elr() != addr {
            if value != 0 {
                print!("ID_AA64PFR1_EL1={:#x};", value);
            }
        }
    }

    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, ID_AFR0_EL1", inout(reg) value);
        if value != 0 {
            print!("ID_AFR0_EL1={:#x};", value);
        }
    }

    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, ID_DFR0_EL1", inout(reg) value);
        if value != 0 {
            print!("ID_DFR0_EL1={:#x};", value);
        }
    }

    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, ID_ISAR0_EL1", inout(reg) value);
        if value != 0 {
            print!("ID_ISAR0_EL1={:#x};", value);
        }
    }

    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, ID_ISAR1_EL1", inout(reg) value);
        if value != 0 {
            print!("ID_ISAR1_EL1={:#x};", value);
        }
    }

    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, ID_ISAR2_EL1", inout(reg) value);
        if value != 0 {
            print!("ID_ISAR2_EL1={:#x};", value);
        }
    }

    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, ID_ISAR3_EL1", inout(reg) value);
        if value != 0 {
            print!("ID_ISAR3_EL1={:#x};", value);
        }
    }

    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, ID_ISAR4_EL1", inout(reg) value);
        if value != 0 {
            print!("ID_ISAR4_EL1={:#x};", value);
        }
    }

    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, ID_ISAR5_EL1", inout(reg) value);
        if value != 0 {
            print!("ID_ISAR5_EL1={:#x};", value);
        }
    }


    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, ID_MMFR0_EL1", inout(reg) value);
        if value != 0 {
            print!("ID_MMFR0_EL1={:#x};", value);
        }
    }

    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, ID_MMFR1_EL1", inout(reg) value);
        if value != 0 {
            print!("ID_MMFR1_EL1={:#x};", value);
        }
    }

    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, ID_MMFR2_EL1", inout(reg) value);
        if value != 0 {
            print!("ID_MMFR2_EL1={:#x};", value);
        }
    }

    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, ID_MMFR3_EL1", inout(reg) value);
        if value != 0 {
            print!("ID_MMFR3_EL1={:#x};", value);
        }
    }

    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, ID_MMFR4_EL1", inout(reg) value);
        if value != 0 {
            print!("ID_MMFR4_EL1={:#x};", value);
        }
    }

    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, ID_MMFR5_EL1", inout(reg) value);
        if value != 0 {
            print!("ID_MMFR5_EL1={:#x};", value);
        }
    }

    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, ID_PFR0_EL1", inout(reg) value);
        if value != 0 {
            print!("ID_PFR0_EL1={:#x};", value);
        }
    }

    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, ID_PFR1_EL1", inout(reg) value);
        if value != 0 {
            print!("ID_PFR1_EL1={:#x};", value);
        }
    }
/*
    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, MVFR0_EL1", inout(reg) value);
        if value != 0 {
            print!("MVFR0_EL1={:#x};", value);
        }
    }

    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, MVFR1_EL1", inout(reg) value);
        if value != 0 {
            print!("MVFR1_EL1={:#x};", value);
        }
    }

    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, MVFR2_EL1", inout(reg) value);
        if value != 0 {
            print!("MVFR2_EL1={:#x};", value);
        }
    }

    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, PMCEID0_EL0", inout(reg) value);
        if value != 0 {
            print!("PMCEID0_EL0={:#x};", value);
        }
    }

    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, PMCEID1_EL0", inout(reg) value);
        if value != 0 {
            print!("PMCEID1_EL0={:#x};", value);
        }
    }

    unsafe {
        let mut value: u64 = 0;
        asm!("mrs {}, PMCR_EL0", inout(reg) value);
        if value != 0 {
            print!("PMCR_EL0={:#x};", value);
        }
    }

  
    unsafe {
        let mut value: u64 = 0;
        let addr : u64;
        asm!(
            "mrs {a}, TPIDR_EL3", 
            "adr {b}, 0",
            a = inout(reg) value,
            b = out(reg) addr
        );
        if processor::get_elr() != addr {
            if value != 0 {
                print!("TPIDR_EL3={:#x};", value);
            }
        }
    }

    unsafe {
        let mut value: u64 = 0;
        let addr : u64;
        asm!(
            "mrs {a}, VMPIDR_EL2", 
            "adr {b}, 0",
            a = inout(reg) value,
            b = out(reg) addr
        );
        if processor::get_elr() != addr {
            if value != 0 {
                print!("VMPIDR_EL2={:#x};", value);
            }
        }
    }

    unsafe {
        let mut value: u64 = 0;
        let addr : u64;
        asm!(
            "mrs {a}, VPIDR_EL2", 
            "adr {b}, 0",
            a = inout(reg) value,
            b = out(reg) addr
        );
        if processor::get_elr() != addr {
            if value != 0 {
                print!("VPIDR_EL2={:#x};", value);
            }
        }
    }

*/

    println!("\x08 ||hostcpu#cluster=P'");

}

pub fn run(_platform:&Box<dyn PlatformOperations>) -> i64 {

    let  current_el = processor::get_current_el();

    println!("ID registers at startup EL-{}\n", current_el);

    let  barekit_vbar : u64;

    unsafe {
        //asm!("mrs {}, VBAR_EL1", inout(reg) PREVIOUS_VBAR);
        PREVIOUS_VBAR = processor::get_vbar();
        
        match current_el {
            1 => barekit_vbar = exception_table as u64,
            2 => barekit_vbar = exception_table_el2 as u64,
            3 => barekit_vbar = exception_table_el3 as u64,
            _ => panic!("Invalid EL")
        }

    }

    unsafe {
        // kvmtool sets VBAR to a special value 
        //TODO: kvmtool to set it to (cached value)
        if PREVIOUS_VBAR != 0 && PREVIOUS_VBAR != 0xf0000000 && PREVIOUS_VBAR != 0xf1000000 {

            //dump_paging();

            //println!("PREVIOUS_VBAR {:#x}", PREVIOUS_VBAR);

            let info = processor::paging_virtual_info(PREVIOUS_VBAR);
            if let Some(vbar_page_info) = info {
                //let page_size = vbar_page_info.0;
                let entry=vbar_page_info.2 as *mut u64;
                //println!("VBAR decriptor is at {:#x}, page is {} bytes", entry as u64, page_size);
                // turn it RW
                *entry = *entry & ! (1 << 7);
                let location = (entry as u64) & 0xFFF;
                processor::paging_invalidate_for(location);
            }

            let mut target = PREVIOUS_VBAR as *mut u64;
            let mut source =  barekit_vbar as *const u64;
            //println!("Copy barekit handler from {:#x} to {:#x}", source as u64, target as u64);
            let mut i:u32 = 0;
            while i < (0x80 / 8)
            {
                *target = *source;
                target = target.add(1);
                source = source.add(1);
                i+=1;
            }
            source = barekit_vbar as *const u64;
            source = source.add(0x200/8);
            target = PREVIOUS_VBAR as *mut u64;
            target = target.add(0x200/8);
            //println!("Copy barekit handler from {:#x} to {:#x}", source as u64, target as u64);
            i = 0;
            while i < 0x80 / 8
            {
                *target = *source;
                target = target.add(1);
                source = source.add(1);
                i+=1;
            }
            source = barekit_vbar as *const u64;
            source = source.add(0x400/8);
            target = PREVIOUS_VBAR as *mut u64;
            target = target.add(0x400/8);
            //println!("Copy barekit handler from {:#x} to {:#x}", source as u64, target as u64);
            i = 0;
            while i < 0x80 / 8
            {
                *target = *source;
                target = target.add(1);
                source = source.add(1);
                i+=1;
            }
            source = barekit_vbar as *const u64;
            source = source.add(0x600/8);
            target = PREVIOUS_VBAR as *mut u64;
            target = target.add(0x600/8);
            //println!("Copy barekit handler from {:#x} to {:#x}", source as u64, target as u64);
            i = 0;
            while i < 0x80 / 8
            {
                *target = *source;
                target = target.add(1);
                source = source.add(1);
                i+=1;
            }

            let offset ;
            if current_el == 1 {
                offset = ((reloc_offset as u64) - (exception_table as u64)+ PREVIOUS_VBAR) as *mut u64;
                *offset = PREVIOUS_VBAR - exception_table as u64;
            } 
            else if current_el == 2 {
                offset = ((reloc_offset_el2 as u64) - (exception_table_el2 as u64)+ PREVIOUS_VBAR) as *mut u64;
                *offset = PREVIOUS_VBAR - exception_table_el2 as u64
            }
            else if current_el == 3 {
                offset = ((reloc_offset_el3 as u64) - (exception_table_el3 as u64)+ PREVIOUS_VBAR) as *mut u64;
                *offset = PREVIOUS_VBAR - exception_table_el3 as u64
            }
                        
            //println!("reloc_offset set to {:#x}", PREVIOUS_VBAR - barekit_vbar);

            asm!(
                "dc cvau, {a}",
                "dsb ish",
                "ic ivau, {a}",
                "dsb ish",
                "isb sy",
                a = in(reg) PREVIOUS_VBAR
            );
            asm!(
                "dc cvau, {a}",
                "dsb ish",
                "ic ivau, {a}",
                "dsb ish",
                "isb sy",
                a = in(reg) PREVIOUS_VBAR+0x200
            );
            asm!(
                "dc cvau, {a}",
                "dsb ish",
                "ic ivau, {a}",
                "dsb ish",
                "isb sy",
                a = in(reg) PREVIOUS_VBAR + 0x400
            );
            asm!(
                "dc cvau, {a}",
                "dsb ish",
                "ic ivau, {a}",
                "dsb ish",
                "isb sy",
                a = in(reg) PREVIOUS_VBAR + 0x600
            );
        }
        else {
            //println!("Setting VBAR_EL1 to {:#x}", barekit_vbar);
            processor::set_vbar(barekit_vbar);
        }

    }

    generate_cpu_vobj();

    unsafe {
        // this is for EFI to properly execute run/boot time services
        if PREVIOUS_VBAR != 0 {            
            println!("Restoring current VBAR to {:#x}", PREVIOUS_VBAR);
            processor::set_vbar(PREVIOUS_VBAR);
        }
    }
    _platform.park();
    return 0;
}

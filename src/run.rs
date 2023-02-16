/* 
    SPDX-License-Identifier: Mozilla Public License 2.0
    Copyrigth (c) 2022-2023 François-Frédéric Ozog
    
*/

use alloc::boxed::Box;
use core::arch::asm;


use crate::PlatformOperations;
use crate::println;



pub fn run(_platform:&Box<dyn PlatformOperations>) -> i64 {
    println!("ID registers at startup\n");

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
        asm!("wfi");
    }
    return 0;
}

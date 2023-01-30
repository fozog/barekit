/* 
    SPDX-License-Identifier: Mozilla Public License 2.0
    Copyrigth (c) 2022-2023 François-Frédéric Ozog
    
*/

use core::arch::asm;

use crate::PlatformOperations;
use crate::PlatformInfo;
use crate::dt::DeviceTree;
use crate::early_prints;

#[cfg(feature = "early_print")]
use crate::print::_early_print_s;

pub struct Platform<'a> {
    fdt_address:    u64,
    information:    PlatformInfo,
    _dt:             Option<DeviceTree<'a>>
}

impl<'a> Platform<'a>  {

    pub fn new(information: PlatformInfo) -> Self {
        early_prints!("Creating S-EL1 platform\n", 0);
        early_prints!("     x2:$\n", information.x2_at_startup);
        early_prints!("     x3:$\n", information.x3_at_startup);
        Self { fdt_address: information.x0_at_startup, information, _dt: None } 
    }
    
}

impl<'a> PlatformOperations<'a> for Platform<'a> {

    fn stop(&self) {
        let tlk_entry_done: u64 = 0x32000003 | (1 << 31);
        // signals tlk_entry_done to tlkd in EL3
        // https://elixir.bootlin.com/arm-trusted-firmware/latest/source/services/spd/tlkd/tlkd_main.c#L405
        unsafe { 
            asm!(
                "mov x0, {x}",
                "smc #1",
                x = in(reg) tlk_entry_done    
            );
        }
    }

    fn get_fdt_address(&self) -> Option<u64> {
        if self.fdt_address == 0 {
            return None;
        }
        else {
            return  Some(self.fdt_address);
        }
    }
    
    fn get_info(&self) -> &PlatformInfo {
        &self.information
    }
 
}
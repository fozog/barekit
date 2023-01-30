/* 
    SPDX-License-Identifier: Mozilla Public License 2.0
    Copyrigth (c) 2022-2023 François-Frédéric Ozog
    
*/

use crate::PlatformOperations;
use crate::PlatformInfo;
use crate::dt::DeviceTree;
use crate::early_prints;


#[cfg(feature = "early_print")]
use crate::print::_early_print_s;

use alloc::vec::Vec;



pub struct Platform<'a> {
    fdt_address:    u64,
    information:    PlatformInfo,
    _dt:             Option<DeviceTree<'a>>,
    _dt_scratchpad:  Option<Vec<u8>>
}

impl<'a> Platform<'a> {

    pub fn new(information: PlatformInfo) -> Self {
        early_prints!("Creating EL 1&2 platform\n", 0);
        Self { fdt_address: information.x0_at_startup, information, _dt: None , _dt_scratchpad: None} 
    }

}

impl<'a> PlatformOperations<'a> for Platform<'a> {

    fn get_info(&self) -> &super::PlatformInfo {
        &self.information
    }

    fn get_fdt_address(&self) -> Option<u64> {
        Some(self.fdt_address)
    }

}
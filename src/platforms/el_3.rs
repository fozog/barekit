/* 
    SPDX-License-Identifier: Mozilla Public License 2.0
    Copyrigth (c) 2022-2023 François-Frédéric Ozog
    
*/

use alloc::boxed::Box;

use crate::PlatformOperations;
use crate::PlatformInfo;
use crate::dt::DeviceTree;
use crate::early_prints;


#[cfg(feature = "early_print")]
use crate::print::_early_print_s;

pub struct Platform<'a> {
    fdt_address:    u64,
    information:    PlatformInfo,
    _dt:            Option<Box<DeviceTree<'a>>>

}

impl<'a> Platform<'a>  {

    pub fn new(information: PlatformInfo) -> Self {
        early_prints!("Creating EL3 platform\n", 0);
        // make sure it can work on QEMU
        Self { fdt_address: 0x40000000, information, _dt: None } 
    }

}

impl<'a> PlatformOperations<'a> for Platform<'a> {

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

    fn set_devt(&'a mut self, devt: Option<Box<DeviceTree<'a>>>) {
        self._dt = devt;
    }

    fn is_secure(&self) -> bool {
        return true;
    }

}
/* 
    SPDX-License-Identifier: Mozilla Public License 2.0
    Copyrigth (c) 2022-2023 François-Frédéric Ozog
    
*/

use alloc::vec::Vec;
use alloc::boxed::Box;

use crate::PlatformOperations;
use crate::PlatformInfo;
use crate::dt::DeviceTree;
use crate::drivers;
use crate::drivers::ns16550a::NS16550Output;
use crate::early_prints;

use crate::log;
use crate::println;
#[cfg(feature = "early_print")]
use crate::print::_early_print_s;


pub struct Platform<'a> {
    fdt_address:    u64,
    information:    PlatformInfo,
    _dt:            Option<Box<DeviceTree<'a>>>,
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

    fn set_devt(&'a mut self, devt: Option<Box<DeviceTree<'a>>>) {
        self._dt = devt;
    }

    fn get_name(&self) -> &str {
        "EL1/2"
    }

    fn set_boot_tty(&mut self) {
        if self.fdt_address == 0 {
            let tty = NS16550Output::from_mmio(drivers::DESIGNWARE , 0xf051_2000,  1, 2);
            let s = log::get_unprinted();
            log::set_target(tty);
            println!("{}", &s);
            }
    }

}
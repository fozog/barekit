/* 
    SPDX-License-Identifier: Mozilla Public License 2.0
    Copyrigth (c) 2022-2023 François-Frédéric Ozog
    
*/

use alloc::boxed::Box;

use crate::PlatformOperations;
use crate::PlatformInfo;
use crate::drivers;
use crate::drivers::ns16550a::NS16550Output;
//use crate::drivers::pl011::PL011Output;
use crate::dt::DeviceTree;
use crate::early_prints;

use crate::log;
use crate::println;
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
        // QEMU
        //Self { fdt_address: 0x40000000, information, _dt: None } 
        // General case
        // for TFA: https://elixir.bootlin.com/arm-trusted-firmware/v2.8.0/source/common/desc_image_load.c#L293
        Self { fdt_address: information.x0_at_startup, information, _dt: None } 
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

    fn get_name(&self) -> &str {
        "EL3"
    }

    fn set_boot_tty(&mut self) {
        if self.fdt_address == 0 {
            let tty = NS16550Output::from_mmio(drivers::DESIGNWARE , 0xf051_2000,  1, 2);
            //let tty = PL011Output::from_mmio(drivers::PL011 , 0x0900_0000,  1, 2);
            let s = log::get_unprinted();
            log::set_target(tty);
            println!("{}", &s);
            }
    }

}

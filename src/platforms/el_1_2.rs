/* 
    SPDX-License-Identifier: Mozilla Public License 2.0
    Copyrigth (c) 2022-2023 François-Frédéric Ozog
    
*/

use alloc::vec::Vec;
use alloc::boxed::Box;
use core::ptr;

use crate::PlatformOperations;
use crate::PlatformInfo;
use crate::dt::DeviceTree;
use crate::drivers;
//use crate::drivers::pl011::PL011Output;
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

const FDT_MAGIC: u32 = 0xD00D_FEED;
const FALLBACK_FDT_ADDRESS: u64 = 0x400_00000;
const FDT_MIN_TOTALSIZE: u32 = 40;
const FDT_MAX_TOTALSIZE: u32 = 8 * 1024 * 1024;

fn read_be_u32(address: u64, offset: u64) -> u32 {
    let raw = unsafe { ptr::read_unaligned((address + offset) as *const u32) };
    u32::from_be(raw)
}

fn has_valid_fdt_header(address: u64) -> bool {
    if address == 0 {
        return false;
    }

    let magic = read_be_u32(address, 0);
    if magic != FDT_MAGIC {
        return false;
    }

    let totalsize = read_be_u32(address, 4);
    if totalsize < FDT_MIN_TOTALSIZE || totalsize > FDT_MAX_TOTALSIZE {
        return false;
    }

    let off_dt_struct = read_be_u32(address, 8);
    let off_dt_strings = read_be_u32(address, 12);
    let off_mem_rsvmap = read_be_u32(address, 16);

    if off_dt_struct >= totalsize || off_dt_strings >= totalsize || off_mem_rsvmap >= totalsize {
        return false;
    }

    if (off_dt_struct & 3) != 0 || (off_dt_strings & 3) != 0 || (off_mem_rsvmap & 7) != 0 {
        return false;
    }

    true
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
        if has_valid_fdt_header(self.fdt_address) {
            early_prints!("Using x0 as FDT address: %\n", self.fdt_address);
            Some(self.fdt_address)
        } else if has_valid_fdt_header(FALLBACK_FDT_ADDRESS) {
            early_prints!("Using fallback FDT address: %\n", FALLBACK_FDT_ADDRESS);
            Some(FALLBACK_FDT_ADDRESS)
        } else {
            early_prints!("No valid FDT address\n", 0);
            None
        }
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
            //let tty = PL011Output::from_mmio(drivers::PL011 , 0x0900_0000,  0, 0);
            let s = log::get_unprinted();
            log::set_target(tty);
            println!("{}", &s);
            }
    }

}

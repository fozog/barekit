/* 
    SPDX-License-Identifier: Mozilla Public License 2.0
    Copyrigth (c) 2022-2023 François-Frédéric Ozog
    
*/

use core::{fmt, sync::atomic::{AtomicU8, AtomicU32, Ordering}, hint};
use alloc::string::String;
use alloc::boxed::Box;
use crate::alloc::borrow::ToOwned;

use crate::{log::TTY, dt::DeviceTree};
use crate::log::Logger;

use fdt_rs::{common::prop::StringPropIter, index::DevTreeIndexNode};
use fdt_rs::prelude::FallibleIterator;
use fdt_rs::prelude::PropReader;

#[cfg(feature = "early_print")]
use crate::print::_early_print_s;
use crate::early_prints;

pub struct NS16550Output<'a> {
    #[allow(dead_code)]
    compatible: String,
    is_32: bool,
    data_reg : &'a mut AtomicU8,
    flag_reg : &'a mut AtomicU8,
    data_reg32 : &'a mut AtomicU32,
    flag_reg32 : &'a mut AtomicU32,
}

pub const BROADCOM_BCM2835 : &str = "brcm,bcm2835-aux-uart";
pub const NS16550 : &str = "ns16550a";
pub const DESIGNWARE : &str = "snps,dw-apb-uart";
pub const PL011 : &str = "arm,pl011";

impl NS16550Output<'_> {

    pub fn new(compatible: String, mmio_base: u64, devt: &DeviceTree, node: &DevTreeIndexNode) -> Option<Box<dyn Logger>> {
        Some(Box::new(
            unsafe {
                let mut reg_io: u32 = 1;
                let mut reg_shift: u32 = 1;
                if compatible.eq_ignore_ascii_case(BROADCOM_BCM2835) {
                    reg_shift = 4;
                } else if compatible.eq_ignore_ascii_case(DESIGNWARE) {
                    let reg_io_prop = devt.get_prop_by_name(&node, "reg-io-width").unwrap();
                    reg_io = reg_io_prop.u32(0).unwrap();
                    let reg_shit_prop = devt.get_prop_by_name(&node, "reg-shift").unwrap();
                    reg_shift = reg_shit_prop.u32(0).unwrap();
                }
                let driver = NS16550Output {
                    compatible,
                    is_32: reg_io == 4, // by default, registers are 8 bits
                    data_reg: AtomicU8::from_mut(&mut *(mmio_base as *mut u8)),
                    flag_reg: AtomicU8::from_mut(&mut *((mmio_base + (5 << reg_shift)) as *mut u8)),
                    data_reg32: AtomicU32::from_mut(&mut *(mmio_base as *mut u32)),
                    flag_reg32: AtomicU32::from_mut(&mut *((mmio_base + (5 << reg_shift)) as *mut u32)),
                };
                early_prints!("ns16550 driver reg_io: % \n", reg_io as u64);
                early_prints!("ns16550 driver reg_shift: % \n", reg_shift as u64);
                early_prints!("ns16550 driver data_reg: % \n", mmio_base);
                early_prints!("ns16550 driver flag_reg: % \n", mmio_base + (5 << reg_shift));
                driver
            }
        ))
    }

    pub fn from_mmio(compatible: &str, mmio_base: u64, reg_io: u32, reg_shift: u32) -> Option<Box<dyn Logger>> {
        Some(Box::new(
            unsafe {
                let driver = NS16550Output {
                    compatible: compatible.to_owned(),
                    is_32: reg_io == 4, // by default, registers are 8 bits
                    data_reg: AtomicU8::from_mut(&mut *(mmio_base as *mut u8)),
                    flag_reg: AtomicU8::from_mut(&mut *((mmio_base + (5 << reg_shift)) as *mut u8)),
                    data_reg32: AtomicU32::from_mut(&mut *(mmio_base as *mut u32)),
                    flag_reg32: AtomicU32::from_mut(&mut *((mmio_base + (5 << reg_shift)) as *mut u32)),
                };
                driver
            }
        ))
    }
    
    pub fn is_compatible(mut candidates: StringPropIter) -> Option<String> {
        while let Some(s) = candidates.next().unwrap() {
            if s.eq(NS16550) {
                return Some(String::from(NS16550));
            }
            else if s.eq(BROADCOM_BCM2835) {
                return Some(String::from(BROADCOM_BCM2835));
            } 
            else if s.eq(DESIGNWARE) {
                return Some(String::from(DESIGNWARE));
            }
        }
        return None;
    }
}

impl fmt::Write for NS16550Output<'_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            if self.is_32 {
                loop {
                    let flags = self.flag_reg32.load(Ordering::Acquire);
                    if flags & 1<<5 != 0 { break; }
                    hint::spin_loop();
                }
                self.data_reg32.store(c as u32, Ordering::Release);
            }
            else {
                loop {
                    let flags = self.flag_reg.load(Ordering::Acquire);
                    if flags & 1<<5 != 0 { break; }
                    hint::spin_loop();
                }
                self.data_reg.store(c as u8, Ordering::Release);
            }
        }
        Ok(())
    }
}

impl TTY for NS16550Output<'_> {
    fn get_unprinted(&self) -> String {
        return String::from("");
    }
}

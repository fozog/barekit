/* 
    SPDX-License-Identifier: Mozilla Public License 2.0
    Copyrigth (c) 2022-2023 François-Frédéric Ozog
    
*/

use core::{fmt, sync::atomic::{AtomicU32, Ordering}, hint};
use alloc::string::String;
use alloc::boxed::Box;
use crate::alloc::borrow::ToOwned;

use fdt_rs::common::prop::StringPropIter;
use fdt_rs::prelude::FallibleIterator;

use crate::log::TTY;
use crate::log::Logger;

#[allow(dead_code)]
pub const PL011 : &str = "arm,pl011";

pub struct PL011Output<'a> {
    pub compatible: String,
    pub data_reg : &'a mut AtomicU32,
    pub flag_reg : &'a mut AtomicU32
}

impl PL011Output<'_> {

    pub fn new(compatible: String, mmio_base: u64) -> Option<Box<dyn Logger>> {
        Some(Box::new(
            unsafe {
                PL011Output {
                    compatible,
                    data_reg: AtomicU32::from_mut(&mut *(mmio_base as *mut u32)),
                    flag_reg: AtomicU32::from_mut(&mut *((mmio_base + 0x18) as *mut u32)),
                }
            }
        ))
    }
    #[allow(dead_code)]
    pub fn from_mmio(compatible: &str, mmio_base: u64, _reg_io: u32, _reg_shift: u32) -> Option<Box<dyn Logger>> {
        Some(Box::new(
            unsafe {
                let driver = PL011Output {
                    compatible: compatible.to_owned(),
                    data_reg: AtomicU32::from_mut(&mut *(mmio_base as *mut u32)),
                    flag_reg: AtomicU32::from_mut(&mut *((mmio_base + 0x18) as *mut u32)),
                };
                driver
            }
        ))
    }
    pub fn is_compatible(mut candidates: StringPropIter) -> Option<String> {
        while let Some(s) = candidates.next().unwrap() {
            if s.eq("arm,pl011") {
                return Some(String::from("arm,pl011"));
            }
            else if s.eq("arm,sbsa-uart") {
                return Some(String::from("arm,sbsa-uart"));
            } 
        }
        return None;
    }
}

impl fmt::Write for PL011Output<'_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {

            loop {
                let flags = self.flag_reg.load(Ordering::Acquire);
                if flags & 1<<4 != 0 { break; }
                hint::spin_loop();
            }
            self.data_reg.store(c as u32, Ordering::Release);
        }
        Ok(())
    }
}

impl TTY for PL011Output<'_> {
    fn get_unprinted(&self) -> String {
        return String::from("");
    }
}

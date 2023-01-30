/* 
    SPDX-License-Identifier: Mozilla Public License 2.0
    Copyrigth (c) 2022-2023 François-Frédéric Ozog
    
*/

use core::{fmt, sync::atomic::{AtomicU8, AtomicU32, Ordering}, hint};
use core::mem::size_of;
use alloc::string::String;
use alloc::boxed::Box;

use crate::log::TTY;
use crate::log::Logger;

use fdt_rs::common::prop::StringPropIter;
use fdt_rs::prelude::FallibleIterator;

pub struct NS16550Output<'a> {
    compatible: String,
    is_32: bool,
    data_reg : &'a mut AtomicU8,
    flag_reg : &'a mut AtomicU8,
    data_reg32 : &'a mut AtomicU32,
    flag_reg32 : &'a mut AtomicU32,
}

impl NS16550Output<'_> {

    pub fn new(compatible: String, mmio_base: u64) -> Option<Box<dyn Logger>> {
        Some(Box::new(
            unsafe {
                let mut driver = NS16550Output {
                    compatible,
                    is_32: false, // by default, registers are 8 bits
                    data_reg: AtomicU8::from_mut(&mut *(mmio_base as *mut u8)),
                    flag_reg: AtomicU8::from_mut(&mut *((mmio_base + 5 * (size_of::<u8>() as u64)) as *mut u8)),
                    data_reg32: AtomicU32::from_mut(&mut *(mmio_base as *mut u32)),
                    flag_reg32: AtomicU32::from_mut(&mut *((mmio_base + 5 * (size_of::<u32>() as u64)) as *mut u32)),
                };
                if driver.compatible.eq_ignore_ascii_case("brcm,bcm2835-aux-uart") {
                    driver.is_32 = true;
                }
                driver
            }
        ))
    }
    
    pub fn is_compatible(mut candidates: StringPropIter) -> Option<String> {
        while let Some(s) = candidates.next().unwrap() {
            if s.eq("ns16550a") {
                return Some(String::from("ns16550a"));
            }
            else if s.eq("brcm,bcm2835-aux-uart") {
                return Some(String::from("brcm,bcm2835-aux-uart"));
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

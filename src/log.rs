/* 
    SPDX-License-Identifier: Mozilla Public License 2.0
    Copyrigth (c) 2022-2023 François-Frédéric Ozog
    
*/

use alloc::{boxed::Box, string::String};

pub trait TTY {
    fn get_unprinted(&self) -> String;
 }
 
pub mod interface {
    pub use core::fmt::Write;
}

pub trait Logger: core::fmt::Write + TTY {}

// blanket implementation
impl<TargetLogger: core::fmt::Write + TTY> Logger for TargetLogger {}

// that variable happen to be NOT initialized
static mut TTY_TARGET: Option<Box<dyn Logger>> = None;


pub fn get_unprinted() -> String {
    unsafe {
        let target = TTY_TARGET.as_mut().unwrap();
        target.get_unprinted()
    }
}

pub fn set_target(target: Option<Box<dyn Logger>>) {
    unsafe {
        TTY_TARGET = target
    };
}

pub fn tty() -> &'static mut Option<Box<dyn Logger>>{
    unsafe {&mut TTY_TARGET}
}
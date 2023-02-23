/* 
    SPDX-License-Identifier: Mozilla Public License 2.0
    Copyrigth (c) 2022-2023 François-Frédéric Ozog
    
*/
use core::fmt;

use alloc::{string::{String}};

use crate::log::TTY;

use r_efi::efi;

pub struct TTYEFI<'a> {
    pub sys_tab: &'a efi::SystemTable
}

impl fmt::Write for TTYEFI<'_> {
    
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            let mut dst: [u16; 2] = [0, 0];
            c.encode_utf16(&mut dst);
            unsafe {
                ((*(self.sys_tab.con_out)).output_string) (self.sys_tab.con_out, dst.as_mut_ptr());
            }
        }
        Ok(())
    }
    
}

impl TTY for TTYEFI<'_> {
    fn get_unprinted(&self) -> String {
        return String::from("");
    }
}

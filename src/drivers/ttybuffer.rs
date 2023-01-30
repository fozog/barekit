/* 
    SPDX-License-Identifier: Mozilla Public License 2.0
    Copyrigth (c) 2022-2023 François-Frédéric Ozog
    
*/

use core::fmt;

use alloc::{string::{String, ToString}};

use crate::log::TTY;

pub struct TTYBuffer<'a> {
    pub buffer: &'a mut[u8],
    pub current: usize,
    pub count: usize
}

impl fmt::Write for TTYBuffer<'_> {
    
    fn write_str(&mut self, s: &str) -> fmt::Result {
        
        for c in s.chars() {
            if self.count >= 4096 {
                break;
            }
            self.count += 1;
            self.buffer[self.current]=c as u8;
            self.current +=1;
        }
        Ok(())
    }
    
}

impl TTY for TTYBuffer<'_> {
    fn get_unprinted(&self) -> String {
        let v = &self.buffer[0..self.count];
        let result = String::from_utf8_lossy(v).to_string();
        return result;
    }
}

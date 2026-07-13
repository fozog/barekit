/* 
    SPDX-License-Identifier: Mozilla Public License 2.0
    Copyrigth (c) 2022-2023 François-Frédéric Ozog
    
*/

use core::fmt;
use core::slice;

use alloc::{string::{String, ToString}};

use crate::log::TTY;

pub struct TTYBuffer {
    pub buffer: *mut u8,
    pub capacity: usize,
    pub current: usize,
    pub count: usize
}

impl fmt::Write for TTYBuffer {
    
    fn write_str(&mut self, s: &str) -> fmt::Result {
        
        for c in s.chars() {
            if self.count >= self.capacity {
                break;
            }
            self.count += 1;
            unsafe {
                self.buffer.add(self.current).write(c as u8);
            }
            self.current +=1;
        }
        Ok(())
    }
    
}

impl TTY for TTYBuffer {
    fn get_unprinted(&self) -> String {
        let n = core::cmp::min(self.count, self.capacity);
        let v = unsafe { slice::from_raw_parts(self.buffer as *const u8, n) };
        let result = String::from_utf8_lossy(v).to_string();
        return result;
    }
}

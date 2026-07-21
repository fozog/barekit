/* 
    SPDX-License-Identifier: Mozilla Public License 2.0
    Copyrigth (c) 2022-2023 François-Frédéric Ozog
    
*/

#[allow(unused_imports)]
use crate::{log};
use core::fmt;

#[cfg(feature = "early_print")]
use core::hint;
#[cfg(feature = "early_print")]
use core::arch::asm;

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    let tty = log::tty();
    tty.as_mut().unwrap().write_fmt(args).unwrap();
}

#[allow(dead_code)]
static mut PL011_QEMU:  *mut u8 = 0x0900_0000 as *mut u8;
#[allow(dead_code)]
static mut NS16550_LKVM:  *mut u8 = 0x100_0000 as *mut u8;
#[allow(dead_code)]
static mut NS16550_RPI4:  *mut u32 = 0xfe21_5040 as *mut u32;
#[allow(dead_code)]
static mut PL011_HONEYCOMB:  *mut u32 = 0x21c_0000 as *mut u32;
#[allow(dead_code)]
static mut PL011_SYNQUACER:  *mut u32 = 0x2a40_0000 as *mut u32;
#[allow(dead_code)]
static mut NS6550_MACCHIATOBIN:  *mut u8 = 0xf051_2000 as *mut u8;

#[allow(dead_code)]
//SETUP: choose default serial, look for SETUP to know all places to change values
//static mut RRT0_PORT: *mut u32 = 0xfe21_5040 as *mut u32;
//static mut RRT0_PORT: *mut u32 = 0x21c_0000 as *mut u32;
static mut RRT0_PORT: *mut u8 = 0x0900_0000 as *mut u8;
//static mut RRT0_PORT: *mut u8 = 0x100_0000 as *mut u8;
// Solidrun Macchiatobin
//static mut RRT0_PORT: *mut u8 = 0xf051_2000 as *mut u8;

#[doc(hidden)]
#[allow(dead_code)]
#[cfg(feature = "early_print")]
pub fn _early_putc(c: char) {
    unsafe {
        asm!(
            "str {value:w}, [{port}]",
            port = in(reg) (RRT0_PORT as *mut u32),
            value = in(reg) (c as u32),
            options(nostack, preserves_flags)
        );
    }
    // bad hack to avoid overloading real HW... 
    // real driver are polling for an appropriate time to send chars...
    for _i in 0..9000 {
        hint::spin_loop();
    }
    if c == '\n' {
        unsafe {
            asm!(
                "str {value:w}, [{port}]",
                port = in(reg) (RRT0_PORT as *mut u32),
                value = in(reg) ('\r' as u32),
                options(nostack, preserves_flags)
            );
        }
        for _i in 0..9000 {
            hint::spin_loop();
        }
    }
}

#[doc(hidden)]
#[allow(dead_code)]
#[cfg(feature = "early_print")]
pub fn _early_print_x(value: u64) {
    let conv = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f'];
    _early_putc('0');
    _early_putc('x');
    for i in (0..16).rev() {
        let n = (value >> (4*i)) & 0xF;
        let c = conv[n as usize];
        _early_putc(c);
    }
}

#[doc(hidden)]
#[allow(dead_code)]
#[cfg(feature = "early_print")]
pub fn _early_print_a(value: u64) {
    unsafe {
        let mut asciiz = value as *const u8;
        for _i in 0..256 {
            let c = *asciiz;
            if c != 0 {
                _early_putc(c as char);
                asciiz = asciiz.add(1);
            }
            else {
                break;
            }
        }
    }
}

#[doc(hidden)]
#[allow(dead_code)]
#[cfg(feature = "early_print")]
pub fn _early_print_s(format:&str, value: u64) {
    for c in format.chars() {
        if c=='%' {
            _early_print_x(value);
        }
        else if c=='$' {
            _early_print_a(value);
        }
        else {
            _early_putc(c);
        }
    }
}

#[macro_export]
#[cfg(not(feature = "early_print"))]
macro_rules! early_prints {
    ($a:expr,$b:expr) => ({})
}

#[macro_export]
#[cfg(feature = "early_print")]
macro_rules! early_prints {
    ($a:expr,$b:expr) => (_early_print_s($a, $b));
}

/// Prints without a newline.
///
/// Carbon copy from <https://doc.rust-lang.org/src/std/macros.rs.html>
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::print::_print(format_args!($($arg)*)));
}


/// Prints with a newline.
///
/// Carbon copy from <https://doc.rust-lang.org/src/std/macros.rs.html>
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($fmt:expr) => ($crate::print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::print!(concat!($fmt, "\n"), $($arg)*));
}


/* 
    SPDX-License-Identifier: Mozilla Public License 2.0
    Copyrigth (c) 2022-2023 François-Frédéric Ozog
    
*/

use alloc::boxed::Box;

use crate::PlatformOperations;
use crate::println;

pub fn run(_platform:&mut Box<dyn PlatformOperations>) -> i64 {
    println!("Hello World!");
    return 0;
}
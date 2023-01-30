/* 
    SPDX-License-Identifier: Mozilla Public License 2.0
    Copyrigth (c) 2022-2023 François-Frédéric Ozog
    
*/

use alloc::boxed::Box;

pub mod el_1_2;
pub mod el_3;
pub mod s_el_1;
pub mod efi;

use crate::RuntimeContext;
use crate::println;
use crate::print;
use crate::ALLOC_SIZE;
use crate::ALLOC_COUNT;

use crate::platforms;


//#[cfg(feature = "early_print")]
//use crate::print::_early_print_s;

pub struct PlatformInfo {
    pub image_base:             u64,
    pub image_end:              u64,
    pub boot_stack_top:         u64,
    pub boot_stack_capacity:    usize,
    pub boot_heap_base:         u64,
    pub boot_heap_capacity:     usize,
    pub runtime_context:        RuntimeContext,
    pub x0_at_startup:          u64,
    pub x1_at_startup:          u64,
    pub x2_at_startup:          u64,
    pub x3_at_startup:          u64
}

pub struct Platform {}

impl Platform {

    pub fn new_from(information: PlatformInfo) -> Box<dyn PlatformOperations<'static>> {
        
        let platform: Box<dyn PlatformOperations> = 
            match information.runtime_context {                
                RuntimeContext::BareMetalEL1    => Box::from(platforms::el_1_2::Platform::new(information)),
                RuntimeContext::BareMetalEL3    => Box::from(platforms::el_3::Platform::new(information)),
                RuntimeContext::BaremetalSEL1   => Box::from(platforms::s_el_1::Platform::new(information)),
                RuntimeContext::BaremetalEL2    => Box::from(platforms::el_1_2::Platform::new(information)),
                RuntimeContext::EFI             => Box::from(platforms::efi::Platform::new(information)),
            };
        platform
    }

}

pub trait PlatformOperations<'a> {

    fn get_fdt_address(&self) -> Option<u64>;

    fn pre_stop(&self) {
        unsafe {
            //asm!("brk #1");
            println!("BumpAllocator stats: {} allocations, {} bytes.", ALLOC_COUNT, ALLOC_SIZE);
        }
    }

    fn stop(&self) {

    }

    fn set_boot_tty(&mut self) {

    }

    fn can_return(&self) -> bool {
        return false;
    }
    
    fn park(&self) {
        print!("Looping forever()");
        loop{}
    }

    fn get_info(&self) -> &PlatformInfo;


}

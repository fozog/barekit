/* 
    SPDX-License-Identifier: Mozilla Public License 2.0
    Copyrigth (c) 2022-2023 François-Frédéric Ozog
    
*/

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

pub mod el_1_2;
pub mod el_3;
pub mod s_el_1;
pub mod efi;

use crate::RuntimeContext;
use crate::println;
use crate::print;
use crate::ALLOC_SIZE;
use crate::ALLOC_COUNT;
use crate::early_prints;

use crate::platforms;
use crate::dt::DeviceTree;

use fdt_rs::index::DevTreeIndexNode;
use fdt_rs::prelude::PropReader;
use fdt_rs::prelude::FallibleIterator;

#[cfg(feature = "early_print")]
use crate::print::_early_print_s;

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

    pub fn get_stdout<'a>(devt: &'a Box<DeviceTree<'a>>, base: &'a str) -> Option<DevTreeIndexNode<'a, 'a, 'a>> {
        // now setup the console
        early_prints!("console stuff\n",0);
        let chosen_node= devt.get_node_by_name(base).unwrap();
        early_prints!("chosen\n",0);
        let stdout_prop = devt.get_prop_by_name(&chosen_node, "stdout-path");
        let mut stdout = "serial0"; // lets have a default... (needed for RPI4 ;-)
        if let Some(p) = stdout_prop {
            early_prints!("stdout-path is set\n",0);
            stdout = p.iter_str().next().unwrap().unwrap();
            println!("stdout={}", stdout);
            early_prints!("stdout-path=$\n", stdout.as_ptr() as u64);
        }
        else {
            early_prints!("stdout-path is not set, trying default=serial0\n",0);
        }
        let full_path = String::from(stdout);
        let s = full_path.split(":");
        let vec: Vec<&str> = s.collect();
        devt.get_node_by_path(vec[0])
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

    fn get_name(&self) -> &str {
        "baremetal"
    }
    
    fn get_info(&self) -> &PlatformInfo;
    
    fn set_devt(&'a mut self, devt: Option<Box<DeviceTree<'a>>>);

    fn is_secure(&self) -> bool {
        return false;
    }

}

/* 
    SPDX-License-Identifier: Mozilla Public License 2.0
    Copyrigth (c) 2022-2023 François-Frédéric Ozog
    
*/

#![no_std]
#![no_main]
#![feature(format_args_nl)]
#![feature(panic_info_message)]
#![feature(allocator_api)]
#![feature(alloc_error_handler)]
#![feature(atomic_from_mut)]
#![feature(strict_provenance)]
#![feature(exclusive_range_pattern)]

extern crate alloc;

use core::ffi;
use core::arch::asm;

use alloc::boxed::Box;

use r_efi::system::{ALLOCATE_ANY_PAGES, LOADER_DATA};
use r_efi::efi::{self, PhysicalAddress};
use peview::header::{DosHeader, NtHeader};

use crate::drivers::TTYBuffer;
use crate::heap::{ALLOC_SIZE, ALLOC_COUNT};
use crate::platforms::{Platform, PlatformInfo, PlatformOperations};
use crate::rrt1::rrt1_entry;

#[cfg(feature = "early_print")]
use crate::print::_early_print_s;

mod drivers;
mod log;
mod print;
mod heap;
mod rrt1;
mod dt;
mod platforms;
mod run;
mod coff_stager;
mod processor;

#[derive(PartialEq)]
pub enum RuntimeContext {
    BareMetalEL3,
    BaremetalEL2,
    BareMetalEL1,
    BaremetalSEL1,
    EFI
}

static mut TTY_BUFFER :[u8; 4096] = [0; 4096];

// the following values can't be changed as they are defined by EFI standard
const EFI_PAGE_SIZE : u64 = 4096;
const EFI_PAGE_MASK : u64 = 0xFFF;

const BOOT_HEAP_SIZE : usize = 512*1024 ;

#[export_name = "entry"]
#[allow(const_item_mutation)]
#[allow(unused_assignments)]
pub extern "C"  fn rrt0_entry(x0: u64, x1: u64, x2: u64, x3: u64, x4: u64, x5: u64) -> i64 {

    /* this block MUST be the first things to do. you may change it if you 
       fully undestand PE/COFF relocation stuff and the Allocator internals. */
    /*
    Test block for SoC simulation in hvftool and kvmtool
    let mut _vbar_el1: u64 = 0x1234;
    unsafe {
        //asm!("msr vbar_el1, {}", in(reg) _vbar_el1);
        //asm!("msr sctlr_el2, {}", in(reg) _vbar_el1);
        //asm!("msr ID_AA64PFR0_EL1, {}", in(reg) _vbar_el1);
        //asm!("mrs {}, ID_AA64PFR0_EL1", out(reg) _vbar_el1);
        //asm!("msr ttbr1_el1, {}", in(reg) _vbar_el1);
        //asm!("smc #1");
        //asm!("hvc #1");
        //asm!("tlbi alle1");
        //asm!("wfe");
        //asm!("wfi");
        //asm!("ldp x0, x1, [x2]");
    }
    */
    let mut rc = RuntimeContext::BareMetalEL1;
    {
        early_prints!("\n\n\n-------------------------------------------------\nrrt0_entry()\n", 0);
        early_prints!("x0=%\n", x0);
        early_prints!("x1=%\n", x1);
        early_prints!("x2=%\n", x2);
        early_prints!("x3=%\n", x3);
        early_prints!("x4=%\n", x4);
        early_prints!("x5=%\n", x5);
        let exception_level = processor::get_current_el();
        early_prints!("EL=%\n", exception_level as u64);
        if exception_level == 3 {
            rc = RuntimeContext::BareMetalEL3;
        }
        else if exception_level == 2 {
            if x1 != 0 && x0 != 0 {
                rc = RuntimeContext::EFI;
            }
            else {
                rc = RuntimeContext::BaremetalEL2;
            }
        }
        else {
            if x1 == 0xF1F0 {
                rc = RuntimeContext::BaremetalSEL1
            } else if x1 !=0 && x0 != 0 {
                rc = RuntimeContext::EFI
            } else {
                rc = RuntimeContext::BareMetalEL1
            }
        }

        if rc != RuntimeContext::EFI {
            // this is bare metal entry
            early_prints!("Baremetal entry, self relocating...", 0);
            let image_base = x4;
            let image_end = x5; 
            unsafe {
                let _res = coff_stager::relocate(image_base as usize, image_end as usize);
            }
            early_prints!("done.\n", 0);
        }
    }

    let load_address: u64;
    let mut end_of_image: u64;
    let start_of_heap: u64;
    let mut end_of_stack: u64;
    
    if rc == RuntimeContext::EFI {
        // this is UEFI entry
        let sys_tab: &efi::SystemTable = unsafe { &*(x1 as * mut efi::SystemTable)};
        let boot_services: &efi::BootServices;
        // find the load_address
        //let mut loaded_image: *mut efi::protocols::loaded_image::Protocol = core::ptr::null_mut();
        early_prints!("EFI entry\n", 0);
        let mut pimage: *mut ffi::c_void = core::ptr::null_mut();
        unsafe {
            boot_services = &*sys_tab.boot_services;
            (boot_services.handle_protocol)(x0 as efi::Handle, & mut efi::protocols::loaded_image::PROTOCOL_GUID, & mut pimage);
            let  loaded_image: *mut efi::protocols::loaded_image::Protocol = pimage as *mut r_efi::protocols::loaded_image::Protocol;
            load_address = (*loaded_image).image_base as u64;
            end_of_image = (*loaded_image).image_size as u64 + load_address;
            end_of_image = (end_of_image + (EFI_PAGE_SIZE - 1)) & !EFI_PAGE_MASK;
            let mut heapbase : PhysicalAddress = 0;
            let heapptr: *mut PhysicalAddress= &mut heapbase;
        let r = (boot_services.allocate_pages)(ALLOCATE_ANY_PAGES, LOADER_DATA, BOOT_HEAP_SIZE / EFI_PAGE_SIZE as usize, heapptr);
            if r.is_error() {
                early_prints!("Could not allocated HEAP\n", 0);
                return -1;
            }
            else {
                early_prints!("HEAP successfully allocated in EFI\n", 0);
            }
            start_of_heap = heapbase as u64;
            asm!(
                "mov {x}, sp",
                x = out(reg) end_of_stack
            );
            // let's assume the top of the stack is the end of the current page
            end_of_stack = (end_of_stack + (EFI_PAGE_SIZE - 1)) & !EFI_PAGE_MASK;
        }
    }
    else {
        load_address = x4;
        end_of_image = x5;
        unsafe {
            asm!(
                "mov {x}, sp",
                x = out(reg) end_of_stack
            );
            let dos_header: &DosHeader = &*(load_address as * const DosHeader);
            let nt_header: &NtHeader = &*((load_address + (dos_header.e_lfanew as u64)) as * const NtHeader);
            let stack_align = nt_header.optional_header.section_alignment as u64;
            end_of_stack = (end_of_stack + (stack_align - 1)) & !(stack_align-1);
        }        
        start_of_heap = end_of_stack;
    }

    // HEAP preparation
    early_prints!("HEAP to be initialized at %\n", start_of_heap as u64);
    heap::heap_init(start_of_heap as usize, BOOT_HEAP_SIZE);
    early_prints!("HEAP initialized with % bytes\n", BOOT_HEAP_SIZE as u64);

    /*  Now that the boot HEAP is available to create Rust structs and things,
        we can use "object orientation" for clearer logic
    */

    let tty_earlydev = unsafe {
        TTYBuffer{
            buffer: &mut TTY_BUFFER, current: 0, count:0
        }
    };

    early_prints!("about to set tty_earlydev, TTY_BUFFER at %...", unsafe {TTY_BUFFER.as_ptr() as u64});
    log::set_target(Some(Box::new(tty_earlydev)));
    early_prints!("done.\n", unsafe {TTY_BUFFER.as_ptr() as u64});

    let information = PlatformInfo { 
        image_base: load_address, 
        image_end: end_of_image,
        boot_stack_top: end_of_stack, 
        boot_stack_capacity: 65536,
        boot_heap_base: start_of_heap,
        boot_heap_capacity: BOOT_HEAP_SIZE,
        runtime_context: rc,
        x0_at_startup: x0,
        x1_at_startup: x1,
        x2_at_startup: x2,
        x3_at_startup: x3
    };

    let mut platform: Box<dyn PlatformOperations> = Platform::new_from(information);

    platform.set_boot_tty();

    rrt1_entry(platform)

}

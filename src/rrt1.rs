/* 
    SPDX-License-Identifier: Mozilla Public License 2.0
    Copyrigth (c) 2022-2023 François-Frédéric Ozog
    
*/

use core::panic::PanicInfo;
use core::hint;

use alloc::boxed::Box;

use crate::drivers::NS16550Output;
use crate::drivers::PL011Output;
use crate::dt;
use crate::log;
use crate::log::Logger;

use crate::platforms::PlatformOperations;
use crate::println;
use crate::dt::DeviceTree;
use crate::dt::read_two_items;
use crate::run::run;

use fdt_rs::base::DevTree;
use fdt_rs::index::DevTreeIndex;
use fdt_rs::prelude::PropReader;
use fdt_rs::error::DevTreeError;

#[cfg(feature = "early_print")]
use crate::print::_early_print_s;
use crate::early_prints;

#[cfg(feature = "early_print")]
#[panic_handler]
#[no_mangle]
fn on_panic(_info: &PanicInfo) -> ! 
{
    let message: Option<&core::fmt::Arguments> = _info.message();
    if let Some(location) = _info.location() {
        early_prints!("\nPanic at $ ", location.file().as_ptr() as u64 );
        early_prints!("line %\n", location.line() as u64);    
    }
    println!("Panic!!");
    println!("{:?}", message);
    
    loop {
        hint::spin_loop();
    }
}

//This will be rust_begin_unwind symbol
#[cfg(not(feature = "early_print"))]
#[panic_handler]
fn on_panic(_info: &PanicInfo) -> ! 
{
    let message: Option<&core::fmt::Arguments> = _info.message();
    println!("Panic!!");
    println!("{:?}", message);
    
    loop {
        hint::spin_loop();
    }
}
static mut SCRATCHPAD: [u8; 132768] = [0; 132768];

#[allow(dead_code)]
pub  fn rrt1_entry(mut platform: Box<dyn PlatformOperations>) -> i64 
{
    // we have platform ownership here
    early_prints!("rr1_entry()\n", 0);

    let information = platform.get_info();

    println!(r#"Hello from Rust Runtime phase 1: {}."#, platform.get_name());
    println!("    Load address  = {:#x}", information.image_base);
    println!("    End of image  = {:#x}", information.image_end);
    println!("    End of stack  = {:#x}", information.boot_stack_top);
    println!("    Start of Heap = {:#x}", information.boot_heap_base);


    // handle DeviceTree here as I can't get that thing done in PlatformOperations default implementation
    // Rust lifetime issues...
    //TODO: make this properly!

    early_prints!("About to handle FDT\n", 0);
    let mut fdt:u64 =0;
    if let Some(fdt_address) = platform.get_fdt_address() {
        fdt =fdt_address;
    }

    early_prints!("FDT @%\n", fdt);

    if fdt == 0 {
        early_prints!("Use default platform TTY as FDT is not known\n", 0);
        platform.set_boot_tty();
    }
    else {

        let devt: Box<DeviceTree> = Box::new(unsafe {

            if fdt as usize & 3 !=0 {
                panic!("Invalid FDT address, need to be 32bits aligned!");
            }
        
            early_prints!("Create from raw pointer...", 0);
            let result = DevTree::from_raw_pointer(fdt as *const u8);
            let fdt : DevTree;
            match result {
                Ok(dt) => fdt =dt,
                Err(why) => {
                    match why {
                        DevTreeError::InvalidParameter(_p) => early_prints!("InvalidParameter\n", _p.as_ptr() as u64),
                        DevTreeError::InvalidMagicNumber => early_prints!("InvalidMagicNumber\n", 0),
                        DevTreeError::InvalidOffset => early_prints!("InvalidOffset\n", 0),
                        DevTreeError::ParseError => early_prints!("ParseError\n", 0),
                        DevTreeError::StrError(_) =>early_prints!("StrError\n", 0),
                        DevTreeError::NotEnoughMemory => early_prints!("NotEnoughMemory\n", 0),
                    }
                    panic!("could not wrap FDT: {:?}", result);
                }
            }

            early_prints!("\n", 0);
            
            let slice = SCRATCHPAD.as_mut_slice();
            let index = DevTreeIndex::new(fdt, slice).unwrap();
            early_prints!("index done\n", 0);
            DeviceTree::new(fdt, index)
        });

            
        /* should use the following memory information along with additional information from usage
        EFI has a memorymap and baremetal should use PlatformInformation to switch from boot HEAP to full RRT HEAP */
        
        early_prints!("Checking memory stuff\n", 0);
        let mem_node= devt.get_node_by_name("memory").unwrap();
        let reg_prop = devt.get_prop_by_name(&mem_node, "reg").unwrap();
        let reg = read_two_items(reg_prop, devt.acells, devt.scells);
        println!("memmory:");
        for r in reg {
            println!("    {:#012x}-{:#012x}", r.base, r.base + r.size);
        }
        early_prints!("Checking memory reservations entries\n", 0);
        println!("memory reservations:");
        for rsv in devt.devtree.reserved_entries()
        {
            println!("    {:#012x}-{:#012x}", u64::from(rsv.address), u64::from(rsv.address) + u64::from(rsv.size));
        }
        early_prints!("Checking memreserve\n", 0);
        let memreserve_node= devt.get_node_by_path("/").unwrap();
        if let Some(reg_prop) = devt.get_prop_by_name(&memreserve_node, "memreserve") {
            let memreserve = read_two_items(reg_prop, 1, 1);
            for r in memreserve {
                println!("    {:#012x}-{:#012x}", r.base, r.base + r.size);
            }
        }

        let stdout_parent = "chosen";
        //if platform.is_secure() {
        //    stdout_parent = "secure-chosen";
        //}
        let stdout_node= crate::Platform::get_stdout(&devt, stdout_parent);
        let stdout = match stdout_node {
            None => panic!("stdout-path target node not found"),
            Some(ref c) => c
        };

        let _path = dt::to_path(stdout);
        early_prints!("stdout-path from node=$\n", _path.as_ptr() as u64);

        let compatible_prop = devt.get_prop_by_name(stdout, "compatible").unwrap();
        let compatible_strings = compatible_prop.iter_str();


        let mmio = devt.parse_mmio(stdout);
        let m=&mmio[0];
        println!("    mmio={:#012x}-{:#012x}", m.base, m.base+m.size); 
        early_prints!("    mmio base %", m.base);
        early_prints!(" - %\n",  m.base+m.size);

        #[allow(unused_assignments)]
        let mut tty: Option<Box<dyn Logger>> = None;
        

        if let Some(compat) = PL011Output::is_compatible(compatible_strings.clone()) {
            tty =  PL011Output::new(compat, m.base);
            early_prints!("pl011 driver created\n", 0);
        }
        else if let Some(compat) = NS16550Output::is_compatible(compatible_strings.clone()) {
            tty =  NS16550Output::new( compat, m.base, &devt, &stdout);

        } else {
            early_prints!("no driver found\n", 0);
            panic!("No driver found")
        }
        
        early_prints!("About to change the driver\n", 0);
        
        let prev = log::get_unprinted();
        log::set_target(tty);
        println!("{}", &prev);
    } /* fdt vs acpi */

    #[allow(unused_assignments)]
    let mut result : i64 = 0;

    result = run(&platform);

    platform.pre_stop();

    // the following may not return (S-EL1 for instance, or poweroff from EL1)
    platform.stop();

    if platform.can_return() {
        // If makes sense, return to calling environment (U-Boot or EFI).
        // this should not happen if exit_boot_services was called
        // or if hardware has been initialized.
        // Practically very early errors can lead there
        return result;
    }
    else {
        platform.park();
        // never reached    
    }
    
    unsafe {
        core::hint::unreachable_unchecked()
    }

}

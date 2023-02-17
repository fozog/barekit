/* 
    SPDX-License-Identifier: Mozilla Public License 2.0
    Copyrigth (c) 2022-2023 François-Frédéric Ozog
    
*/

use alloc::boxed::Box;
use r_efi::efi::Status;
use r_efi::efi::RESET_COLD;

use crate::PlatformOperations;
use crate::PlatformInfo;
use crate::dt::DeviceTree;

use crate::drivers::TTYEFI;
use crate::log;
use crate::println;

use r_efi::efi;


//#[cfg(feature = "early_print")]
//use crate::print::_early_print_s;

pub const FDT_TABLE_GUID: r_efi::efi::Guid = r_efi::efi::Guid::from_fields(
    0xb1b621d5,
    0xf19c,
    0x41a5,
    0x83,
    0x0b,
    &[0xd9,0x15,0x2c,0x69,0xaa,0xe0],
);


pub struct Platform<'a> {
    _image_handle:          efi::Handle,
    sys_tab:        *const efi::SystemTable,
    information:    PlatformInfo,
    _dt:            Option<Box<DeviceTree<'a>>>
}


impl<'a> Platform<'a>  {

    pub fn new(information: PlatformInfo) -> Self {
        let sys_tab: &efi::SystemTable = unsafe { &*(information.x1_at_startup as * mut efi::SystemTable)};
        let image_handle = information.x0_at_startup  as efi::Handle;
        Self { _image_handle: image_handle, sys_tab, information, _dt: None } 
    }
    
}

impl<'a> PlatformOperations<'a> for Platform<'a> {

    fn get_fdt_address(&self) -> Option<u64> {
        // need to find the address in the EFI tables
        let st = unsafe {&*(self.sys_tab)};
        for i in 0..st.number_of_table_entries {
            let table = unsafe{st.configuration_table.add(i)};
            //println!("table guid={:#?}", unsafe {(*table).vendor_guid});
            let tmp = unsafe {(*table).vendor_guid};
            if FDT_TABLE_GUID.eq(&tmp)  {
                return Some(unsafe {(*table).vendor_table as u64})
            }
        }
        None
    }

    fn set_boot_tty(&mut self) {
        let st = unsafe {&*(self.sys_tab)};
        let tty = TTYEFI{sys_tab: st};
        //early_prints!("about to set tty_earlydev, TTYEFI\n", 0);
        let s = log::get_unprinted();
        log::set_target(Some(Box::new(tty)));
        println!("{}", &s);
    }

    fn get_info(&self) -> &PlatformInfo {
        &self.information
    }

    fn can_return(&self) -> bool {
        // as we dont call exit boot service yet, this is always true
        true
    }

    fn set_devt(&'a mut self, devt: Option<Box<DeviceTree<'a>>>) {
        self._dt = devt;
    }

    fn get_name(&self) -> &str {
        "EFI"
    }

    fn stop(&self) {
        let st = unsafe {&*(self.sys_tab)};
        unsafe {
            ((*(st.runtime_services)).reset_system)(RESET_COLD, Status::from_usize(0), 0, 0 as *mut core::ffi::c_void);
        }
    }

}
/* 
    SPDX-License-Identifier: Mozilla Public License 2.0
    Copyrigth (c) 2022-2023 François-Frédéric Ozog
    
*/

use core::mem::size_of;
use core::ptr::read_unaligned;
use core::slice::ChunksExact;


extern crate alloc;

use alloc::vec::Vec;
use alloc::string::String;


use fdt_rs::base::DevTree;
use fdt_rs::index::DevTreeIndex;
use fdt_rs::prelude::FallibleIterator;
use fdt_rs::prelude::PropReader;
use fdt_rs::index::DevTreeIndexProp;
use fdt_rs::index::DevTreeIndexNode;
use fdt_rs::index::iters::DevTreeIndexNodeIter;

pub struct DeviceTree<'a> {
    pub devtree: DevTree<'a> ,
    pub index: DevTreeIndex<'a, 'a>,
    pub acells: u32,
    pub scells: u32,
    pub interrupt_parent: u32,
    pub compatible: &'a str,
    pub device_type: &'a str 
}

pub struct Region {
    pub base: u64,
    pub size: u64
}
pub struct Translation {
    pub bus_base: u64,
    pub host_base: u64,
    pub size: u64
}

fn read_item( buffer: &mut ChunksExact<u8>, cells: u32) -> u64 {
    if cells == 1 {
        let chunk = buffer.next();
        let v = chunk.unwrap();
        #[allow(clippy::cast_ptr_alignment)]
        unsafe {
            let val = read_unaligned::<u32>(v.as_ptr() as *const u32);
            let val = u32::from_be(val);
            return val as u64;
        }
    }
    else if cells == 2 {
        let mut chunk = buffer.next();
        let mut v = chunk.unwrap();
        let mut high: u32;
        let mut low: u32;
        #[allow(clippy::cast_ptr_alignment)]
        unsafe { high= read_unaligned::<u32>(v.as_ptr() as *const u32); }
        high = u32::from_be(high);
        chunk = buffer.next();
        v = chunk.unwrap();
        unsafe { low = read_unaligned::<u32>(v.as_ptr() as *const u32); }
        low = u32::from_be(low);
        return (high as u64) << 32 | (low as u64);
    }
    else {
        panic!("Invalid size passed to read_item {}", cells);
    }
}

pub fn read_two_items(prop: DevTreeIndexProp, acells: u32, scells: u32) -> Vec<Region> {

    let sizeof_reg = ((acells+scells) as usize) * size_of::<u32>() ;
    let count = prop.length() / sizeof_reg;
    let mut result: Vec<Region> =  Vec::with_capacity(count);
    
    if prop.length() %  sizeof_reg!= 0 {
        panic!("reg property of {} does not contain proper amout of data!", prop.name().unwrap());
    }

    let mut chunks= prop.raw().chunks_exact(size_of::<u32>()).into_iter();

    for _c in 0..count
    {
        let base = read_item(&mut chunks, acells);
        let size = read_item(&mut chunks, scells);
        result.push(Region { base, size });
    };

    return result;
}

pub fn to_path(node: &DevTreeIndexNode) -> String {
    let mut result = String::from(node.name().unwrap());
    let mut current: DevTreeIndexNode = node.clone();
    while let Some(parent) = current.parent() {
        result = String::from(parent.name().unwrap()) + "/" + result.as_str();
        current = parent;
    }
    return result;
}

/* the items is assumed to be two element: <name to search> and <name to search@>
for example the memory node can be found as memory or as memory@4000000 */
fn matches_name(node: DevTreeIndexNode, items: &Vec<&str>) -> bool {
    let name = node.name().unwrap();
    return name.eq(items[0]) || name.starts_with(items[1]);
}

fn matches_path(node: DevTreeIndexNode, items: &Vec<&str>) -> bool {
    let binding = to_path(&node);
    let path = binding.as_str();
    return path.eq(items[0]) || path.starts_with(items[1]);
}

fn translate_one(r: Region, translations: &Vec<Translation>) -> Region {
    for t in translations {
        if r.base >= t.bus_base && r.base <= t.bus_base + t.size {
            if r.base+r.size <= t.bus_base + t.size {
                let result = Region {base: t.host_base + (r.base - t.bus_base), size: r.size};
                return result;
            }
        }
    }
    return r;
}

fn translate(reg: Vec<Region>, translations: &Vec<Translation>) -> Vec<Region> {
    let mut result : Vec<Region> = Vec::with_capacity(reg.capacity());
    for r in reg {
        result.push(translate_one(r, translations));
    }
    return result;
}

impl<'a> DeviceTree<'a> {

    pub fn parse_mmio(&self, node: &DevTreeIndexNode) -> Vec<Region> {

        let bus = node.parent().unwrap();
        let bus_acells = self.get_prop_by_name(&bus, "#address-cells").unwrap().u32(0).unwrap();
        let bus_scells = self.get_prop_by_name(&bus, "#size-cells").unwrap().u32(0).unwrap();

        let prop_reg = self.get_prop_by_name(node, "reg").unwrap();
        let reg = read_two_items(prop_reg, bus_acells, bus_scells);

        if let Some(ranges_prop) = self.get_prop_by_name(&bus, "ranges") {
            let sizeof_range = (bus_acells + self.acells + bus_scells) as usize * size_of::<u32>();
            if ranges_prop.length() % sizeof_range != 0 {
                panic!("Invalid range definition, length not a multiple of bus.acells+dt.acells+bus.scells");
            }
            let count = ranges_prop.length() / sizeof_range;
            let mut chunks= ranges_prop.raw().chunks_exact(size_of::<u32>()).into_iter();
            let mut translations: Vec<Translation> =  Vec::with_capacity(count);
            for _c in 0..count {
                let bus_base = read_item(&mut chunks, bus_acells);
                let host_base = read_item(&mut chunks, self.acells);
                let size = read_item(&mut chunks, bus_scells);
                translations.push(Translation { bus_base, host_base, size });
            }
            return translate(reg, &translations); 
        }
        else {
            return reg;
        }
    }
    
        #[allow(dead_code)]
    pub fn get_root(&self) -> DevTreeIndexNode {
        return self.index.root();
    }

    #[allow(dead_code)]
    pub fn get_nodes(&self) -> DevTreeIndexNodeIter {
        return self.index.nodes();
    }

    #[allow(dead_code)]
    pub fn get_node_by_name(&self, name: &str) -> Option<DevTreeIndexNode> {
        let mut search :Vec<&str> = Vec::with_capacity(2);
        search.push(name);
        let mut name2 = String::from(name);
        name2.push_str("@");
        search.push(name2.as_str());
        let mut finder = self.index.nodes().filter(|a: &DevTreeIndexNode| matches_name(a.clone(), &search));
        return finder.next();
    }

    #[allow(dead_code)]
    pub fn get_node_by_path(&self, path: &str) -> Option<DevTreeIndexNode> {
        if path.eq("/") { return Some(self.index.root()) };
        let mut target = path;
        if !path.starts_with("/") {
            let alias_node = self.get_node_by_path("/aliases");
            if let Some(c) = alias_node {
                let p = self.get_prop_by_name(&c, path);
                if let Some(alias) = p {
                    target = alias.iter_str().next().unwrap().unwrap();
                }
                else {
                    return None;
                }
            }
            else {
                return None;
            }
            
        }
        let mut search :Vec<&str> = Vec::with_capacity(2);
        search.push(target);
        let mut target2 = String::from(target);
        target2.push_str("@");
        search.push(target2.as_str());
        let mut finder = self.index.nodes().filter(|a: &DevTreeIndexNode| matches_path(a.clone(), &search));
        return finder.next();
    }

    #[allow(dead_code)]
    pub fn get_prop_by_name<'i, 'dt>(&self, node: &DevTreeIndexNode<'a, 'i, 'dt>, name: &str) -> Option<DevTreeIndexProp<'a, 'i, 'dt>> {
        let mut finder = node.props().filter (|x| x.name().unwrap().eq(name));
        return finder.next();
    }

    pub unsafe fn new(fdt: DevTree<'a>, index: DevTreeIndex<'a, 'a>) -> DeviceTree<'a> {
        
        let mut this = Self { 
            devtree: fdt,
            index:  index,
            acells: 1, scells: 1, interrupt_parent: 0, compatible:"", device_type: ""
            };
            
            for prop in this.index.root().props() {
            let pname = prop.name().unwrap();
            //println!("     prop {}: {} bytes ", pname, prop.length());
            if pname.eq("#address-cells") {
                this.acells=prop.u32(0).unwrap();
                //println!("#address-cells {}", this.acells);
            }
            else if pname.eq("#size-cells") {
                this.scells=prop.u32(0).unwrap();
                //println!("#size-cells {}", this.scells);
            }
            else if pname.eq("interrupt-parent") {
                this.interrupt_parent=prop.phandle(0).unwrap();
            }
            else if pname.eq("compatible") {
                this.compatible = prop.str().unwrap();
            }
            else if pname.eq("device_type") {
                this.device_type = prop.str().unwrap();
            }
        }
        this
    }


}

/* the key issue is to make sure the scratchpad backend lives long enough.
creating the backend in a block or in a function will just drop it after the block.
So we need to "move" the Vec to a variable living long enough.
All attempts to "move" the Vec into the DeviceTree object failed, hence a macro 
that moves key variables in containing block/function */
#[macro_export]
macro_rules! set_dt_from_raw_parts {
    ( $buffer: expr, $scratchpad_backend: expr) => ({
        if $buffer as usize & 3 !=0 {
            panic!("Invalid FDT address, need to be 32bits aligned!");
        }

        let fdt = DevTree::from_raw_pointer($buffer).unwrap();
        
        let layout = DevTreeIndex::get_layout(&fdt).expect("Failed to parse DTB - it is invalid.");

        $scratchpad_backend = vec![0u8; layout.size() + layout.align() ];
        let slice = $scratchpad_backend.as_mut_slice();
        let index = DevTreeIndex::new(fdt, slice).unwrap();
        DeviceTree::new(fdt, index)
    })
}

/* 
    SPDX-License-Identifier: Mozilla Public License 2.0
    Copyrigth (c) 2022-2023 François-Frédéric Ozog
    
*/

use core::alloc::{GlobalAlloc, Layout};

use crate::{println, early_prints};

#[cfg(feature = "early_print")]
use crate::print::_early_print_s;

/// A simple allocator that allocates memory linearly and ignores freed memory.
pub struct BumpAllocator {
    heap_start: usize,
    heap_end: usize,
    next: usize,
}

    /// Align downwards. Returns the greatest x with alignment `align`
/// so that x <= addr. The alignment must be a power of 2.
pub fn align_down(addr: usize, align: usize) -> usize {
    if align.is_power_of_two() {
        addr & !(align - 1)
    } else if align == 0 {
        addr
    } else {
        panic!("`align` must be a power of 2");
    }
}

/// Align upwards. Returns the smallest x with alignment `align`
/// so that x >= addr. The alignment must be a power of 2.
pub fn align_up(addr: usize, align: usize) -> usize {
    align_down(addr + align - 1, align)
}

impl BumpAllocator {
    /// Crate a new UNINITIALIZED heap allocator
    ///
    /// You must initialize this heap using the
    /// [`init`](Self::init) method before using the allocator.
    pub  const fn empty() -> BumpAllocator {
        // the initial next is set to non null to avoid a strange problem
        // if all fields are 0, then the compiler generate a 0 length .data section:
        //0 .text           00004a58 0000000040201000 TEXT
        //1 .rdata          00000f52 0000000040206000 DATA
        //2 .data           00000000 0000000040207000 DATA
        //3 .reloc          000000b4 0000000040208000 DATA
        // while HEAP is precisely located at the begining of the data section.
        // setting the value to 1, correctly reserve the space:
        //  0 .text           00004a58 0000000040201000 TEXT
        //1 .rdata          00000f52 0000000040206000 DATA
        //2 .data           00000018 0000000040207000 DATA
        //3 .reloc          000000b4 0000000040208000 DATA
        BumpAllocator { heap_start: 0, heap_end: 0, next: 0}
    }

    pub fn init(&mut self, heap_start: usize, heap_size: usize)  {
        self.heap_start = heap_start;
        self.heap_end = heap_start+heap_size;
        self.next = heap_start;
    }

}

unsafe impl GlobalAlloc for BumpAllocator {
    
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let alloc_start = align_up(self.next, layout.align());
        let alloc_end = alloc_start.saturating_add(layout.size());
        ALLOC_COUNT += 1;
        ALLOC_SIZE += layout.size();
        //early_prints!("alloc % bytes\n", layout.size() as u64);
        if alloc_end <= self.heap_end {
            HEAP.next = alloc_end;
            //println!("alloc ok @{:#x}", alloc_start);
            return alloc_start as *mut u8;
        } else {
            //println!("alloc problem @{:#x}", alloc_start);
            early_prints!("\n\nOUT OF MEMORY\n", 0);
            return 0 as * mut u8;
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // do nothing, leak memory
    }
}

// The following is accessed in a way that does not depend on relocations
/*
;     HEAP.init(heap_base, 1024*1024);
40202388: e1 73 40 f9   ldr     x1, [sp, #224]
4020238c: 20 00 00 b0   adrp    x0, 0x40207000 <.text+0x13a0>
40202390: 00 00 00 91   add     x0, x0, #0
40202394: 08 02 a0 52   mov     w8, #1048576
40202398: e2 03 08 2a   mov     w2, w8
4020239c: 8d fd ff 97   bl      0x402019d0 <.text+0x9d0>
 */

 #[global_allocator]
static  mut HEAP: BumpAllocator = BumpAllocator::empty();
pub static mut ALLOC_COUNT: u64 = 0;
pub static mut ALLOC_SIZE: usize = 0;

#[alloc_error_handler]
fn oom(_: Layout) -> ! {
    println!("Out of memory");
    loop {}
}

pub fn heap_init(heap_start: usize, heap_size: usize)
{
    unsafe {

        HEAP.init(heap_start, heap_size);
    }
}
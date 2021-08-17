#![feature(lang_items)]
#![feature(ptr_internals)]
#![no_std]

use core::panic::PanicInfo;
use crate::memory::FrameAllocator;

extern crate rlibc;
extern crate multiboot2;
extern crate x86_64;

#[macro_use]
extern crate bitflags;

mod vga_buffer;
mod memory;

#[no_mangle]
pub extern fn rust_main(multiboot_information_address: usize) {
    vga_buffer::clear_screen();
    println!("Hello world{}", "!");

    let boot_info = unsafe {
        multiboot2::load(multiboot_information_address)
    };

    let memory_map_tag = boot_info.memory_map_tag()
        .expect("Memory map tag required");
    
    let mem_info = memory::MemoryInfo::new(multiboot_information_address);
    let mut frame_allocator = memory::Area_frame_allocator::AreaFrameAllocator::new(
        mem_info.get_kernel_start(), mem_info.get_kernel_end(),
        mem_info.get_multiboot_start(), mem_info.get_multiboot_end(),
        memory_map_tag.memory_areas(),
    );

    /*
    println!("{:?}", frame_allocator.allocate_frame());
    
    for i in 0.. {
        if let None = frame_allocator.allocate_frame() {
            println!("allocated {} frames", i);
            break;
        }
    }
    */
    memory::test_paging(&mut frame_allocator);

    loop{}
}

#[lang = "eh_personality"] 
#[no_mangle] 
pub extern fn eh_personality() {}

#[panic_handler] 
#[no_mangle] 
pub extern fn panic_fmt(info: &PanicInfo) -> ! {
    println!("[failed]\n");
    println!("Error: {}\n", info);
    loop{}
}
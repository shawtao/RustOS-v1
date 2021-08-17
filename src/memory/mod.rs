pub mod Area_frame_allocator;
pub mod paging;

pub use self::paging::test_paging;
use self::paging::PhysicalAddress;

pub const PAGE_SIZE: usize = 4096;

pub struct MemoryInfo {
    kernel_start: usize,
    kernel_end: usize,
    multiboot_start: usize,
    multiboot_end: usize,
}

impl MemoryInfo {
    pub fn new(multiboot_information_address: usize) -> MemoryInfo {
        let boot_info = unsafe {
            multiboot2::load(multiboot_information_address)
        };

        /* Option<T>的expect方法可以返回T，None触发panic,打印错误信息 */ 
        let memory_map_tag = boot_info.memory_map_tag()
            .expect("Memory map tag required");
    
        let elf_section_tag = boot_info.elf_sections_tag()
            .expect("Elf-sections tag required");

        let kernel_start = elf_section_tag.sections()
            .map(|s| s.addr).min().unwrap();
        let kernel_end = elf_section_tag.sections()
            .map(|s| s.addr + s.size).max().unwrap();
        let multiboot_start = multiboot_information_address;
        let multiboot_end = multiboot_information_address
            + (boot_info.total_size as usize);
        
        MemoryInfo {
            kernel_start: kernel_start as usize,
            kernel_end: kernel_end as usize,
            multiboot_start,
            multiboot_end,
        }
    }
    
    pub fn get_kernel_start(&self) -> usize {
        self.kernel_start
    }

    pub fn get_kernel_end(&self) -> usize {
        self.kernel_end
    }

    pub fn get_multiboot_start(&self) -> usize {
        self.multiboot_start
    }

    pub fn get_multiboot_end(&self) -> usize {
        self.multiboot_end
    }

}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Frame {
    number: usize,         //similar to pfn(page frame number)
}

impl Frame {
    fn containing_address(address: usize) -> Frame {
        Frame{ number: address/PAGE_SIZE}
    }

    fn start_address(&self) -> PhysicalAddress {
        self.number * PAGE_SIZE
    }
}

pub trait FrameAllocator {
    fn allocate_frame(&mut self) -> Option<Frame>;
    fn deallocate_frame(&mut self, frame: Frame);
}
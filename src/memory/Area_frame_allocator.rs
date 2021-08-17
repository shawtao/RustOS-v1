use crate::memory::{Frame, FrameAllocator};
use multiboot2::{MemoryAreaIter, MemoryArea};

/*
    页框分配器
*/
pub struct AreaFrameAllocator {
    next_free_frame: Frame,                 
    current_area: Option<&'static MemoryArea>,      
    areas:  MemoryAreaIter,             //MemoryArea迭代器，用于找到下一个MemoryArea
    kernel_start: Frame,
    kernel_end: Frame,
    multiboot_start: Frame,
    multiboot_end: Frame,
}

impl FrameAllocator for AreaFrameAllocator {

    fn allocate_frame(&mut self) -> Option<Frame> {
        if let Some(area) = self.current_area {
            let frame = Frame {number: self.next_free_frame.number};

            let current_area_end_frame = {
                let address = area.base_addr + area.length - 1;
                Frame::containing_address(address as usize)
            };

            if frame > current_area_end_frame {
                self.choose_next_area();
            } else if frame >= self.kernel_start && frame <= self.kernel_end {
                self.next_free_frame = Frame {
                    number: self.kernel_end.number + 1
                };
            } else if frame >= self.multiboot_start && frame <= self.multiboot_end {
                self.next_free_frame = Frame {
                    number: self.multiboot_end.number + 1
                };
            } else {
                self.next_free_frame.number += 1;
                return Some(frame);
            }

            return self.allocate_frame();

        } else {
            None
        }
    }

    fn deallocate_frame(&mut self, frame: Frame) {
        unimplemented!()
    }
}

impl AreaFrameAllocator {
    pub fn new(kernel_start: usize, kernel_end: usize, 
        multiboot_start: usize, multiboot_end: usize, 
        memory_areas: MemoryAreaIter) -> AreaFrameAllocator {
         let mut allocator = AreaFrameAllocator {
             next_free_frame: Frame::containing_address(0),
             current_area: None,
             areas: memory_areas,
             kernel_start: Frame::containing_address(kernel_start),
             kernel_end: Frame::containing_address(kernel_end),
             multiboot_start: Frame::containing_address(multiboot_start),
             multiboot_end: Frame::containing_address(multiboot_end),
         };

         allocator.choose_next_area();
         allocator
    }



    fn choose_next_area(&mut self) {
        self.current_area = self.areas.clone().filter(|area| {     //这里需要使用clone克隆迭代器，因为min_by_key会消耗迭代器，从而转移所有权。
            let address = area.base_addr + area.length - 1;
            Frame::containing_address(address as usize) >= self.next_free_frame
        }).min_by_key(|area| area.base_addr);

        if let Some(area) = self.current_area {
            let start_frame = Frame::containing_address(area.base_addr as usize);
            if start_frame > self.next_free_frame {
                self.next_free_frame = start_frame;
            }
        }
    }
}
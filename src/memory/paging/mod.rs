/*
    因为已经进入了长模式开启了分页，所以此时程序中的地址均为虚拟地址，
    访问页表时用得也是虚拟地址，所以首先要将页表映射到虚拟空间中，以便
    程序对页表的访问。
 */
mod entry;
mod table;

use core::ptr::Unique;
use crate::memory::PAGE_SIZE;
use crate::memory::Frame;
use crate::println;
use self::entry::*;
use self::table::*;

const ENTRY_COUNT: usize = 512;

pub type PhysicalAddress = usize;
pub type VirtualAddress = usize;

/*
    struct Page represents a virtual page instead of a physical frame
*/
pub struct Page {
    number: usize,
}

impl Page {
    pub fn containing_address(address: VirtualAddress) -> Page {
        assert!(address < 0x0000_8000_0000_0000 || 
            address >= 0xffff_8000_0000_0000,
            "invalid address: 0x{:x}", address);
        Page {number: address / PAGE_SIZE}
    }

    fn start_address(&self) -> usize {
        self.number * PAGE_SIZE
    }
    
    fn p4_index(&self) -> usize {
        (self.number >> 27) & 0o777
    }

    fn p3_index(&self) -> usize {
        (self.number >> 18) & 0o777
    }

    fn p2_index(&self) -> usize {
        (self.number >> 9) & 0o777
    }

    fn p1_index(&self) -> usize {
        (self.number >> 0) & 0o777
    }

}

pub struct ActivePageTable {
    p4: Unique<Table<Level4>>,
}

impl ActivePageTable {
    pub unsafe fn new() -> ActivePageTable {
        ActivePageTable {
            p4: Unique::new_unchecked(table::P4),
        }
    }

    fn p4(&self) -> &Table<Level4> {
        unsafe { self.p4.as_ref() }
    } 

    fn p4_mut(&mut self) -> &mut Table<Level4> {
        unsafe { self.p4.as_mut()}
    }

    fn translate(&self, virtual_address: VirtualAddress)
    -> Option<PhysicalAddress>
    {
        let offset = virtual_address % PAGE_SIZE;
        self.translate_page(Page::containing_address(virtual_address))
            .map(|frame| frame.number * PAGE_SIZE + offset)
    }

    fn translate_page(&self, page: Page) -> Option<Frame> {
        let p3 = self.p4().next_table(page.p4_index());
    
        p3.and_then(|p3| p3.next_table(page.p3_index()))
        .and_then(|p2| p2.next_table(page.p2_index()))
        .and_then(|p1| p1[page.p1_index()].pointed_frame())
    }

    pub fn map_to<A>(&mut self,page: Page, frame: Frame, flags: EntryFlags,
                        allocator: &mut A)
        where A: super::FrameAllocator
    {
        let mut p3 = self.p4_mut().next_table_create(page.p4_index(), allocator);
        let mut p2 = p3.next_table_create(page.p3_index(), allocator);
        let mut p1 = p2.next_table_create(page.p2_index(), allocator);

        assert!(p1[page.p1_index()].is_unused());
        p1[page.p1_index()].set(frame, flags | PRESENT);
    }

    pub fn map<A>(&mut self, page: Page, flags: EntryFlags, allocator: &mut A)
        where A: super::FrameAllocator
    {
        let frame = allocator.allocate_frame().expect("out of memory");
        self.map_to(page, frame, flags, allocator)
    }

    pub fn identity_map<A>(&mut self, frame: Frame, flags:EntryFlags,
                                allocator: &mut A)
        where A: super::FrameAllocator
    {
        let page = Page::containing_address(frame.start_address());
        self.map_to(page, frame, flags, allocator)
    }

    fn unmap<A>(&mut self, page: Page, allocator: &mut A)
    where A: super::FrameAllocator
    {
        assert!(self.translate(page.start_address()).is_some());

        let p1 = self.p4_mut()
                .next_table_mut(page.p4_index())
                .and_then(|p3| p3.next_table_mut(page.p3_index()))
                .and_then(|p2| p2.next_table_mut(page.p2_index()))
                .expect("mapping code does not support huge pages");
        let frame = p1[page.p1_index()].pointed_frame().unwrap();
        p1[page.p1_index()].set_unused();

        use x86_64::instructions::tlb;
        use x86_64::VirtAddr;
        tlb::flush(VirtAddr::new(page.start_address() as u64));

        // TODO free p(1,2,3) table if empty
        allocator.deallocate_frame(frame);
    }

}

pub fn test_paging<A>(allocator: &mut A)
    where A: super::FrameAllocator
{
    let mut page_table = unsafe { ActivePageTable::new()};
    let addr = 42 * 512 * 512 * 4096;
    let page = Page::containing_address(addr);
    let frame = allocator.allocate_frame().expect("no more frame!");
    println!("None = {:?}, map to {:?}", page_table.translate(addr), frame);
    page_table.map_to(page, frame, EntryFlags::empty(), allocator);
    println!("Some = {:?}", page_table.translate(addr));
    println!("next free frame: {:?}", allocator.allocate_frame());

}

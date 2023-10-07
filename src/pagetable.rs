use super::SimulatorConfig;

use log::{debug, error, info, trace, warn};

#[derive(Clone, Debug)]
pub struct PageTable {
    /// The number of virtual pages in the page table.
    virtual_pages: u64,
    /// The number of physical pages in the page table.
    physical_pages: u64,
    /// Page size in bytes.
    page_size: u64,
    /// The page table entries.
    entries: Vec<Option<PageTableEntry>>,
    /// Allocated physical pages.
    allocated_physical_pages: u64,
    /// Physical page bookkeeping.
    physical_page_bookkeeping: Vec<u64>,
}

impl PageTable {
    pub fn new(virtual_pages: u64, physical_pages: u64, page_size: u64) -> Self {
        info!("Creating new page table with {virtual_pages} virtual pages, {physical_pages} physical pages, and a page size of {page_size}");
        let entries = vec![None; virtual_pages as usize];
        Self {
            virtual_pages,
            physical_pages,
            page_size,
            entries,
            allocated_physical_pages: 0,
            physical_page_bookkeeping: vec![0; physical_pages as usize],
        }
    }

    /// Create the page table from the configuration file.
    pub fn new_from_config(config: &SimulatorConfig) -> Self {
        let virtual_pages = config.page_table.number_of_virtual_pages;
        let physical_pages = config.page_table.number_of_physical_pages;
        let page_size = config.page_table.page_size;
        Self::new(virtual_pages, physical_pages, page_size)
    }

    /// Get the number of bits in the page offset. These are the number of bits shared by the physical and virtual address:
    /// the part of the address that remains untranslated.
    pub fn get_offset_bits(&self) -> u64 {
        self.page_size.trailing_zeros() as u64
    }

    /// Get the number of index bits used for the virtual page table. The number of index bits is the number
    /// of bits used to index the virtual pages in the table.
    pub fn get_index_bits(&self) -> u64 {
        self.virtual_pages.trailing_zeros() as u64
    }

    /// Get the index of the page table entry for the given virtual address.
    pub fn get_index_from_virtual_address(&self, virtual_address: u64) -> usize {
        (virtual_address
            .overflowing_shr(self.get_offset_bits() as u32)
            .0) as usize
    }

    /// Get the page offset from an address. This is the offset of the address
    /// into the given page; this masks off all the bits outside the range of the page.
    pub fn get_offset(&self, address: u64) -> u64 {
        address & (self.page_size - 1)
    }

    /// Get the physical page number from a physical address
    pub fn get_physical_page_number(&self, physical_address: u64) -> u64 {
        physical_address
            .overflowing_shr(self.get_offset_bits() as u32)
            .0
    }

    /// Get the virtual page number from a virtual address
    pub fn get_virtual_page_number(&self, virtual_address: u64) -> u64 {
        virtual_address
            .overflowing_shr(self.get_offset_bits() as u32)
            .0
    }

    /// Get the data for a page table entry associated with a virtual address.
    fn get_entry(&self, virtual_address: u64) -> Option<PageTableEntry> {
        let index = self.get_index_from_virtual_address(virtual_address);
        if index < self.entries.len() {
            self.entries[index]
        } else {
            error!("Could not find page for virtual address {virtual_address:x}");
            None
        }
    }

    /// Get a mutable reference to the page table entry for a given virtual address.
    fn get_entry_mut(&mut self, virtual_address: u64) -> Option<&mut PageTableEntry> {
        let index = self.get_index_from_virtual_address(virtual_address);
        if index < self.entries.len() {
            self.entries[index].as_mut()
        } else {
            error!("Could not find page for virtual address {virtual_address:x}");
            None
        }
    }

    /// Get the last access time for the page associated with the physical page number.
    fn get_last_access_time_page_number(&self, physical_page_number: u64) -> u64 {
        self.physical_page_bookkeeping[physical_page_number as usize]
    }

    /// Get the last access time for the page associated with a physical address.
    fn get_last_access_time_addr(&self, physical_address: u64) -> u64 {
        let physical_page_number = self.get_physical_page_number(physical_address);
        self.get_last_access_time_page_number(physical_page_number)
    }

    /// Set the last access time for the page associated with the physical page number.
    /// This will update the LRU data for the page.
    fn set_last_access_time_page_number(
        &mut self,
        physical_page_number: u64,
        current_access_time: u64,
    ) {
        let physical_page_number = physical_page_number as usize;
        if physical_page_number < self.physical_page_bookkeeping.len() {
            trace!("Setting access time for physical page #{physical_page_number:x} to be {current_access_time}");
            self.physical_page_bookkeeping[physical_page_number] = current_access_time;
        } else {
            error!("Could not set access time for page #{physical_page_number} at time={current_access_time}; page doesn't exist");
        }
    }

    /// Set the last access time for the page associated with a physical address.
    /// This will update the LRU data for the page.
    fn set_last_access_time_addr(&mut self, physical_address: u64, current_access_time: u64) {
        let physical_page_number = self.get_physical_page_number(physical_address);
        self.set_last_access_time_page_number(physical_page_number, current_access_time);
    }

    /// This will free the page associated with a physical page number.
    /// This will set its last access time to zero, so that it will
    /// be the least recently used.
    fn mark_page_number_free(&mut self, physical_page_number: u64) {
        trace!("Marking page #{physical_page_number:x} as free");
        self.set_last_access_time_page_number(physical_page_number, 0);
        self.invalidate_page_number(physical_page_number);
    }

    /// This will free the page associated with a physical address.
    /// This will set its last access time to zero, so that it will
    /// be the least recently used.
    fn mark_addr_free(&mut self, physical_address: u64) {
        trace!("Marking page associated with {physical_address:x} as free");
        self.set_last_access_time_addr(physical_address, 0);
        self.invalidate_addr(physical_address);
    }

    /// Is the page with the physical page number free?
    fn is_page_number_free(&self, physical_page_number: u64) -> bool {
        self.get_last_access_time_page_number(physical_page_number) == 0
    }

    /// Is the page associated with the physical address free?
    fn is_addr_free(&self, physical_addr: u64) -> bool {
        self.get_last_access_time_addr(physical_addr) == 0
    }

    /// Invalidate all the page table entries that are mapped to a given physical page number.
    fn invalidate_page_number(&mut self, physical_page_number: u64) {
        // Go through all the entries that reference the physical page number and invalidate them.
        trace!("Invalidating entries for page #{physical_page_number:x}");
        for entry in self.entries.iter_mut() {
            if let Some(inner_entry) = entry {
                if inner_entry.get_physical_page_number() == physical_page_number {
                    trace!(
                        "Invalidated virtual page #{}",
                        inner_entry.get_virtual_page_number()
                    );
                    *entry = None;
                }
            }
        }
    }

    /// Invalidate all the page table entries that are mapped to a page associated with a physical address.
    fn invalidate_addr(&mut self, physical_address: u64) {
        let physical_page_number = self.get_physical_page_number(physical_address);
        self.invalidate_page_number(physical_page_number);
    }

    /// Evict a page from the full page table. If there is a free entry
    fn evict(&mut self) {
        let mut min_access_time = u64::MAX;
        let mut min_access_time_page_number = 0;
        for page_number in 0..self.physical_page_bookkeeping.len() {
            if self.is_page_number_free(page_number as u64) {
                min_access_time_page_number = page_number;
                break;
            }

            let last_access = self.get_last_access_time_page_number(page_number as u64);
            trace!(
                "Last access time for page number {} is {}",
                page_number,
                last_access
            );
            if last_access < min_access_time {
                min_access_time = last_access;
                min_access_time_page_number = page_number;
            }
        }

        // Evict the chosen page from the table.
        self.mark_page_number_free(min_access_time_page_number as u64);

        trace!(
            "Evicting page table entry at index {}",
            min_access_time_page_number
        );
        self.allocated_physical_pages -= 1
    }

    /// Allocate a page in the page table with the given virtual address, and the current time of access.
    /// This will bring the new page into the page table and evict the least recently used entry if full.
    ///
    /// This function returns the physical page number of the physical page.
    fn allocate_physical_page(
        &mut self,
        virtual_address: u64,
        current_access_time: u64,
    ) -> Option<u64> {
        let index = self.get_index_from_virtual_address(virtual_address);
        if index >= self.entries.len() {
            // If the virtual address is not a valid address, return None.
            return None;
        }
        // Get the physical page number for the next page
        let mut physical_page_number = self.allocated_physical_pages;
        // Mark a new allocated page
        self.allocated_physical_pages += 1;
        // If the table is already full, evict a page.
        if self.allocated_physical_pages >= self.physical_pages {
            trace!("Page table full; evicting page to fit {virtual_address:x} at time={current_access_time}");
            self.evict();
        } else {
            trace!("Using already free page #{physical_page_number} in page table for new entry.");
        }

        // Find the free page
        for i in 0..self.physical_page_bookkeeping.len() {
            if self.is_page_number_free(i as u64) {
                physical_page_number = i as u64;
                break;
            }
        }
        // Create the new page table entry in the free'd slot
        self.entries[index] = Some(PageTableEntry::new(
            physical_page_number << self.get_offset_bits(),
            virtual_address,
            self.page_size,
            current_access_time,
        ));
        // Mark the page table entry as accessed.
        self.mark_virtual_access(virtual_address, current_access_time);
        trace!(
            "Allocated page table entry for virtual address {virtual_address:x} at physical page #{}",
            physical_page_number
        );
        // Return the allocated physical page number
        Some(physical_page_number)
    }

    /// This is used to update the LRU bookkeeping for the virtual address.
    pub fn mark_virtual_access(&mut self, virtual_address: u64, current_access_time: u64) {
        let entry = self.get_entry_mut(virtual_address);
        if let Some(entry) = entry {
            entry.last_access_time = current_access_time;
            let entry = entry.clone();
            trace!("Marking virtual access for address {virtual_address:x} at time={current_access_time}");
            self.mark_physical_access(entry.get_physical_address(), current_access_time);
        } else {
            error!("Could not mark access for virtual address {virtual_address:x}, no associated entry found");
        }
    }

    /// This will update the bookkeeping for the physical address. This sets the last access time of the physical page.
    /// This is used to update LRU info about each entry allocated in the page table.
    pub fn mark_physical_access(&mut self, physical_address: u64, current_access_time: u64) {
        trace!("Marking physical access for address {physical_address:x} at time={current_access_time}");
        let physical_page_number = self.get_physical_page_number(physical_address);
        self.physical_page_bookkeeping[physical_page_number as usize] = current_access_time.max(1);
    }

    /// This will translate the virtual address to a physical address using the page table.
    /// If the page is not mapped (page fault), then the page will allocated and brought into the page table.
    /// If the page table is full when this happens, a page will be evicted.
    /// This function will return the translated physical address, and whether or not the translation was a hit (not a fault).
    pub fn translate(
        &mut self,
        virtual_address: u64,
        current_access_time: u64,
    ) -> Option<(u64, bool)> {
        trace!("Translating virtual address {virtual_address:x} at time={current_access_time}");
        let is_hit: bool = self.get_entry(virtual_address).is_some();
        if !is_hit {
            trace!("Page fault, allocating page for virtual address {virtual_address:x} at time={current_access_time}");
            self.allocate_physical_page(virtual_address, current_access_time);
        }
        let offset = self.get_offset(virtual_address);
        let entry = self.get_entry_mut(virtual_address)?;
        trace!(
            "Updating last access time for page table entry at physical page #{} from {} to {}",
            entry.get_physical_page_number(),
            entry.last_access_time,
            current_access_time
        );
        entry.last_access_time = current_access_time;
        let physical_address = entry.get_physical_address() | offset;
        self.mark_physical_access(physical_address, current_access_time);
        trace!(
            "Virtual address {:08x} translated to physical address {:08x}",
            virtual_address,
            physical_address
        );

        Some((physical_address, is_hit))
    }
}

#[derive(Clone, Default, Copy, Debug)]
pub struct PageTableEntry {
    /// The physical address of the page.
    physical_address: u64,
    /// The virtual address of the page.
    virtual_address: u64,
    /// Last access time.
    last_access_time: u64,
    /// Page size
    page_size: u64,
}

impl PageTableEntry {
    pub fn new(
        physical_address: u64,
        virtual_address: u64,
        page_size: u64,
        current_access_time: u64,
    ) -> Self {
        trace!(
            "Allocating page table entry for virtual address {:08x} at physical address {:08x}",
            virtual_address,
            physical_address
        );
        Self {
            physical_address,
            virtual_address,
            page_size,
            last_access_time: current_access_time,
        }
    }

    pub fn get_physical_address(&self) -> u64 {
        self.physical_address & !(self.page_size - 1)
    }

    pub fn get_virtual_address(&self) -> u64 {
        self.virtual_address
    }

    pub fn get_virtual_page_number(&self) -> u64 {
        (self.virtual_address & !(self.page_size - 1)) >> self.page_size.trailing_zeros()
    }

    pub fn get_physical_page_number(&self) -> u64 {
        (self.physical_address & !(self.page_size - 1)) >> self.page_size.trailing_zeros()
    }

    pub fn get_offset_bits(&self) -> u64 {
        self.page_size.trailing_zeros() as u64
    }
}

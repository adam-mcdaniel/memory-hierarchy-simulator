use super::SimulatorConfig;


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

    pub fn new_from_config(config: &SimulatorConfig) -> Self {
        let virtual_pages = config.page_table.number_of_virtual_pages;
        let physical_pages = config.page_table.number_of_physical_pages;
        let page_size = config.page_table.page_size;
        Self::new(virtual_pages, physical_pages, page_size)
    }

    pub fn get_offset_bits(&self) -> u64 {
        self.page_size.trailing_zeros() as u64
    }

    pub fn get_index_bits(&self) -> u64 {
        self.virtual_pages.trailing_zeros() as u64
    }

    /// Get the index of the page table entry for the given virtual address.
    pub fn get_index_from_virtual_address(&self, virtual_address: u64) -> usize {
        (virtual_address.overflowing_shr(self.get_offset_bits() as u32).0) as usize
    }

    pub fn get_offset(&self, address: u64) -> u64 {
        address & (self.page_size - 1)
    }

    pub fn get_physical_page_number(&self, physical_address: u64) -> u64 {
        physical_address.overflowing_shr(self.get_offset_bits() as u32).0
    }
    
    pub fn get_virtual_page_number(&self, virtual_address: u64) -> u64 {
        virtual_address.overflowing_shr(self.get_offset_bits() as u32).0
    }

    fn get_entry(&self, virtual_address: u64) -> Option<PageTableEntry> {
        let index = self.get_index_from_virtual_address(virtual_address);
        if index < self.entries.len() {
            self.entries[index]
        } else {
            None
        }
    }

    fn get_last_access_time_page_number(&self, physical_page_number: u64) -> u64 {
        self.physical_page_bookkeeping[physical_page_number as usize]
    }

    fn get_last_access_time_addr(&self, physical_address: u64) -> u64 {
        let physical_page_number = self.get_physical_page_number(physical_address);
        self.get_last_access_time_page_number(physical_page_number)
    }

    fn get_entry_mut(&mut self, virtual_address: u64) -> Option<&mut PageTableEntry> {
        let index = self.get_index_from_virtual_address(virtual_address);
        if index < self.entries.len() {
            self.entries[index].as_mut()
        } else {
            None
        }
    }

    fn evict(&mut self) {
        let mut min_access_time = u64::MAX;
        let mut min_access_time_page_number = 0;
        for page_number in 0..self.physical_page_bookkeeping.len() {
            let physical_address = (page_number as u64) << self.get_offset_bits();
            if self.physical_page_bookkeeping[page_number] == 0 {
                min_access_time_page_number = page_number;
                break;
            }
            let last_access = self.get_last_access_time_addr(physical_address);
            eprintln!("Last access time for page number {} is {}", page_number, last_access);
            if last_access < min_access_time {
                min_access_time = last_access;
                min_access_time_page_number = page_number;
            }
        }
        self.physical_page_bookkeeping[min_access_time_page_number] = 0;

        // Evict the page table entry for the corresponding physical page.
        // let physical_address = (min_access_time_page_number as u64) << self.get_offset_bits();
        for (i, entry) in self.entries.iter_mut().enumerate() {
            if let Some(inner_entry) = entry {
                if inner_entry.get_physical_page_number() == min_access_time_page_number as u64 {
                    *entry = None;
                }
            }
        }

        eprintln!("Evicting page table entry at index {}", min_access_time_page_number);
        self.allocated_physical_pages -= 1
    }

    fn allocate_physical_page(&mut self, virtual_address: u64, current_access_time: u64) -> Option<u64> {
        let index = self.get_index_from_virtual_address(virtual_address);
        if index < self.entries.len() {
            let mut physical_page_number = self.allocated_physical_pages;
            self.allocated_physical_pages += 1;
            if self.allocated_physical_pages >= self.physical_pages {
                self.evict();
            }
            
            for i in 0..self.physical_page_bookkeeping.len() {
                if self.physical_page_bookkeeping[i] == 0 {
                    physical_page_number = i as u64;
                    break;
                }
            }
            self.entries[index] = Some(PageTableEntry::new(physical_page_number << self.get_offset_bits(), virtual_address, self.page_size, current_access_time));
            // self.physical_page_bookkeeping[physical_page_number as usize] = current_access_time + 1;
            self.mark_virtual_access(virtual_address, current_access_time);
            eprintln!("Allocated page table entry at index {}", physical_page_number);
            Some(physical_page_number)
        } else {
            None
        }
    }

    pub fn mark_virtual_access(&mut self, virtual_address: u64, current_access_time: u64) {
        let entry = self.get_entry_mut(virtual_address);
        if let Some(entry) = entry {
            entry.last_access_time = current_access_time;
            let entry = entry.clone();
            self.mark_physical_access(entry.get_physical_address(), current_access_time);
        }
    }

    pub fn mark_physical_access(&mut self, physical_address: u64, current_access_time: u64) {
        let physical_page_number = self.get_physical_page_number(physical_address);
        self.physical_page_bookkeeping[physical_page_number as usize] = current_access_time.max(1);
    }

    pub fn translate(&mut self, virtual_address: u64, current_access_time: u64) -> Option<(u64, bool)> {
        let is_hit: bool = self.get_entry(virtual_address).is_some();
        if !is_hit {
            self.allocate_physical_page(virtual_address, current_access_time);
        }
        let offset = self.get_offset(virtual_address);
        let entry = self.get_entry_mut(virtual_address)?;
        eprintln!("Updating last access time for page table entry at index {} from {} to {}", entry.get_physical_page_number(), entry.last_access_time, current_access_time);
        entry.last_access_time = current_access_time;
        let physical_address = entry.get_physical_address() | offset;
        self.mark_physical_access(physical_address, current_access_time);
        eprintln!("Virtual address {:08x} translated to physical address {:08x}", virtual_address, physical_address);
        
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
    pub fn new(physical_address: u64, virtual_address: u64, page_size: u64, current_access_time: u64) -> Self {
        eprintln!("Allocating page table entry for virtual address {:08x} at physical address {:08x}", virtual_address, physical_address);
        Self {
            physical_address,
            virtual_address,
            page_size,
            last_access_time: current_access_time
        }
    }

    pub fn get_physical_address(&self) -> u64 {
        self.physical_address & !(self.page_size - 1)
    }

    pub fn get_virtual_address(&self) -> u64 {
        self.virtual_address
    }

    pub fn get_virtual_page_number(&self) -> u64 {
        self.virtual_address & !(self.page_size - 1) >> self.page_size.trailing_zeros()
    }

    pub fn get_physical_page_number(&self) -> u64 {
        self.physical_address & !(self.page_size - 1) >> self.page_size.trailing_zeros()
    }

    pub fn get_offset_bits(&self) -> u64 {
        self.page_size.trailing_zeros() as u64
    }    
}
use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    io::{BufReader, Read},
};

use super::{get_bool, get_decimal, get_header};
use crate::EvictionPolicy;

pub struct SimulatorConfig {
    /// Are virtual addresses enabled?
    pub virtual_addresses_enabled: bool,
    /// Is the TLB enabled?
    pub tlb_enabled: bool,
    /// Is the L2 cache enabled?
    pub l2_cache_enabled: bool,

    /// The configuration settings for the TLB.
    pub tlb: TLBConfig,
    /// The configuration settings for the page table.
    pub page_table: PageTableConfig,
    /// The configuration settings for the data cache.
    pub data_cache: DataCacheConfig,
    /// The configuration settings for the L2 cache.
    pub l2_cache: L2CacheConfig,
}

impl Default for SimulatorConfig {
    fn default() -> Self {
        // Read the configuration from the file "trace.config".
        Self::from_file("trace.config")
    }
}

impl SimulatorConfig {
    /// Read the configuration from a file, buffer, or other reader.
    fn from_buffer<R>(buffer: &mut BufReader<R>) -> Self
    where
        R: Read,
    {
        // Read the TLB configuration from the file.
        let tlb = TLBConfig::from_buffer(buffer);
        // Read the page table configuration from the file.
        let page_table = PageTableConfig::from_buffer(buffer);
        // Read the data cache configuration from the file.
        let data_cache = DataCacheConfig::from_buffer(buffer);
        // Read the L2 cache configuration from the file.
        let l2_cache = L2CacheConfig::from_buffer(buffer);

        // Read the last three lines of the file, which enable certain features of the simulator.
        let virtual_addresses_enabled = get_bool(buffer, Some("Virtual addresses")).unwrap().1;
        let tlb_enabled = get_bool(buffer, Some("TLB")).unwrap().1;
        let l2_cache_enabled = get_bool(buffer, Some("L2 cache")).unwrap().1;

        Self {
            virtual_addresses_enabled,
            tlb_enabled,
            l2_cache_enabled,
            tlb,
            page_table,
            data_cache,
            l2_cache,
        }
    }

    /// Read the configuration from a file.
    fn from_file(path: &str) -> Self {
        let file = std::fs::File::open(path).unwrap();
        let mut buffer = BufReader::new(file);
        Self::from_buffer(&mut buffer)
    }

    /// Get the size of a page in bytes.
    pub fn get_page_size(&self) -> u64 {
        self.page_table.get_page_size()
    }

    pub fn get_tlb_tag_bits(&self) -> u64 {
        // The number of gits in the virtual page number
        self.page_table.get_virtual_page_number_bits()
    }

    pub fn get_tlb_index_bits(&self) -> u64 {
        self.tlb.get_index_bits()
    }

    pub fn get_page_table_index_bits(&self) -> u64 {
        self.page_table.get_index_bits()
    }

    pub fn get_page_table_offset_bits(&self) -> u64 {
        self.page_table.get_offset_bits()
    }

    pub fn get_data_cache_index_bits(&self) -> u64 {
        self.data_cache.get_index_bits()
    }

    pub fn get_data_cache_offset_bits(&self) -> u64 {
        self.data_cache.get_offset_bits()
    }

    pub fn get_l2_cache_index_bits(&self) -> u64 {
        self.l2_cache.get_index_bits()
    }

    pub fn get_l2_cache_offset_bits(&self) -> u64 {
        self.l2_cache.get_offset_bits()
    }

    pub fn is_tlb_enabled(&self) -> bool {
        self.tlb_enabled
    }

    pub fn is_l2_cache_enabled(&self) -> bool {
        self.l2_cache_enabled
    }

    pub fn is_virtual_addresses_enabled(&self) -> bool {
        self.virtual_addresses_enabled
    }
}

impl Display for SimulatorConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "{}\n{}\n{}\n{}",
            self.tlb, self.page_table, self.data_cache, self.l2_cache
        )?;

        writeln!(
            f,
            "The addresses read in are {} addresses.",
            if self.virtual_addresses_enabled {
                "virtual"
            } else {
                "physical"
            }
        )?;

        if !self.tlb_enabled {
            writeln!(f, "TLB is disabled in this configuration.")?;
        }

        if !self.l2_cache_enabled {
            writeln!(f, "L2 cache is disabled in this configuration.")?;
        }

        Ok(())
    }
}

pub struct TLBConfig {
    pub number_of_sets: u64,
    pub set_size: u64,
}

impl TLBConfig {
    pub fn new(number_of_sets: u64, set_size: u64) -> Self {
        Self {
            number_of_sets,
            set_size,
        }
    }

    /// Get the eviction policy for the TLB cache.
    pub fn get_eviction_policy(&self) -> EvictionPolicy {
        EvictionPolicy::LRU
    }

    /// Returns the number of bits used for the TLB index.
    /// The TLB index is the number of bits used to address a TLB entry.
    /// The TLB doesn't have sets because it is fully associative.
    pub fn get_index_bits(&self) -> u64 {
        self.number_of_sets.trailing_zeros() as u64
    }

    /// Get the number of sets in the TLB.
    pub fn get_number_of_sets(&self) -> u64 {
        self.number_of_sets
    }

    /// Get the number of page table entries in each set.
    pub fn get_entries_in_set(&self) -> u64 {
        self.set_size
    }

    /// Get the associativity of the TLB.
    pub fn get_associativity(&self) -> u64 {
        self.number_of_sets
    }

    /// Read the configuration from a file, buffer, or other reader.
    fn from_buffer<R>(buffer: &mut BufReader<R>) -> Self
    where
        R: Read,
    {
        get_header(buffer, "Data TLB configuration");
        let number_of_sets = get_decimal(buffer, Some("Number of sets")).unwrap().1;
        let set_size = get_decimal(buffer, Some("Set size")).unwrap().1;
        Self::new(number_of_sets, set_size)
    }
}

impl Display for TLBConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        writeln!(f, "Data TLB contains {} sets.\nEach set contains {} entries.\nNumber of bits used for the index is {}.", self.number_of_sets, self.set_size, self.get_index_bits())
    }
}

/// Configuration for the page table.
pub struct PageTableConfig {
    /// Number of virtual pages in the page table.
    pub number_of_virtual_pages: u64,
    /// Total number physical pages available in the system.
    pub number_of_physical_pages: u64,
    /// Number of bytes in each page.
    pub page_size: u64,
}

impl PageTableConfig {
    pub fn new(
        number_of_virtual_pages: u64,
        number_of_physical_pages: u64,
        page_size: u64,
    ) -> Self {
        Self {
            number_of_virtual_pages,
            number_of_physical_pages,
            page_size,
        }
    }

    /// Get the size of a page.
    pub fn get_page_size(&self) -> u64 {
        self.page_size
    }

    /// Returns the number of bits used to represent the virtual page number
    /// in a virtual address.
    pub fn get_virtual_page_number_bits(&self) -> u64 {
        self.number_of_virtual_pages.trailing_zeros() as u64
    }

    /// Returns the number of bits used for the page table index.
    /// The page table index is the number of bits used to address a page table entry.
    pub fn get_index_bits(&self) -> u64 {
        self.number_of_virtual_pages.trailing_zeros() as u64
    }

    /// Returns the number of bits used for the page offset.
    /// The page offset is the number of bits used to address a byte within a page.
    pub fn get_offset_bits(&self) -> u64 {
        self.page_size.trailing_zeros() as u64
    }

    /// Read the configuration from a file, buffer, or other reader.
    fn from_buffer<R>(buffer: &mut BufReader<R>) -> Self
    where
        R: Read,
    {
        get_header(buffer, "Page Table configuration");
        let number_of_virtual_pages = get_decimal(buffer, Some("Number of virtual pages"))
            .unwrap()
            .1;
        let number_of_physical_pages = get_decimal(buffer, Some("Number of physical pages"))
            .unwrap()
            .1;
        let page_size = get_decimal(buffer, Some("Page size")).unwrap().1;
        Self::new(number_of_virtual_pages, number_of_physical_pages, page_size)
    }
}

impl Display for PageTableConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        writeln!(f, "Number of virtual pages is {}.\nNumber of physical pages is {}.\nEach page contains {} bytes.\nNumber of bits used for the page table index is {}.\nNumber of bits used for the page offset is {}.", self.number_of_virtual_pages, self.number_of_physical_pages, self.page_size, self.get_index_bits(), self.get_offset_bits())
    }
}

/// Configuration for the data cache.
pub struct DataCacheConfig {
    /// Number of sets in the cache.
    pub number_of_sets: u64,
    /// Number of entries in each set.
    pub set_size: u64,
    /// Number of bytes in each cache line.
    pub line_size: u64,
    /// Is the cache write-through?
    pub write_through: bool,
}

impl DataCacheConfig {
    pub fn new(number_of_sets: u64, set_size: u64, line_size: u64, write_through: bool) -> Self {
        Self {
            number_of_sets,
            set_size,
            line_size,
            write_through,
        }
    }

    /// Returns the number of bits used for the cache set index.
    /// The cache set index is the number of bits used to address a cache set.
    pub fn get_index_bits(&self) -> u64 {
        self.number_of_sets.trailing_zeros() as u64
    }

    /// Returns the number of bits used for the cache line offset.
    /// The cache line offset is the number of bits used to address a byte within a cache line.
    pub fn get_offset_bits(&self) -> u64 {
        self.line_size.trailing_zeros() as u64
    }

    /// Is the cache write-through?
    /// Write through means that the data is written to both the cache and the main memory.
    /// A write-through cache uses a no-write-allocate policy.
    pub fn is_write_through(&self) -> bool {
        self.write_through
    }

    /// Is the cache no-write-allocate?
    /// No-write-allocate means that the cache line is not loaded into the cache when a write miss occurs.
    /// A write-through cache uses a no-write-allocate policy.
    pub fn is_no_write_allocate(&self) -> bool {
        self.write_through
    }

    /// Is the cache write-back?
    /// Write back means that the data is written to the cache and the main memory is updated when the cache line is evicted.
    /// A write-back cache uses a write-allocate policy.
    pub fn is_write_back(&self) -> bool {
        !self.write_through
    }

    /// Is the cache write-allocate?
    /// Write allocate means that the cache line is loaded into the cache when a write miss occurs.
    /// A write-back cache uses a write-allocate policy.
    pub fn is_write_allocate(&self) -> bool {
        !self.write_through
    }

    /// Get the associativity of the data cache from the configuration.
    pub fn get_associativity(&self) -> u64 {
        // The associativity is the number of blocks in each set
        self.set_size
    }

    /// Get the number of bytes in each block (the line size for the cache).
    pub fn get_block_size(&self) -> u64 {
        self.line_size
    }

    /// Get the eviction policy for the data cache.
    pub fn get_eviction_policy(&self) -> EvictionPolicy {
        EvictionPolicy::LRU
    }

    /// Get the number of sets in the data cache.
    pub fn get_number_of_sets(&self) -> u64 {
        self.number_of_sets
    }

    /// Read the configuration from a file, buffer, or other reader.
    fn from_buffer<R>(buffer: &mut BufReader<R>) -> Self
    where
        R: Read,
    {
        get_header(buffer, "Data Cache configuration");
        let number_of_sets = get_decimal(buffer, Some("Number of sets")).unwrap().1;
        let set_size = get_decimal(buffer, Some("Set size")).unwrap().1;
        let line_size = get_decimal(buffer, Some("Line size")).unwrap().1;
        let write_through = get_bool(buffer, Some("Write through/no write allocate"))
            .unwrap()
            .1;
        Self::new(number_of_sets, set_size, line_size, write_through)
    }
}

impl Display for DataCacheConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let is_write_through = self.is_write_through();
        let is_write_allocate = self.is_write_allocate();

        let allocate_policy = if is_write_allocate { "" } else { "no " };
        let write_policy = if is_write_through { "through" } else { "back" };

        writeln!(f, "D-cache contains {} sets.\nEach set contains {} entries.\nEach line is {} bytes.\nThe cache uses a {}write-allocate and write-{} policy.\nNumber of bits used for the index is {}.\nNumber of bits used for the offset is {}.", self.number_of_sets, self.set_size, self.line_size, allocate_policy, write_policy, self.get_index_bits(), self.get_offset_bits())

        // writeln!(f, "D-cache contains {} sets.\nEach set contains {} entries.\nEach line is {} bytes.\nThe cache uses a {}write-allocate and write-{} policy.\nNumber of bits used for the index is {}.\nNumber of bits used for the offset is {}.", self.number_of_sets, self.set_size, self.line_size, if self.write_through { "no " } else { "" }, if self.write_through { "through" } else {"back"}, self.get_index_bits(), self.get_offset_bits())
    }
}

/// Configuration for the L2 cache.
pub struct L2CacheConfig {
    /// Number of sets in the L2 cache.
    pub number_of_sets: u64,
    /// Number of entries in each set.
    pub set_size: u64,
    /// Number of bytes in each cache line.
    pub line_size: u64,
    /// Is the cache write-through?
    pub write_through: bool,
}

impl L2CacheConfig {
    pub fn new(number_of_sets: u64, set_size: u64, line_size: u64, write_through: bool) -> Self {
        Self {
            number_of_sets,
            set_size,
            line_size,
            write_through,
        }
    }

    /// Returns the number of bits used for the cache set index.
    /// The cache set index is the number of bits used to address a cache set.
    pub fn get_index_bits(&self) -> u64 {
        self.number_of_sets.trailing_zeros() as u64
    }

    /// Returns the number of bits used for the cache line offset.
    /// The cache line offset is the number of bits used to address a byte within a cache line.
    pub fn get_offset_bits(&self) -> u64 {
        self.line_size.trailing_zeros() as u64
    }

    /// Is the cache write-through?
    /// Write through means that the data is written to both the cache and the main memory.
    /// A write-through cache uses a no-write-allocate policy.
    pub fn is_write_through(&self) -> bool {
        self.write_through
    }

    /// Is the cache no-write-allocate?
    /// No-write-allocate means that the cache line is not loaded into the cache when a write miss occurs.
    /// A write-through cache uses a no-write-allocate policy.
    pub fn is_no_write_allocate(&self) -> bool {
        self.write_through
    }

    /// Is the cache write-back?
    /// Write back means that the data is written to the cache and the main memory is updated when the cache line is evicted.
    /// A write-back cache uses a write-allocate policy.
    pub fn is_write_back(&self) -> bool {
        !self.write_through
    }

    /// Is the cache write-allocate?
    /// Write allocate means that the cache line is loaded into the cache when a write miss occurs.
    /// A write-back cache uses a write-allocate policy.
    pub fn is_write_allocate(&self) -> bool {
        !self.write_through
    }

    /// Get the associativity of the data cache from the configuration.
    pub fn get_associativity(&self) -> u64 {
        // The associativity is the number of blocks in each set
        self.set_size
    }

    /// Get the number of bytes in each block (the line size for the cache).
    pub fn get_block_size(&self) -> u64 {
        self.line_size
    }

    /// Get the eviction policy for the data cache.
    pub fn get_eviction_policy(&self) -> EvictionPolicy {
        EvictionPolicy::LRU
    }

    /// Get the number of sets in the data cache.
    pub fn get_number_of_sets(&self) -> u64 {
        self.number_of_sets
    }

    /// Read the configuration from a file, buffer, or other reader.
    fn from_buffer<R>(buffer: &mut BufReader<R>) -> Self
    where
        R: Read,
    {
        get_header(buffer, "L2 Cache configuration");
        let number_of_sets = get_decimal(buffer, Some("Number of sets")).unwrap().1;
        let set_size = get_decimal(buffer, Some("Set size")).unwrap().1;
        let line_size = get_decimal(buffer, Some("Line size")).unwrap().1;
        let write_through = get_bool(buffer, Some("Write through/no write allocate"))
            .unwrap()
            .1;
        Self::new(number_of_sets, set_size, line_size, write_through)
    }
}

impl Display for L2CacheConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        writeln!(f, "L2-cache contains {} sets.\nEach set contains {} entries.\nEach line is {} bytes.\nThe cache uses a {}write-allocate and write-{} policy.\nNumber of bits used for the index is {}.\nNumber of bits used for the offset is {}.\n", self.number_of_sets, self.set_size, self.line_size, if self.write_through { "no " } else { "" }, if self.write_through { "through" } else {"back"}, self.get_index_bits(), self.get_offset_bits())
    }
}

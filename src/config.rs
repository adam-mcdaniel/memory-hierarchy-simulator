use std::{
    io::{Read, BufRead, BufReader},
    fmt::{Display, Formatter, Result as FmtResult},
};

use super::{get_header, get_decimal, get_bool};

pub struct Config {
    pub virtual_addresses_enabled: bool,
    pub tlb_enabled: bool,
    pub l2_cache_enabled: bool,

    pub tlb: TLBConfig,
    pub page_table: PageTableConfig,
    pub data_cache: DataCacheConfig,
    pub l2_cache: L2Cache,
}

impl Config {
    fn from_buffer<R>(buffer: &mut BufReader<R>) -> Self where R: Read {
        let tlb = TLBConfig::from_buffer(buffer);
        let page_table = PageTableConfig::from_buffer(buffer);
        let data_cache = DataCacheConfig::from_buffer(buffer);
        let l2_cache = L2Cache::from_buffer(buffer);

        let virtual_addresses_enabled = get_bool(buffer, Some("Virtual addresses")).1;
        let tlb_enabled = get_bool(buffer, Some("TLB")).1;
        let l2_cache_enabled = get_bool(buffer, Some("L2 cache")).1;

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

    fn from_file(path: &str) -> Self {
        let file = std::fs::File::open(path).unwrap();
        let mut buffer = BufReader::new(file);
        Self::from_buffer(&mut buffer)
    }
}

impl Display for Config {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}\n{}\n{}\n{}", self.tlb, self.page_table, self.data_cache, self.l2_cache)?;

        writeln!(f, "The addresses read in are {} addresses.", if self.virtual_addresses_enabled { "virtual" } else { "physical" })?;

        if !self.tlb_enabled {
            writeln!(f, "TLB is disabled in this configuration.")?;
        }

        if !self.l2_cache_enabled {
            writeln!(f, "L2 cache is disabled in this configuration.")?;
        }

        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::from_file("trace.config")
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

    pub fn get_index_bits(&self) -> u64 {
        self.number_of_sets.trailing_zeros() as u64
    }

    fn from_buffer<R>(buffer: &mut BufReader<R>) -> Self where R: Read {
        get_header(buffer, "Data TLB configuration");
        let number_of_sets = get_decimal(buffer, Some("Number of sets")).1;
        let set_size = get_decimal(buffer, Some("Set size")).1;
        Self::new(number_of_sets, set_size)
    }
}

impl Display for TLBConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        writeln!(f, "Data TLB contains {} sets.\nEach set contains {} entries.\nNumber of bits used for the index is {}.", self.number_of_sets, self.set_size, self.get_index_bits())
    }
}

pub struct PageTableConfig {
    pub number_of_virtual_pages: u64,
    pub number_of_physical_pages: u64,
    pub page_size: u64,
}

impl PageTableConfig {
    pub fn new(number_of_virtual_pages: u64, number_of_physical_pages: u64, page_size: u64) -> Self {
        Self {
            number_of_virtual_pages,
            number_of_physical_pages,
            page_size,
        }
    }

    pub fn get_index_bits(&self) -> u64 {
        self.number_of_virtual_pages.trailing_zeros() as u64
    }

    pub fn get_offset_bits(&self) -> u64 {
        self.page_size.trailing_zeros() as u64
    }

    fn from_buffer<R>(buffer: &mut BufReader<R>) -> Self where R: Read {
        get_header(buffer, "Page Table configuration");
        let number_of_virtual_pages = get_decimal(buffer, Some("Number of virtual pages")).1;
        let number_of_physical_pages = get_decimal(buffer, Some("Number of physical pages")).1;
        let page_size = get_decimal(buffer, Some("Page size")).1;
        Self::new(number_of_virtual_pages, number_of_physical_pages, page_size)
    }
}

impl Display for PageTableConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        writeln!(f, "Number of virtual pages is {}.\nNumber of physical pages is {}.\nEach page contains {} bytes.\nNumber of bits used for the page table index is {}.\nNumber of bits used for the page offset is {}.", self.number_of_virtual_pages, self.number_of_physical_pages, self.page_size, self.get_index_bits(), self.get_offset_bits())
    }
}

pub struct DataCacheConfig {
    pub number_of_sets: u64,
    pub set_size: u64,
    pub line_size: u64,
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

    pub fn get_index_bits(&self) -> u64 {
        self.number_of_sets.trailing_zeros() as u64
    }

    pub fn get_offset_bits(&self) -> u64 {
        self.line_size.trailing_zeros() as u64
    }

    fn from_buffer<R>(buffer: &mut BufReader<R>) -> Self where R: Read {
        get_header(buffer, "Data Cache configuration");
        let number_of_sets = get_decimal(buffer, Some("Number of sets")).1;
        let set_size = get_decimal(buffer, Some("Set size")).1;
        let line_size = get_decimal(buffer, Some("Line size")).1;
        let write_through = get_bool(buffer, Some("Write through/no write allocate")).1;
        Self::new(number_of_sets, set_size, line_size, write_through)
    }
}

impl Display for DataCacheConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        writeln!(f, "D-cache contains {} sets.\nEach set contains {} entries.\nEach line is {} bytes.\nThe cache uses a {}write-allocate and write-{} policy.\nNumber of bits used for the index is {}.\nNumber of bits used for the offset is {}.", self.number_of_sets, self.set_size, self.line_size, if self.write_through { "no " } else { "" }, if self.write_through { "through" } else {"back"}, self.get_index_bits(), self.get_offset_bits())
    }
}

pub struct L2Cache {
    pub number_of_sets: u64,
    pub set_size: u64,
    pub line_size: u64,
    pub write_through: bool,
}

impl L2Cache {
    pub fn new(number_of_sets: u64, set_size: u64, line_size: u64, write_through: bool) -> Self {
        Self {
            number_of_sets,
            set_size,
            line_size,
            write_through,
        }
    }

    pub fn get_index_bits(&self) -> u64 {
        self.number_of_sets.trailing_zeros() as u64
    }

    pub fn get_offset_bits(&self) -> u64 {
        self.line_size.trailing_zeros() as u64
    }

    fn from_buffer<R>(buffer: &mut BufReader<R>) -> Self where R: Read {
        get_header(buffer, "L2 Cache configuration");
        let number_of_sets = get_decimal(buffer, Some("Number of sets")).1;
        let set_size = get_decimal(buffer, Some("Set size")).1;
        let line_size = get_decimal(buffer, Some("Line size")).1;
        let write_through = get_bool(buffer, Some("Write through/no write allocate")).1;
        Self::new(number_of_sets, set_size, line_size, write_through)
    }
}

impl Display for L2Cache {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        writeln!(f, "L2-cache contains {} sets.\nEach set contains {} entries.\nEach line is {} bytes.\nThe cache uses a {}write-allocate and write-{} policy.\nNumber of bits used for the index is {}.\nNumber of bits used for the offset is {}.\n", self.number_of_sets, self.set_size, self.line_size, if self.write_through { "no " } else { "" }, if self.write_through { "through" } else {"back"}, self.get_index_bits(), self.get_offset_bits())
    }
}
use std::{
    io::{Read, BufRead, BufReader},
    fmt::{Display, Formatter, Result as FmtResult},
};

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

        let virtual_addresses_enabled = get_bool(buffer, "Virtual addresses");
        let tlb_enabled = get_bool(buffer, "TLB");
        let l2_cache_enabled = get_bool(buffer, "L2 cache");

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
        write!(f, "{}\n{}\n{}\n{}\nVirtual addresses: {}\nTLB: {}\nL2 cache: {}", self.tlb, self.page_table, self.data_cache, self.l2_cache, if self.virtual_addresses_enabled { "y" } else { "n" }, if self.tlb_enabled { "y" } else { "n" }, if self.l2_cache_enabled { "y" } else { "n" })
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::from_file("trace.config")
    }
}

fn get_header<R>(buffer: &mut BufReader<R>, text: &str) where R: Read {
    let mut line = String::new();
    while line.trim() == "" {
        buffer.read_line(&mut line).unwrap();
    }

    if line.trim() != text {
        panic!("Expected \"{}\", got \"{}\"", text, line.trim());
    }
}

fn get_number<R>(buffer: &mut BufReader<R>, text: &str) -> u64 where R: Read {
    let mut line = String::new();
    while line.trim() == "" {
        buffer.read_line(&mut line).unwrap();
    }
    let mut split = line.split(':');
    if split.clone().count() != 2 {
        panic!("Expected \"{}: {{number}}\", got \"{}\"", text, line.trim());
    }
    if split.next().unwrap().trim() != text {
        panic!("Expected \"{}: {{number}}\", got \"{}\"", text, line.trim());
    }
    split.next().unwrap().trim().parse::<u64>().unwrap()
}

fn get_bool<R>(buffer: &mut BufReader<R>, text: &str) -> bool where R: Read {
    let mut line = String::new();
    while line.trim() == "" {
        buffer.read_line(&mut line).unwrap();
    }
    let mut split = line.split(':');
    if split.clone().count() != 2 {
        panic!("Expected \"{}: {{bool}}\", got \"{}\"", text, line.trim());
    }
    if split.next().unwrap().trim() != text {
        panic!("Expected \"{}: {{bool}}\", got \"{}\"", text, line.trim());
    }
    let value = split.next().unwrap().trim();
    if value == "y" || value == "Y" {
        true
    } else if value == "n" || value == "N" {
        false
    } else {
        panic!("Expected \"{}: {{bool}}\", got \"{}\"", text, line.trim());
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

    fn from_buffer<R>(buffer: &mut BufReader<R>) -> Self where R: Read {
        get_header(buffer, "Data TLB configuration");
        let number_of_sets = get_number(buffer, "Number of sets");
        let set_size = get_number(buffer, "Set size");
        Self::new(number_of_sets, set_size)
    }
}

impl Display for TLBConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        writeln!(f, "Data TLB configuration\nNumber of sets: {}\nSet size: {}", self.number_of_sets, self.set_size)
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

    fn from_buffer<R>(buffer: &mut BufReader<R>) -> Self where R: Read {
        get_header(buffer, "Page Table configuration");
        let number_of_virtual_pages = get_number(buffer, "Number of virtual pages");
        let number_of_physical_pages = get_number(buffer, "Number of physical pages");
        let page_size = get_number(buffer, "Page size");
        Self::new(number_of_virtual_pages, number_of_physical_pages, page_size)
    }
}

impl Display for PageTableConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        writeln!(f, "Page table configuration\nNumber of virtual pages: {}\nNumber of physical pages: {}\nPage size: {}", self.number_of_virtual_pages, self.number_of_physical_pages, self.page_size)
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

    fn from_buffer<R>(buffer: &mut BufReader<R>) -> Self where R: Read {
        get_header(buffer, "Data Cache configuration");
        let number_of_sets = get_number(buffer, "Number of sets");
        let set_size = get_number(buffer, "Set size");
        let line_size = get_number(buffer, "Line size");
        let write_through = get_bool(buffer, "Write through/no write allocate");
        Self::new(number_of_sets, set_size, line_size, write_through)
    }
}

impl Display for DataCacheConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        writeln!(f, "Data Cache configuration\nNumber of sets: {}\nSet size: {}\nLine size: {}\nWrite through/no write allocate: {}", self.number_of_sets, self.set_size, self.line_size, if self.write_through { "y" } else { "n" })
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

    fn from_buffer<R>(buffer: &mut BufReader<R>) -> Self where R: Read {
        get_header(buffer, "L2 Cache configuration");
        let number_of_sets = get_number(buffer, "Number of sets");
        let set_size = get_number(buffer, "Set size");
        let line_size = get_number(buffer, "Line size");
        let write_through = get_bool(buffer, "Write through/no write allocate");
        Self::new(number_of_sets, set_size, line_size, write_through)
    }
}

impl Display for L2Cache {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        writeln!(f, "L2 Cache configuration\nNumber of sets: {}\nSet size: {}\nLine size: {}\nWrite through/no write allocate: {}", self.number_of_sets, self.set_size, self.line_size, if self.write_through { "y" } else { "n" })
    }
}
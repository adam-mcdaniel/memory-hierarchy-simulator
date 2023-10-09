use super::{get_hexadecimal, SimulatorConfig};
use core::fmt::{Display, Formatter, Result as FmtResult};

use std::io::{BufReader, Read};

/// A memory access operation to be performed by the simulator.
#[derive(Clone, Copy, Debug)]
pub enum Operation {
    Read(u64),
    Write(u64),
}

impl Operation {
    /// Reads an operation from stdin.
    fn from_stdin() -> Option<Self> {
        // <accesstype>:<hexaddress>
        // where <accesstype> is either R or W
        let stdin = std::io::stdin();
        let mut buffer = std::io::BufReader::new(stdin.lock());
        Self::from_buffer(&mut buffer)
    }

    fn from_buffer<R>(buffer: &mut BufReader<R>) -> Option<Self>
    where
        R: Read,
    {
        let (access_type, address) = get_hexadecimal(buffer, None)?;

        match access_type.as_str() {
            "R" => Some(Self::Read(address)),
            "W" => Some(Self::Write(address)),
            _ => None,
        }
    }

    pub fn is_read(&self) -> bool {
        match self {
            Self::Read(_) => true,
            Self::Write(_) => false,
        }
    }

    pub fn is_write(&self) -> bool {
        match self {
            Self::Read(_) => false,
            Self::Write(_) => true,
        }
    }

    pub fn address(&self) -> u64 {
        match self {
            Self::Read(address) => *address,
            Self::Write(address) => *address,
        }
    }
}

impl Display for Operation {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Read(address) => write!(f, "R:{:03x}", address),
            Self::Write(address) => write!(f, "W:{:03x}", address),
        }
    }
}

/// A block address in a cache.
#[derive(Clone, Copy, Debug)]
pub struct BlockAddress {
    /// The tag of the block. This is the uppermost bits of the address.
    /// The tag is used to determine if the block is in the cache.
    /// If the tag of the block in the cache matches the tag of the block
    /// being accessed, then the block is in the cache.
    pub tag: u64,
    /// The index of the block. This is the middle bits of the address.
    /// The index is used to determine which set of the cache the block
    /// is in.
    pub index: u64,
    /// The offset in the block. This is the lowermost bits of the address.
    /// The offset is used to locate the value accessed in the cache line.
    pub offset: u64,

    /// The number of bits in the tag.
    pub tag_bits: u64,
    /// The number of bits in the index.
    pub index_bits: u64,
    /// The number of bits in the offset.
    pub offset_bits: u64,
}

impl BlockAddress {
    pub fn new(address: u64, index_bits: u64, offset_bits: u64) -> Self {
        let tag = address >> (index_bits + offset_bits);
        let index = (address >> offset_bits) & ((1 << index_bits) - 1);
        let offset = address & ((1 << offset_bits) - 1);

        let tag_bits = 32 - index_bits - offset_bits;

        // eprintln!("address: {x:08x} {x:032b}", x = address);
        // eprintln!(
        //     "tag:     {x:08x} {x:0bits$b}",
        //     x = tag,
        //     bits = tag_bits as usize
        // );
        // let space = " ".repeat(tag_bits as usize);
        // eprintln!(
        //     "index:   {x:08x} {space}{x:0bits$b}",
        //     x = index,
        //     bits = index_bits as usize
        // );
        // if offset_bits > 0 {
        //     let space = " ".repeat((tag_bits + index_bits) as usize);
        //     eprintln!(
        //         "offset:  {x:08x} {space}{x:0bits$b}",
        //         x = offset,
        //         bits = offset_bits as usize
        //     );
        // }
        Self {
            tag_bits,
            index_bits,
            offset_bits,

            tag,
            index,
            offset,
        }
    }

    pub fn get_address(&self) -> u64 {
        (self.tag << (self.offset_bits + self.index_bits))
            | (self.index << self.offset_bits)
            | self.offset
    }

    pub fn new_data_cache_address(address: u64, config: &SimulatorConfig) -> Self {
        let index_bits = config.get_data_cache_index_bits();
        let offset_bits = config.get_data_cache_offset_bits();
        Self::new(address, index_bits, offset_bits)
    }

    pub fn new_l2_cache_address(address: u64, config: &SimulatorConfig) -> Self {
        let index_bits = config.get_l2_cache_index_bits();
        let offset_bits = config.get_l2_cache_offset_bits();
        Self::new(address, index_bits, offset_bits)
    }

    pub fn new_page_table_address(address: u64, config: &SimulatorConfig) -> Self {
        let index_bits = config.get_page_table_index_bits();
        let offset_bits = config.get_page_table_offset_bits();
        Self::new(address, index_bits, offset_bits)
    }

    pub fn new_tlb_address(address: u64, config: &SimulatorConfig) -> Self {
        let index_bits = config.get_tlb_index_bits();
        let page_number =
            (address & !(config.get_page_size() - 1)) >> config.get_page_table_offset_bits();
        Self::new(page_number, index_bits, 0)
    }
}

impl Display for BlockAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let addr = (self.tag << self.index_bits + self.offset_bits)
            | (self.index << self.offset_bits)
            | self.offset;
        write!(f, "{:03x}", addr)
    }
}

/// A trace of memory access operations to be performed by the simulator.
#[derive(Clone, Debug)]
pub struct Trace {
    /// The memory access operations to be performed by the simulator.
    pub operations: Vec<Operation>,
}

impl Trace {
    /// Creates a new, empty trace.
    pub fn new() -> Self {
        Self {
            operations: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.operations.len()
    }

    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }

    pub fn get(&self, index: usize) -> Option<Operation> {
        self.operations.get(index).copied()
    }

    /// Reads a trace from stdin.
    pub fn from_stdin() -> Self {
        let mut trace = Self::new();
        while let Some(operation) = Operation::from_stdin() {
            trace.operations.push(operation);
        }
        trace
    }

    pub fn from_file(filename: &str) -> Self {
        let file = std::fs::File::open(filename).unwrap();
        let mut buffer = BufReader::new(file);
        let mut trace = Self::new();
        while let Some(operation) = Operation::from_buffer(&mut buffer) {
            trace.operations.push(operation);
        }
        trace
    }

    pub fn iter(&self) -> impl Iterator<Item = &Operation> {
        self.operations.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Operation> {
        self.operations.iter_mut()
    }

    pub fn into_iter(self) -> impl Iterator<Item = Operation> {
        self.operations.into_iter()
    }

    pub fn push(&mut self, operation: Operation) {
        self.operations.push(operation);
    }
}

impl Default for Trace {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoIterator for Trace {
    type Item = Operation;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.operations.into_iter()
    }
}

impl Display for Trace {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        for (i, operation) in self.operations.iter().enumerate() {
            write!(f, "{}", operation)?;
            if i != self.operations.len() - 1 {
                writeln!(f)?;
            }
        }
        Ok(())
    }
}

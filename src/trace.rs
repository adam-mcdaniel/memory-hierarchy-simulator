use core::fmt::{Display, Formatter, Result as FmtResult};

use super::{get_header, get_decimal, get_hexadecimal, get_bool};

#[derive(Clone, Copy, Debug)]
pub enum Operation {
    Read(u64),
    Write(u64),
}

impl Operation {
    fn from_stdin() -> Option<Self> {
        // <accesstype>:<hexaddress>
        // where <accesstype> is either R or W
        let stdin = std::io::stdin();
        let buffer = &mut std::io::BufReader::new(stdin.lock());
        
        let (access_type, address) = get_hexadecimal(buffer, None);

        match access_type.as_str() {
            "R" => Some(Self::Read(address)),
            "W" => Some(Self::Write(address)),
            _ => None,
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

#[derive(Clone, Copy, Debug)]
pub struct BlockAddress {
    pub tag: u64,
    pub index: u64,
    pub offset: u64,
}

#[derive(Clone, Debug)]
pub struct Trace {
    pub operations: Vec<Operation>,
}

impl Trace {
    pub fn new() -> Self {
        Self {
            operations: Vec::new(),
        }
    }

    pub fn from_stdin() -> Self {
        let mut trace = Self::new();
        while let Some(operation) = Operation::from_stdin() {
            trace.operations.push(operation);
        }
        trace
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
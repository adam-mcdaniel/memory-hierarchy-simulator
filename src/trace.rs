use core::fmt::{Display, Formatter, Result as FmtResult};

#[derive(Clone, Copy, Debug)]
pub enum Operation {
    Read(u64),
    Write(u64),
}

impl Operation {
    fn from_stdin() -> Option<Self> {
        // <accesstype>:<hexaddress>
        // where <accesstype> is either R or W
        let mut line = String::new();
        std::io::stdin().read_line(&mut line).unwrap();
        let mut split = line.split(':');
        if split.clone().count() != 2 {
            return None;
        }
        let access_type = split.next().unwrap();
        let address = split.next().unwrap();
        match access_type {
            "R" => Some(Self::Read(u64::from_str_radix(address.trim(), 16).unwrap())),
            "W" => Some(Self::Write(u64::from_str_radix(address.trim(), 16).unwrap())),
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
use super::*;
use core::fmt::{Display, Formatter, Result as FmtResult};
use log::{debug, error, info, trace, warn};

pub struct AccessOutput {
    pub access: Operation,

    /// The virtual address.
    pub virtual_address: Option<u64>,
    /// The physical address.
    pub physical_address: u64,
    /// The virtual page number of the virtual address (if there is one.)
    pub virtual_page_number: Option<u64>,
    /// The page offset of the address.
    pub page_offset: u64,
    /// The cache address in the TLB.
    pub tlb_address: Option<BlockAddress>,
    /// Did the TLB access result in a hit?
    pub tlb_hit: Option<bool>,
    /// Was the page table access a hit? (if there was one.)
    pub page_table_hit: Option<bool>,
    /// Physical page number of the address.
    pub physical_page_number: u64,
    /// The address into the data cache
    pub dc_address: BlockAddress,
    /// Was the data cache access a hit?
    pub dc_hit: bool,
    /// The address into the L2 cache
    pub l2_address: Option<BlockAddress>,
    /// Was the L2 cache acces a hit? (if the l2 cache is enabled)
    pub l2_hit: Option<bool>,
}


impl AccessOutput {
    pub fn get_virtual_address(&self) -> Option<u64> {
        self.virtual_address
    }

    pub fn get_page_offset(&self) -> u64 {
        self.page_offset
    }

    pub fn get_tlb_tag(&self) -> Option<u64> {
        self.tlb_address.map(|addr| addr.tag)
    }

    pub fn get_tlb_index(&self) -> Option<u64> {
        self.tlb_address.map(|addr| addr.index)
    }

    pub fn get_tlb_hit(&self) -> Option<bool> {
        self.tlb_hit
    }

    pub fn get_page_table_hit(&self) -> Option<bool> {
        self.page_table_hit
    }

    pub fn get_physical_page_number(&self) -> u64 {
        self.physical_page_number
    }

    pub fn get_dc_tag(&self) -> u64 {
        self.dc_address.tag
    }

    pub fn get_dc_index(&self) -> u64 {
        self.dc_address.index
    }

    pub fn get_dc_hit(&self) -> bool {
        self.dc_hit
    }

    pub fn get_l2_tag(&self) -> Option<u64> {
        self.l2_address.map(|addr| addr.tag)
    }

    pub fn get_l2_index(&self) -> Option<u64> {
        self.l2_address.map(|addr| addr.index)
    }

    pub fn get_l2_hit(&self) -> Option<bool> {
        self.l2_hit
    }
}

impl Display for AccessOutput {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self.get_virtual_address() {
            Some(addr) => write!(f, "{addr:08x}"),
            None => write!(f, "{}", " ".repeat(8)),
        }?;
        write!(f, " ")?;
        match self.virtual_page_number {
            Some(vpn) => write!(f, "{vpn:>6x}"),
            None => write!(f, "{}", " ".repeat(6)),
        }?;
        write!(f, " {:>4x} ", self.page_offset)?;

        match self.get_tlb_tag() {
            Some(tag) => write!(f, "{tag:>6x}"),
            None => write!(f, "{}", " ".repeat(6)),
        }?;
        write!(f, " ")?;
        match self.get_tlb_index() {
            Some(idx) => write!(f, "{idx:>3x}"),
            None => write!(f, "{}", " ".repeat(3)),
        }?;
        write!(f, " ")?;
        match self.tlb_hit {
            Some(hit) => write!(f, "{}", if hit { "hit " } else { "miss" }),
            _ => write!(f, "{}", " ".repeat(4)),
        }?;
        write!(f, " ")?;
        match self.page_table_hit {
            Some(hit) if self.tlb_hit != Some(true) => {
                write!(f, "{}", if hit { "hit " } else { "miss" })
            }
            _ => write!(f, "{}", " ".repeat(4)),
        }?;
        write!(
            f,
            " {:>4x} {:>6x} {:>3x} {:>4} ",
            self.physical_page_number,
            self.get_dc_tag(),
            self.get_dc_index(),
            if self.dc_hit { "hit " } else { "miss" }
        )?;
        match self.get_l2_tag() {
            Some(tag) if !self.dc_hit => write!(f, "{tag:>6x}"),
            _ => write!(f, "{}", " ".repeat(6)),
        }?;
        write!(f, " ")?;
        match self.get_l2_index() {
            Some(idx) if !self.dc_hit => write!(f, "{idx:>3x}"),
            _ => write!(f, "{}", " ".repeat(3)),
        }?;
        write!(f, " ")?;
        match self.l2_hit {
            Some(hit) if !self.dc_hit => write!(f, "{}", if hit { "hit " } else { "miss" }),
            _ => write!(f, "{}", " ".repeat(4)),
        }
    }
}

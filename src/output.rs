use super::*;
use core::fmt::{Display, Formatter, Result as FmtResult};
use log::{debug, error, info, trace, warn};

#[derive(Clone, Default)]
pub struct SimulatorOutput {
    pub config: SimulatorConfig,
    pub accesses: Vec<AccessOutput>,
    
    pub tlb_hits: u64,
    pub tlb_misses: u64,
    pub tlb_hit_ratio: f64,

    pub pt_hits: u64,
    pub pt_faults: u64,
    pub pt_hit_ratio: f64,

    pub dc_hits: u64,
    pub dc_misses: u64,
    pub dc_hit_ratio: f64,
    
    pub l2_hits: u64,
    pub l2_misses: u64,
    pub l2_hit_ratio: f64,

    pub total_reads: u64,
    pub total_writes: u64,
    pub ratio_of_reads: f64,

    /// The number of main memory references
    pub main_memory_refs: u64,
    /// The number of TLB misses
    pub page_table_refs: u64,
    /// This is equal to the number of page faults
    pub disk_refs: u64,
}

impl SimulatorOutput {
    pub fn empty(config: SimulatorConfig) -> Self {
        Self {
            config,
            .. Default::default()
        }
    }

    pub fn add_access(&mut self, access: AccessOutput) {
        if access.access.is_read() {
            self.total_reads += 1;
        } else {
            self.total_writes += 1;
        }
        self.accesses.push(access);
    }

    pub fn add_main_memory_access(&mut self) {
        self.main_memory_refs += 1;
    }

    pub fn add_main_memory_accesses(&mut self, count: u64) {
        self.main_memory_refs += count;
    }

    pub fn add_tlb_access(&mut self, hit: bool) {
        if !self.config.is_tlb_enabled() { return }

        if hit {
            self.tlb_hits += 1;
        } else {
            self.tlb_misses += 1;
        }
    }

    pub fn add_page_table_access(&mut self, hit: bool) {
        if !self.config.is_virtual_addresses_enabled() { return }

        if hit {
            self.pt_hits += 1;
        } else {
            self.pt_faults += 1;
        }
    }

    pub fn add_dc_access(&mut self, hit: bool) {
        if hit {
            self.dc_hits += 1;
        } else {
            self.dc_misses += 1;
        }
    }

    pub fn add_l2_access(&mut self, hit: bool) {
        if !self.config.is_l2_cache_enabled() { return }

        if hit {
            self.l2_hits += 1;
        } else {
            self.l2_misses += 1;
        }
    }

    pub fn add_l2_accesses(&mut self, count: u64) {
        if !self.config.is_l2_cache_enabled() { return }

        self.l2_hits += count;
    }
}

impl Display for SimulatorOutput {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        writeln!(f, "{}", self.config)?;
        writeln!(f,
            r#"{} Virt.  Page TLB    TLB TLB  PT   Phys        DC  DC          L2  L2
Address  Page # Off  Tag    Ind Res. Res. Pg # DC Tag Ind Res. L2 Tag Ind Res.
-------- ------ ---- ------ --- ---- ---- ---- ------ --- ---- ------ --- ----"#,
            if self.config.is_virtual_addresses_enabled() {
                "Virtual "
            } else {
                "Physical"
            }
        )?;

        let mut main_mem_accesses = self.main_memory_refs;
        for access in &self.accesses {
            main_mem_accesses += access.get_main_memory_accesses(&self.config);
            writeln!(f, "{}", access)?;
        }

        
        writeln!(f, "\nSimulation statistics\n")?;
        let hit_ratio = |hits, misses| hits as f64 / ((hits + misses) as f64).max(0.0000001);

        writeln!(f, "dtlb hits        : {}", self.tlb_hits)?;
        writeln!(f, "dtlb misses      : {}", self.tlb_misses)?;
        if self.config.is_tlb_enabled() {
            writeln!(f, "dtlb hit ratio   : {:1.6}\n", hit_ratio(self.tlb_hits, self.tlb_misses))?;
        } else {
            writeln!(f, "dtlb hit ratio   : N/A\n")?;
        }

        writeln!(f, "pt hits          : {}", self.pt_hits)?;
        writeln!(f, "pt faults        : {}", self.pt_faults)?;
        
        if self.config.is_virtual_addresses_enabled() {
            writeln!(f, "pt hit ratio     : {:1.6}\n", hit_ratio(self.pt_hits, self.pt_faults))?;
        } else {
            writeln!(f, "pt hit ratio     : N/A\n")?;
        }

        writeln!(f, "dc hits          : {}", self.dc_hits)?;
        writeln!(f, "dc misses        : {}", self.dc_misses)?;
        writeln!(f, "dc hit ratio     : {:1.6}\n", hit_ratio(self.dc_hits, self.dc_misses))?;

        writeln!(f, "L2 hits          : {}", self.l2_hits)?;
        writeln!(f, "L2 misses        : {}", self.l2_misses)?;
        if self.config.is_l2_cache_enabled() {
            writeln!(f, "L2 hit ratio     : {:1.6}\n", hit_ratio(self.l2_hits, self.l2_misses))?;
        } else {
            writeln!(f, "L2 hit ratio     : N/A\n")?;
        }

        writeln!(f, "Total reads      : {}", self.total_reads)?;
        writeln!(f, "Total writes     : {}", self.total_writes)?;
        writeln!(f, "Ratio of reads   : {:1.6}\n", hit_ratio(self.total_reads, self.total_writes))?;

        writeln!(f, "main memory refs : {}", main_mem_accesses)?;
        writeln!(f, "page table refs  : {}", self.pt_hits + self.pt_faults)?;
        write!(f, "disk refs        : {}", self.pt_faults)?;

        Ok(())
    }
}



#[derive(Clone, Copy, Debug)]
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
    pub fn get_main_memory_accesses(&self, config: &SimulatorConfig) -> u64 {
        if self.access.is_read() {
            if self.dc_hit {
                0
            } else if self.l2_hit == Some(true) {
                0
            } else {
                1
            }
        } else {
            // Access is a write
            if self.dc_hit {
                if config.data_cache.is_write_through() && config.l2_cache.is_write_through() {
                    1
                } else if config.data_cache.is_write_through() && config.l2_cache.is_write_back() {
                    0
                } else if config.data_cache.is_write_back() && config.l2_cache.is_write_through() {
                    1
                } else if config.data_cache.is_write_back() && config.l2_cache.is_write_back() {
                    0
                } else {
                    1
                }
            } else if self.l2_hit == Some(true) {
                if config.l2_cache.is_write_through() {
                    1
                } else {
                    0
                }
            } else {
                1
            }
        }
    }

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
            None => write!(f, "{:08x}", self.physical_address),
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
        if self.l2_hit == None {
            // return write!(f, " ");
            return Ok(());
        }
        match self.get_l2_tag() {
            Some(tag) => write!(f, "{tag:>6x} "),
            _ => Ok(()),
        }?;
        match self.get_l2_index() {
            Some(idx) => write!(f, "{idx:>3x} "),
            _ => Ok(()),
        }?;
        match self.l2_hit {
            Some(hit) => write!(f, "{}", if hit { "hit " } else { "miss" }),
            _ => Ok(()),
        }
    }
}

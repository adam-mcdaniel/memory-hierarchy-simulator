use super::*;
use log::{info, trace};

pub struct Simulator {
    l2: Option<L2Cache>,
    dc: DataCache,
    tlb: Option<TLBCache>,
    page_table: Option<PageTable>,
    config: SimulatorConfig,
    time: u64,
    output: SimulatorOutput,
}

impl From<SimulatorConfig> for Simulator {
    fn from(config: SimulatorConfig) -> Self {
        Self {
            output: SimulatorOutput::empty(config.clone()),
            l2: config
                .is_l2_cache_enabled()
                .then_some(L2Cache::new_from_config(&config)),
            dc: DataCache::new_from_config(&config),
            tlb: config
                .is_tlb_enabled()
                .then_some(TLBCache::new_from_config(&config)),
            page_table: config
                .is_virtual_addresses_enabled()
                .then_some(PageTable::new_from_config(&config)),
            config,
            time: 1,
        }
    }
}

impl Simulator {
    fn health_check(&self) -> Result<(), ()> {
        if self.config.is_virtual_addresses_enabled() != self.get_page_table().is_some() {
            return Err(());
        }

        if self.config.tlb_enabled != self.get_tlb().is_some() {
            return Err(());
        }

        if self.config.is_l2_cache_enabled() != self.get_l2().is_some() {
            return Err(());
        }

        Ok(())
    }

    pub fn get_l2(&self) -> Option<&L2Cache> {
        // assert!(self.health_check().is_ok());
        self.l2.as_ref()
    }

    pub fn get_dc(&self) -> &DataCache {
        // assert!(self.health_check().is_ok());
        &self.dc
    }

    pub fn get_tlb(&self) -> Option<&TLBCache> {
        // assert!(self.health_check().is_ok());
        self.tlb.as_ref()
    }

    pub fn get_page_table(&self) -> Option<&PageTable> {
        // assert!(self.health_check().is_ok());
        self.page_table.as_ref()
    }

    pub fn get_config(&self) -> &SimulatorConfig {
        // assert!(self.health_check().is_ok());
        &self.config
    }

    pub fn get_l2_mut(&mut self) -> Option<&mut L2Cache> {
        // assert!(self.health_check().is_ok());
        self.l2.as_mut()
    }

    pub fn get_dc_mut(&mut self) -> &mut DataCache {
        // assert!(self.health_check().is_ok());
        &mut self.dc
    }

    pub fn get_tlb_mut(&mut self) -> Option<&mut TLBCache> {
        // assert!(self.health_check().is_ok());
        self.tlb.as_mut()
    }

    pub fn get_page_table_mut(&mut self) -> Option<&mut PageTable> {
        // assert!(self.health_check().is_ok());
        self.page_table.as_mut()
    }

    pub fn get_time(&self) -> u64 {
        self.time
    }

    fn age(&mut self) {
        self.time += 1;
        info!("Time is now {time}", time = self.time);
    }

    pub fn simulate(&mut self, trace: Trace) -> SimulatorOutput {
        self.output = SimulatorOutput::empty(self.config.clone());
        for access in trace {
            self.simulate_access(access);
        }
        self.output.clone()
    }

    pub fn simulate_access(&mut self, access: Operation) -> AccessOutput {
        assert!(self.health_check().is_ok());
        let virtual_address = access.address();
        let physical_address;
        
        let time = self.get_time();
        trace!("Access {access} at {time}");
        let mut is_tlb_hit;
        let tlb_address;
        let is_page_table_hit;
        match (&mut self.tlb, &mut self.page_table) {
            (Some(tlb), Some(page_table)) => {
                let addr = BlockAddress::new_tlb_address(virtual_address, &self.config);
                tlb_address = Some(addr);
                // info!("About to translate TLB address...");
                is_tlb_hit = tlb.translate(addr, time);
                // info!("About to translate page table address...");
                (physical_address, is_page_table_hit) =
                    page_table.translate(virtual_address, time).unwrap();
                is_tlb_hit = is_tlb_hit && is_page_table_hit;
            }
            (None, Some(page_table)) => {
                is_tlb_hit = false;
                tlb_address = None;
                // info!("About to translate page table address...");
                (physical_address, is_page_table_hit) =
                    page_table.translate(virtual_address, time).unwrap();
            }
            _ => {
                physical_address = virtual_address;
                tlb_address = None;
                is_tlb_hit = false;
                is_page_table_hit = false;
            }
        }
        if self.config.is_tlb_enabled() {
            self.output.add_tlb_access(is_tlb_hit);
        }
        if !is_tlb_hit && self.config.is_virtual_addresses_enabled() {
            self.output.add_page_table_access(is_page_table_hit);
        }
        let is_page_fault =
            self.config.is_virtual_addresses_enabled() && !is_tlb_hit && !is_page_table_hit;

        if is_page_fault {
            // let count = self.dc.invalidate_page(physical_address & !(self.config.get_page_size() - 1), &self.config);
            if let (Some(tlb), Some(pt)) = (&mut self.tlb, &mut self.page_table) {
                let blocks = tlb.invalidate_page(physical_address, pt, &self.config);
                let count = blocks.len();
                if count > 0 {
                    eprintln!("Evicted {count} pages from the TLB");
                }

            }
            let blocks = self.dc.invalidate_page(physical_address, &self.config);

            let count = blocks.len();
            if count > 0 {
                eprintln!("Evicted {count} pages from the DC");
            }
            // self.output.add_main_memory_accesses(count as u64);

            if self.config.data_cache.is_write_back() && !self.config.is_l2_cache_enabled() {
                // self.output.add_main_memory_accesses(count as u64);
            } else if let Some(l2) = &mut self.l2 {
                // let count = l2.invalidate_page(physical_address, &self.config);
                let blocks = l2.invalidate_page(physical_address, &self.config);
                let count = blocks.len();
                // if self.config.l2_cache.is_write_back() {
                //     self.output.add_main_memory_accesses(count as u64);
                // }
                if count > 0 {
                    eprintln!("Evicted {count} pages from the L2");
                }
            }
        }

        let dc_address = BlockAddress::new_data_cache_address(physical_address, &self.config);
        let dc_hit = self.dc.access(access.is_read(), dc_address, time);
        self.output.add_dc_access(dc_hit);

        let l2_address;
        let l2_hit;
        match &mut self.l2 {
            Some(l2) => {
                let addr = BlockAddress::new_l2_cache_address(physical_address, &self.config);
                l2_address = Some(addr);
                // if self.config.l2_cache.is_write_back() {
                //     let result = l2.access(access.is_read(), addr, time);
                //     self.output.add_l2_access(result);
                //     l2_hit = Some(result);
                // } else if !dc_hit || ((self.config.data_cache.is_write_through() || self.config.l2_cache.is_write_through()) && access.is_write()) {
                // } else {
                //     l2_hit = None;
                // }
                // if dc_hit && (self.config.data_cache.is_write_back() || self.config.l2_cache.is_write_back()) {
                //     l2.access(access.is_read(), addr, time);
                //     l2_hit = None
                // } else if !dc_hit {
                //     let result = l2.access(access.is_read(), addr, time);
                //     self.output.add_l2_access(result);
                //     l2_hit = Some(result);
                // } else {
                //     l2_hit = None
                // }

                // if !dc_hit || (dc_hit && (self.config.data_cache.is_write_back() || self.config.l2_cache.is_write_back())) {
                //     let result = l2.access(access.is_read(), addr, time);
                //     self.output.add_l2_access(result);
                //     l2_hit = Some(result);
                // } else {
                //     l2_hit = None
                // }


                if self.config.data_cache.is_write_through() && self.config.l2_cache.is_write_through() {
                    // Good, do not change!
                    if !dc_hit || access.is_write() {
                        let result = l2.access(access.is_read(), addr, time);
                        self.output.add_l2_access(result);
                        l2_hit = Some(result);
                    } else {
                        l2_hit = None;
                    }
                } else if self.config.data_cache.is_write_through() && self.config.l2_cache.is_write_back() {
                    // This works a little, but there is a bug with DC hits
                    // With only writes, or only reads, this works!
                    if !dc_hit || (dc_hit && access.is_write()) {
                        let result = l2.access(access.is_read(), addr, time);
                        self.output.add_l2_access(result);
                        l2_hit = Some(result);
                    } else {
                        l2_hit = None;
                    }
                } else if self.config.data_cache.is_write_back() && self.config.l2_cache.is_write_through() {
                    // let result = l2.access(access.is_read(), addr, time);
                    // self.output.add_l2_access(result);
                    // if dc_hit {
                    //     l2_hit = None;
                    // } else {
                    //     l2_hit = Some(result);
                    // }
                    if access.is_write() {
                        let result = l2.access(access.is_read(), addr, time);
                        self.output.add_l2_access(result);
                        if !dc_hit {
                            l2_hit = Some(result);
                        } else {
                            l2_hit = None;
                        }
                    } else {
                        if !dc_hit {
                            let result = l2.access(access.is_read(), addr, time);
                            self.output.add_l2_access(result);
                            l2_hit = Some(result);
                        } else {
                            l2_hit = None;
                        }
                    }
                } else if self.config.data_cache.is_write_back() && self.config.l2_cache.is_write_back() {
                    if !dc_hit || access.is_write() {
                        let result = l2.access(access.is_read(), addr, time);
                        assert!(self.dc.access(access.is_read(), dc_address, time));
                        self.output.add_l2_access(result);
                        if dc_hit {
                            l2_hit = None;
                        } else {
                            l2_hit = Some(result);
                        }
                    } else {
                        l2_hit = None
                    }
                } else {
                    unreachable!()
                }
            }
            None => {
                l2_address = None;
                l2_hit = None;
            }
        }

        let to_page_number = |addr| {
            (addr & !(self.config.get_page_size() - 1))
                >> (self.config.get_page_size().trailing_zeros())
        };

        let virtual_address = self
            .config
            .is_virtual_addresses_enabled()
            .then_some(virtual_address);
        let virtual_page_number = virtual_address.map(to_page_number);
        let physical_page_number = to_page_number(physical_address);

        let page_offset = physical_address & (self.config.get_page_size() - 1);
        self.age();

        let result = AccessOutput {
            access,
            virtual_address,
            physical_address,
            virtual_page_number,
            physical_page_number,
            page_offset,
            tlb_address,
            tlb_hit: self.config.is_tlb_enabled().then_some(is_tlb_hit),
            page_table_hit: self
                .config
                .is_virtual_addresses_enabled()
                .then_some(is_page_table_hit),
            dc_address,
            dc_hit,
            l2_address,
            l2_hit,
        };

        self.output.add_access(result);

        result
    }
}

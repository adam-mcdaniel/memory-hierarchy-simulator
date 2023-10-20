use super::*;
use log::{debug, info, trace, warn};

pub struct TLBCache {
    cache: Cache,
}

impl TLBCache {
    pub fn new(
        sets: usize,
        block_size: u64,
        associativity: u64,
        evict_policy: EvictionPolicy,
    ) -> Self {
        info!("Creating new TLBCache with {sets} sets, associativity={associativity}, block-size={block_size}, policy={evict_policy:?}");
        Self {
            cache: Cache::new(sets, block_size, associativity, evict_policy),
        }
    }

    pub fn new_from_config(config: &SimulatorConfig) -> Self {
        let number_of_sets = config.tlb.get_number_of_sets();
        let entries_in_set = config.tlb.get_entries_in_set();
        let entry_size = config.get_page_size();
        let block_size = entry_size;
        let evict_policy = config.tlb.get_eviction_policy();
        Self::new(
            number_of_sets as usize,
            block_size,
            entries_in_set,
            evict_policy,
        )
    }

    /// Try to translate the address using the TLB. This function
    /// returns whether or not the translation was a hit.
    pub fn translate(&mut self, address: BlockAddress, current_access_time: u64) -> bool {
        self.cache
            .is_read_and_allocate_hit(address, current_access_time)
    }


    /// Invalidate a physical page from the cache. This gets all the blocks loaded from
    /// the page, and then invalidates them in the cache.
    /// This returns the number of invalidate pages.
    pub fn invalidate_page(&mut self, physical_address: u64, page_table: &mut PageTable, config: &SimulatorConfig) -> Vec<Block> {
        // Invalidate all the blocks associated with this physical page.
        // let page_number = physical_address / config.get_page_size();

        let mut invalidated_blocks = Vec::new();

        for entry in page_table.get_entries() {
            if entry.get_physical_address() == physical_address {
                let addr = entry.get_virtual_address();
                if let Some(block) = self.cache.invalidate(BlockAddress::new_tlb_address(addr, config)) {
                    trace!("Invalidated TLB block {block:?}");
                    invalidated_blocks.push(block);
                }
            }
        }
        // page_table.invalidate_page_number(physical_address / config.get_page_size());
        let number_of_blocks = invalidated_blocks.len();
        debug!("TLB Invalidating {number_of_blocks} blocks = {} bytes at {physical_address:x}", number_of_blocks as u64 * config.data_cache.get_block_size());
        
        invalidated_blocks
    }
}

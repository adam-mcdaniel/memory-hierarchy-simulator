use super::*;
use log::{debug, info, trace, warn};

pub struct DataCache {
    cache: Cache,
    is_write_allocate: bool,
    total_read_misses: u64,
    total_write_misses: u64,
    total_reads: u64,
    total_writes: u64,
}

impl DataCache {
    /// Create a new cache.
    fn new(
        sets: usize,
        block_size: u64,
        associativity: u64,
        evict_policy: EvictionPolicy,
        is_write_allocate: bool,
    ) -> Self {
        info!("Creating new DataCache with {sets} sets, block-size={block_size}, associativity={associativity}, and policy={evict_policy:?}");
        Self {
            cache: Cache::new(sets, block_size, associativity, evict_policy),
            is_write_allocate,
            total_read_misses: 0,
            total_write_misses: 0,
            total_reads: 0,
            total_writes: 0,
        }
    }

    /// Create a new fully associative cache.
    pub fn new_fully_associative(
        size_in_bytes: u64,
        block_size: u64,
        evict_policy: EvictionPolicy,
        is_write_allocate: bool,
    ) -> Self {
        let number_of_sets = size_in_bytes / block_size;
        Self::new(
            number_of_sets as usize,
            block_size,
            number_of_sets,
            evict_policy,
            is_write_allocate,
        )
    }

    /// Create a new direct-mapped cache.
    pub fn new_direct_mapped(
        size_in_bytes: u64,
        block_size: u64,
        evict_policy: EvictionPolicy,
        is_write_allocate: bool,
    ) -> Self {
        let number_of_sets = size_in_bytes / block_size;
        Self::new(
            number_of_sets as usize,
            block_size,
            1,
            evict_policy,
            is_write_allocate,
        )
    }

    /// Create a new set-associative cache.
    pub fn new_set_associative(
        associativity: u64,
        size_in_bytes: u64,
        block_size: u64,
        evict_policy: EvictionPolicy,
        is_write_allocate: bool,
    ) -> Self {
        let number_of_sets = (size_in_bytes / block_size) / associativity;
        Self::new(
            number_of_sets as usize,
            block_size,
            associativity,
            evict_policy,
            is_write_allocate,
        )
    }

    /// Create a new data cache from the configuration file.
    pub fn new_from_config(config: &SimulatorConfig) -> Self {
        let number_of_sets = config.data_cache.get_number_of_sets();
        let associativity = config.data_cache.get_associativity();
        let block_size = config.data_cache.get_block_size();
        let evict_policy = config.data_cache.get_eviction_policy();
        let is_write_allocate = config.data_cache.is_write_allocate();
        Self::new(
            number_of_sets as usize,
            block_size,
            associativity,
            evict_policy,
            is_write_allocate,
        )
    }

    /// Write to a block in the data cache.
    /// This will return whether or not the write was a hit.
    pub fn write(&mut self, address: BlockAddress, current_access_time: u64) -> bool {
        self.total_writes += 1;
        let result = if self.is_write_allocate {
            self.cache
                .is_write_and_allocate_hit(address, current_access_time)
        } else {
            self.cache.try_write(address, current_access_time)
        };

        // If the result was not a hit, increment the miss count
        if !result {
            self.total_write_misses += 1;
        }

        result
    }

    /// Read a block in the data cache.
    /// This will return whether or not the read was a hit.
    pub fn read(&mut self, address: BlockAddress, current_access_time: u64) -> bool {
        self.total_reads += 1;
        let result = self
            .cache
            .is_read_and_allocate_hit(address, current_access_time);
        if !result {
            self.total_read_misses += 1
        }

        result
    }

    /// Perform an access operation.
    /// This returns whether or not the operation was a hit.
    pub fn access(
        &mut self,
        is_read: bool,
        address: BlockAddress,
        current_access_time: u64,
    ) -> bool {
        if is_read {
            self.read(address, current_access_time)
        } else {
            self.write(address, current_access_time)
        }
    }

    /// Invalidate a physical page from the cache. This gets all the blocks loaded from
    /// the page, and then invalidates them in the cache.
    /// This returns the number of invalidate pages.
    pub fn invalidate_page(&mut self, physical_address: u64, config: &SimulatorConfig) -> usize {
        let block_size = config.data_cache.get_block_size();
        let page_size = config.get_page_size();
        let number_of_blocks = (page_size / block_size) as usize;
        assert!(number_of_blocks as u64 * block_size == page_size);
        let mut invalidated_block_count = 0;
        for block in 0..number_of_blocks {
            let block_address = BlockAddress::new_data_cache_address(
                physical_address + block as u64 * block_size,
                config,
            );

            if let Some(block) = self.cache.invalidate(block_address) {
                invalidated_block_count += 1;
                trace!("Invalidated DC block {block:?}");
            }
        }
        invalidated_block_count
    }
}

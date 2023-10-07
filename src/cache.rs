use super::BlockAddress;
use log::{debug, error, info, trace, warn};

/// This encodes the eviction policy for a generic cache.
/// Whenever a block is evicted from a set, this is used to
/// select the block to evict from the given set.
#[derive(Copy, Clone, Debug)]
pub enum EvictionPolicy {
    /// Evict the least recently used block from the set.
    LRU,
    /// Evict the least recently block loaded into the set (first-in-first-out).
    FIFO,
    /// Evict a random a block from the set.
    Random,
}

impl EvictionPolicy {
    /// Return the tag of the block to evict.
    fn evict(&self, set: &mut Set) -> Option<Block> {
        // Is the set full?
        if !set.is_full() {
            // The set is not full, so there is no block to evict.
            trace!("No need to evict a block; set is not full");
            return None;
        }

        // Get the first block in the set
        let tags = set.get_tags();
        // If we've gotten to this point, there *must* must be at least one block
        // so the list of tags in the set cannot be empty.
        assert!(!tags.is_empty());
        // Get the tag of the first block in the set.
        let first_tag = tags[0];

        match self {
            // If we're using the first-in-first-out policy
            Self::FIFO => {
                // A variable to keep track of the oldest block we've seen so far
                let mut oldest_block_seen = *set.get_block_with_tag(first_tag).unwrap(); // Initialized to the first block in the set.

                // Find the block with the oldest fist access time (skip the first one we already grabbed)
                for tag in tags.iter().skip(1) {
                    let block = *set.get_block_with_tag(*tag).unwrap();
                    // Was this block created before the oldest block?
                    if block.first_access < oldest_block_seen.first_access {
                        oldest_block_seen = block;
                    }
                }

                // Return the block to evict
                let result = Some(oldest_block_seen);
                // Evict the block
                trace!("FIFO policy evicting block {oldest_block_seen:?}");
                set.evict_tag(oldest_block_seen.get_tag());

                result
            }

            Self::LRU => {
                // A variable to keep track of the oldest block we've seen so far
                let mut oldest_block_seen = *set.get_block_with_tag(first_tag).unwrap(); // Initialized to the first block in the set.

                // Find the block with the oldest last access time (skip the first one we already grabbed)
                for tag in tags.iter().skip(1) {
                    let block = *set.get_block_with_tag(*tag).unwrap();
                    // Was this block accessed before the oldest block?
                    // trace!("Checking block {block:?}");
                    if block.last_access < oldest_block_seen.last_access {
                        oldest_block_seen = block;
                    }
                }

                // Return the block to evict
                let result = Some(oldest_block_seen);
                // Evict the block
                trace!("LRU policy evicting block {oldest_block_seen:?}");
                set.evict_tag(oldest_block_seen.get_tag());
                // Return the block to evict
                result
            }

            Self::Random => {
                // Pick a random block to evict
                let random_index = rand::random::<usize>() % tags.len();
                let random_tag = tags[random_index];

                // Return the block to evict
                let result = *set.get_block_with_tag(random_tag).unwrap();
                // Evict the block
                trace!("Evicting random block with tag {random_tag:x}");
                set.evict_tag(random_tag);
                // Return the block to evict
                Some(result)
            }
        }
    }
}

/// A line in a cache.
/// This contains the data in the line, as well as the tag, index, offset,
/// dirty bit.
#[derive(Copy, Clone, Debug)]
pub struct Block {
    /// The tag of the block. This is the uppermost bits of the address.
    /// The tag is used to determine if the block is in the cache.
    /// If the tag of the block in the cache matches the tag of the block
    /// being accessed, then the block is in the cache.
    tag: u64,
    /// The index of the block. This is the middle bits of the address.
    /// The index is used to determine which set of the cache the block
    /// is in.
    index: u64,
    /// The dirty bit of the block. This is set to true if the block has
    /// been written to since it was loaded into the cache.
    dirty: bool,
    /// The size of the block in bytes.
    size: u64,
    /// Last access time of the block in cycles.
    last_access: u64,
    /// The first access time of the block in cycles.
    first_access: u64,
}

impl Block {
    /// Construct a new block with the given tag, set index, and the current access time
    /// of the creation of the block.
    pub fn new(tag: u64, index: u64, size: u64, current_access_time: u64) -> Self {
        trace!("Creating new block in set #{index} with tag={tag:x}, time={current_access_time}, size={size}");
        Self {
            tag,
            index,
            // The block is loaded in clean.
            dirty: false,
            size,
            last_access: current_access_time,
            first_access: current_access_time,
        }
    }

    /// Return the size of the block in bytes.
    pub fn size_in_bytes(&self) -> u64 {
        self.size
    }

    /// Return the tag of the block.
    /// This is the uppermost bits of the address.
    /// The tag is used to determine if the block is in the cache.
    /// If the tag of the block in the cache matches the tag of the block
    /// being accessed, then the block is in the cache.
    pub fn get_tag(&self) -> u64 {
        self.tag
    }

    /// Return the index of the block.
    /// This is the middle bits of the address.
    /// The index is used to determine which set of the cache the block
    /// is in.
    pub fn get_index(&self) -> u64 {
        self.index
    }

    /// Does this block address hit this block?
    pub fn is_hit(&self, address: BlockAddress) -> bool {
        self.tag == address.tag && self.index == address.index
    }

    /// Write to the block.
    /// This sets the dirty bit to true.
    /// This also updates the last access time.
    pub fn write(&mut self, current_access_time: u64) {
        trace!(
            "Wrote to block with tag={:x} in set #{}",
            self.get_tag(),
            self.get_index()
        );
        self.dirty = true;
        self.last_access = current_access_time;
    }

    /// Read the block.
    /// This also updates the last access time.
    /// This does not set the dirty bit.
    pub fn read(&mut self, current_access_time: u64) {
        trace!(
            "Read from block with tag={:x} in set #{}",
            self.get_tag(),
            self.get_index()
        );
        self.last_access = current_access_time;
    }
}

/// A set in a cache.
/// This contains the blocks in the set.
#[derive(Clone, Debug)]
pub struct Set {
    /// The blocks in the set.
    blocks: Vec<Option<Block>>,
    /// The size of the blocks in the set in bytes.
    block_size: u64,
    /// The eviction policy of the set.
    evict_policy: EvictionPolicy,
}

impl Set {
    /// Create a new set.
    pub fn new(block_size: u64, associativity: u64, evict_policy: EvictionPolicy) -> Self {
        trace!("Creating set with block-size={block_size}, associativity={associativity}, policy={evict_policy:?}");
        Self {
            blocks: vec![None; associativity as usize],
            block_size,
            evict_policy,
        }
    }

    /// Is full?
    pub fn is_full(&self) -> bool {
        self.blocks.iter().all(|block| block.is_some())
    }

    /// Return the number of blocks in the set.
    pub fn len(&self) -> usize {
        self.blocks.len()
    }

    /// Evict the block with the given tag.
    /// Return the block that was evicted.
    fn evict_tag(&mut self, tag: u64) -> Option<Block> {
        trace!("Evicting block with tag={tag:x}");
        // Find the block with the matching tag and index
        for block_slot in self.blocks.iter_mut() {
            let result = *block_slot;
            if let Some(present_block) = result {
                if present_block.tag == tag {
                    trace!("Evicting block {block_slot:?}");
                    *block_slot = None;
                    return result;
                }
            }
        }
        None
    }

    /// Insert the block into the set.
    /// If the set is full, then evict a block.
    /// This will return the block that was evicted, if any.
    fn allocate_block(&mut self, block: BlockAddress, current_access_time: u64) -> Option<Block> {
        let result = if self.is_full() {
            trace!("Set is full, evicting a block.");
            // Evict a block
            self.evict()
        } else {
            None
        };

        // Find the first empty block slot
        for block_slot in self.blocks.iter_mut() {
            if block_slot.is_none() {
                // Allocate the block
                *block_slot = Some(Block::new(
                    block.tag,
                    block.index,
                    self.block_size,
                    current_access_time,
                ));
                break;
            }
        }

        result
    }

    /// Evict a block from the set, and return the evicted block.
    pub fn evict(&mut self) -> Option<Block> {
        // Get the policy
        let policy = self.evict_policy;
        // Evict a block from the set using it
        if let Some(result) = policy.evict(self) {
            trace!("Evicting the block from set: {:?}", result);
            return Some(result);
        }
        None
    }

    /// Return the tags of the blocks in the set.
    pub fn get_tags(&self) -> Vec<u64> {
        self.blocks
            .iter()
            .filter_map(|block| {
                if let Some(block) = block {
                    Some(block.tag)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Return the block with the given tag.
    // fn get_block_with_tag_mut(&mut self, tag: u64) -> Option<&mut Block> {
    //     // Find the block with the matching tag and index
    //     for block in self.blocks.iter_mut() {
    //         if let Some(block) = block {
    //             if block.tag == tag {
    //                 return Some(block);
    //             }
    //         }
    //     }
    //     None
    // }

    /// Return the block with the given tag.
    fn get_block_with_tag(&mut self, tag: u64) -> Option<&Block> {
        // Find the block with the matching tag and index
        for block in self.blocks.iter() {
            if let Some(block) = block {
                if block.tag == tag {
                    return Some(block);
                }
            }
        }
        None
    }

    /// Get the block at the given address.
    /// Return None if the block is not in the set.
    /// Return Some(block) if the block is in the set.
    fn get_block_with_addr_mut(&mut self, block_address: BlockAddress) -> Option<&mut Block> {
        let tag = block_address.tag;
        for block in self.blocks.iter_mut() {
            if let Some(block) = block {
                if block.tag == tag {
                    return Some(block);
                }
            }
        }
        None
    }

    /// Get the block associated with the block address.
    fn get_block_with_addr(&self, block_address: BlockAddress) -> Option<&Block> {
        let tag = block_address.tag;
        for block in self.blocks.iter() {
            if let Some(block) = block {
                if block.tag == tag {
                    return Some(block);
                }
            }
        }
        None
    }

    /// Does this set contain the block at the given address?
    fn is_hit(&self, block_address: BlockAddress) -> bool {
        self.get_block_with_addr(block_address).is_some()
    }

    /// Try to write to the block at the given address.
    /// If the block is not in the set, then do nothing.
    /// Returns whether or not the write was a hit.
    pub fn try_write(&mut self, block_address: BlockAddress, current_access_time: u64) -> bool {
        if let Some(block) = self.get_block_with_addr_mut(block_address) {
            // If the block is already in the set, update the last access time
            block.write(current_access_time);
            return true;
        }
        false
    }

    /// Try to read the block at the given address.
    /// If the block is not in the set, then do nothing.
    /// Returns whether or not the read was a hit.
    pub fn try_read(&mut self, block_address: BlockAddress, current_access_time: u64) -> bool {
        if let Some(block) = self.get_block_with_addr_mut(block_address) {
            // If the block is already in the set, update the last access time
            block.read(current_access_time);
            return true;
        }
        false
    }

    /// Insert the block at the given address into the set.
    /// Return the old block that was replaced, if any.
    pub fn write_and_allocate(
        &mut self,
        block_address: BlockAddress,
        current_access_time: u64,
    ) -> Option<Block> {
        // Try writing to the block
        if self.try_write(block_address, current_access_time) {
            // If the operation succeeded, there was no block replaced so return None.
            return None;
        }
        // If the operation failed, allocate the block and then try again
        let result = self.allocate_block(block_address, current_access_time);
        // It *MUST* work after the block has been allocated.
        // Otherwise it was not allocated properly
        assert!(self.try_write(block_address, current_access_time));
        result
    }

    /// Read the block at the given address.
    /// This will update the last access time of the block.
    /// It will allocate the block if it is not already in the set.
    /// Return the old block that was replaced, if any.
    pub fn read_and_allocate(
        &mut self,
        block_address: BlockAddress,
        current_access_time: u64,
    ) -> Option<Block> {
        // Try reading the block
        if self.try_read(block_address, current_access_time) {
            // If the operation succeeded, there was no block replaced so return None.
            return None;
        }
        // If the operation failed, allocate the block and try again
        let result = self.allocate_block(block_address, current_access_time);
        // It *MUST* work after the block has been allocated.
        // Otherwise it was not allocated properly.
        assert!(self.try_read(block_address, current_access_time));
        result
    }

    /// Performs the write and allocate operation, and returns true if it was a write hit.
    pub fn is_write_and_allocate_hit(
        &mut self,
        block_address: BlockAddress,
        current_access_time: u64,
    ) -> bool {
        let is_hit = self.is_hit(block_address);
        self.write_and_allocate(block_address, current_access_time);
        is_hit
    }

    /// Performs the read and allocate operation, and returns true if it was a read hit.
    pub fn is_read_and_allocate_hit(
        &mut self,
        block_address: BlockAddress,
        current_access_time: u64,
    ) -> bool {
        let is_hit = self.is_hit(block_address);
        self.read_and_allocate(block_address, current_access_time);
        is_hit
    }

    /// Return the size of the set in bytes (the combined size of the blocks in the set).
    pub fn size_in_bytes(&self) -> u64 {
        self.blocks.len() as u64 * self.block_size
    }
}

/// A cache.
/// This contains the sets in the cache.
/// The cache is indexed by the index of the block.
#[derive(Clone, Debug)]
pub struct Cache {
    /// The sets in the cache.
    sets: Vec<Set>,
    /// The number of blocks in each set.
    /// This is the associativity of the cache.
    /// If the associativity is 1, then the cache is direct-mapped.
    /// If the associativity is the number of blocks in the cache, then
    /// the cache is fully associative.
    associativity: u64,
    /// The eviction policy of the cache.
    /// This is used to determine which block to evict when a block is
    /// inserted into a set that is full.
    evict_policy: EvictionPolicy,
}

impl Cache {
    /// Create a new cache.
    pub fn new(
        sets: usize,
        block_size: u64,
        associativity: u64,
        evict_policy: EvictionPolicy,
    ) -> Self {
        Self {
            sets: vec![Set::new(block_size, associativity, evict_policy); sets],
            associativity,
            evict_policy,
        }
    }

    /// Create a new fully associative cache.
    pub fn new_fully_associative(
        size_in_bytes: u64,
        block_size: u64,
        evict_policy: EvictionPolicy,
    ) -> Self {
        let number_of_sets = size_in_bytes / block_size;
        Self::new(
            number_of_sets as usize,
            block_size,
            number_of_sets,
            evict_policy,
        )
    }

    /// Create a new direct-mapped cache.
    pub fn new_direct_mapped(
        size_in_bytes: u64,
        block_size: u64,
        evict_policy: EvictionPolicy,
    ) -> Self {
        let number_of_sets = size_in_bytes / block_size;
        Self::new(number_of_sets as usize, block_size, 1, evict_policy)
    }

    /// Create a new set-associative cache.
    pub fn new_set_associative(
        associativity: u64,
        size_in_bytes: u64,
        block_size: u64,
        evict_policy: EvictionPolicy,
    ) -> Self {
        let number_of_sets = (size_in_bytes / block_size) / associativity;
        Self::new(
            number_of_sets as usize,
            block_size,
            associativity,
            evict_policy,
        )
    }

    /// Return the number of sets in the cache.
    pub fn len(&self) -> usize {
        self.sets.len()
    }

    /// Return the size of the cache in bytes (the combined size of the sets in the cache).
    /// This is the size of the cache in bytes.
    pub fn size_in_bytes(&self) -> u64 {
        self.sets.len() as u64 * self.sets[0].size_in_bytes()
    }

    /// Does this cache contain the block at the given address?
    pub fn is_hit(&self, address: BlockAddress) -> bool {
        self.get(address).is_some()
    }

    /// Get the associativity of the cache.
    pub fn get_associativity(&self) -> u64 {
        self.associativity
    }

    /// Get the eviction policy of the cache.
    pub fn get_eviction_policy(&self) -> EvictionPolicy {
        self.evict_policy
    }

    /// Get the block at the given address.
    /// Return None if the block is not in the cache.
    /// Return Some(block) if the block is in the cache.
    pub fn get(&self, address: BlockAddress) -> Option<&Block> {
        let set = &self.sets[address.index as usize];
        set.get_block_with_addr(address)
    }

    /// Write to the block at the given address.
    /// If the block isn't in the cache, then allocate a block.
    /// Return the evicted block, if any.
    pub fn write_and_allocate(
        &mut self,
        address: BlockAddress,
        current_access_time: u64,
    ) -> Option<Block> {
        let set = &mut self.sets[address.index as usize];
        set.write_and_allocate(address, current_access_time)
    }

    /// Read the block at the given address.
    /// If the block isn't in the cache, then allocate a block.
    /// Return the evicted block, if any.
    pub fn read_and_allocate(
        &mut self,
        address: BlockAddress,
        current_access_time: u64,
    ) -> Option<Block> {
        let set = &mut self.sets[address.index as usize];
        set.read_and_allocate(address, current_access_time)
    }

    /// Performs the write and allocate operation, and returns true if it was a write hit.
    /// Return the evicted block, if any.
    pub fn is_write_and_allocate_hit(
        &mut self,
        address: BlockAddress,
        current_access_time: u64,
    ) -> bool {
        let set = &mut self.sets[address.index as usize];
        set.is_write_and_allocate_hit(address, current_access_time)
    }

    /// Performs the read and allocate operation, and returns true if it was a read hit.
    /// Return the evicted block, if any.
    pub fn is_read_and_allocate_hit(
        &mut self,
        address: BlockAddress,
        current_access_time: u64,
    ) -> bool {
        let set = &mut self.sets[address.index as usize];
        set.is_read_and_allocate_hit(address, current_access_time)
    }

    /// Try to write to the block at the given address.
    /// If the block is not in the cache, then do nothing.
    /// Returns whether or not the write was a hit.
    pub fn try_write(&mut self, address: BlockAddress, current_access_time: u64) -> bool {
        let set = &mut self.sets[address.index as usize];
        set.try_write(address, current_access_time)
    }

    /// Try to read the block at the given address.
    /// If the block is not in the cache, then do nothing.
    /// Returns whether or not the read was a hit.
    pub fn try_read(&mut self, address: BlockAddress, current_access_time: u64) -> bool {
        let set = &mut self.sets[address.index as usize];
        set.try_read(address, current_access_time)
    }

    /// Return the number of blocks in the cache.
    /// This is the number of sets in the cache multiplied by the associativity.
    /// This is the number of blocks that can be stored in the cache.
    pub fn number_of_blocks(&self) -> u64 {
        self.sets.len() as u64 * self.associativity
    }
}

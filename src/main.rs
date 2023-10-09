use memory_hierarchy::*;

fn main() {
    env_logger::init();

    let config = SimulatorConfig::default();
    println!("{}", config);

    // Check for command line arguments
    let args: Vec<String> = std::env::args().collect();
    let trace = if args.len() > 1 {
        let filename = &args[1];
        eprintln!("Reading trace from file \"{}\"...", filename);
        Trace::from_file(filename)
    } else {
        eprintln!("Reading trace from stdin...");
        Trace::from_stdin()
    };
    eprintln!("Done reading trace");

    let mut page_table = PageTable::new_from_config(&config);
    // eprintln!("\nPage table:\n{:#?}", page_table);
    // eprintln!(
    //     "Index bits: {}, offset bits: {}",
    //     page_table.get_index_bits(),
    //     page_table.get_offset_bits()
    // );
    let mut dc = DataCache::new_from_config(&config);
    let mut tlb = TLBCache::new_from_config(&config);
    let mut l2 = L2Cache::new_from_config(&config);

    let mut page_tables_hits = 0;
    let mut page_tables_misses = 0;
    let mut cache_hits = 0;
    let mut cache_misses = 0;

    let mut sim = Simulator::from(config);

    eprintln!(
        r#"
Virtual  Virt.  Page TLB    TLB TLB  PT   Phys        DC  DC          L2  L2
Address  Page # Off  Tag    Ind Res. Res. Pg # DC Tag Ind Res. L2 Tag Ind Res.
-------- ------ ---- ------ --- ---- ---- ---- ------ --- ---- ------ --- ----"#
    );

    for (i, access) in trace.iter().enumerate() {
        eprintln!("{}", sim.simulate(*access));
        continue;

        eprintln!("Access: {access:#?}");
        let current_access_time = i as u64;
        let virtual_address = access.address();

        let tlb_address = BlockAddress::new_tlb_address(virtual_address, &config);
        eprintln!(
            "#{i} TLB {virtual_address:x} = {}",
            tlb.translate(tlb_address, current_access_time)
        );

        if let Some((physical_address, is_hit)) =
            page_table.translate(virtual_address, current_access_time)
        {
            if is_hit {
                page_tables_hits += 1;
                eprintln!(
                    "#{}: page {}, {:08x} translated to {:08x}, {}",
                    current_access_time,
                    access,
                    virtual_address,
                    physical_address,
                    if is_hit { "hit" } else { "miss" }
                );
            } else {
                page_tables_misses += 1;
                eprintln!(
                    "#{}: page {}, {:08x} translated to {:08x}, {}",
                    current_access_time,
                    access,
                    virtual_address,
                    physical_address,
                    if is_hit { "hit" } else { "miss" }
                );
            }
            eprintln!(
                "#{i} Virtual page number:  {:0num_bits$x}, offset: {:0off_bits$x}",
                page_table.get_virtual_page_number(virtual_address),
                page_table.get_offset(virtual_address),
                num_bits = page_table.get_index_bits() as usize / 16,
                off_bits = page_table.get_offset_bits() as usize / 16
            );
            eprintln!(
                "#{i} Physical page number: {:0num_bits$x}, offset: {:0off_bits$x}",
                page_table.get_physical_page_number(physical_address),
                page_table.get_offset(physical_address),
                num_bits = page_table.get_index_bits() as usize / 16,
                off_bits = page_table.get_offset_bits() as usize / 16
            );

            let address = BlockAddress::new_data_cache_address(physical_address, &config);
            // eprintln!("CACHE ADDRESS: {}", address);
            if access.is_write() {
                // if cache.is_write_and_allocate_hit(address, current_access_time) {
                //     eprintln!("Cache Write {}: {}, hit", current_access_time, access);
                //     cache_hits += 1;
                // } else {
                //     eprintln!("Cache Write {}: {}, miss", current_access_time, access);
                //     cache_misses += 1;
                // }
                if dc.write(address, current_access_time) {
                    eprintln!("#{i} DC Write Hit {access}");
                } else {
                    eprintln!("#{i} DC Write Miss {access}");
                    if l2.write(
                        BlockAddress::new_l2_cache_address(physical_address, &config),
                        current_access_time,
                    ) {
                        eprintln!("#{i} L2 Write Hit {access}");
                    } else {
                        eprintln!("#{i} L2 Write Miss {access}")
                    }
                }
            } else {
                if dc.read(address, current_access_time) {
                    eprintln!("#{i} Cache Read Hit {}: {}", current_access_time, access);
                } else {
                    eprintln!("#{i} Cache Read Miss {}: {}", current_access_time, access);
                    if l2.read(
                        BlockAddress::new_l2_cache_address(physical_address, &config),
                        current_access_time,
                    ) {
                        eprintln!("#{i} L2 Read Hit {access}");
                    } else {
                        eprintln!("#{i} L2 Read Miss {access}")
                    }
                }
                // cache.read_and_allocate(address, current_access_time);
                // if cache.is_read_and_allocate_hit(address, current_access_time) {
                //     eprintln!("Cache Read {}: {}, hit", current_access_time, access);
                //     cache_hits += 1;
                // } else {
                //     eprintln!("Cache Read {}: {}, miss", current_access_time, access);
                //     cache_misses += 1;
                // }
            }
            // println!("{:#?}", cache);
        } else {
            eprintln!("{}: {}, fault", current_access_time, access);
            break;
        }
    }
    /*
    let mut cache = Cache::new_direct_mapped(64, 16, EvictionPolicy::LRU);

    // let mut cache = Cache::new_direct_mapped(1024, 64, EvictionPolicy::FIFO);
    // let address = Address::new_data_cache_address(trace.get(0).unwrap().address(), &config);
    // println!("{}", address);
    let mut page_tables_hits = 0;
    let mut page_tables_misses = 0;
    let mut cache_hits = 0;
    let mut cache_misses = 0;

    for (i, access) in trace.iter().enumerate() {
        let current_access_time = i as u64;
        let virtual_address = access.address();
        if let Some((physical_address, is_hit)) =
            page_table.translate(virtual_address, current_access_time)
        {
            if is_hit {
                page_tables_hits += 1;
                eprintln!(
                    "#{}: page {}, {:08x} translated to {:08x}, {}",
                    current_access_time,
                    access,
                    virtual_address,
                    physical_address,
                    if is_hit { "hit" } else { "miss" }
                );
            } else {
                page_tables_misses += 1;
                eprintln!(
                    "#{}: page {}, {:08x} translated to {:08x}, {}",
                    current_access_time,
                    access,
                    virtual_address,
                    physical_address,
                    if is_hit { "hit" } else { "miss" }
                );
            }
            // eprintln!("Virtual page number:  {:0num_bits$x}, offset: {:0off_bits$x}", page_table.get_virtual_page_number(virtual_address), page_table.get_offset(virtual_address), num_bits=page_table.get_index_bits() as usize / 16, off_bits=page_table.get_offset_bits() as usize / 16);
            // eprintln!("Physical page number: {:0num_bits$x}, offset: {:0off_bits$x}", page_table.get_physical_page_number(physical_address), page_table.get_offset(physical_address), num_bits=page_table.get_index_bits() as usize / 16, off_bits=page_table.get_offset_bits() as usize / 16);

            let address = BlockAddress::new_data_cache_address(physical_address, &config);
            eprintln!("CACHE ADDRESS: {}", address);
            if access.is_write() {
                if cache.is_write_and_allocate_hit(address, current_access_time) {
                    eprintln!("Cache Write {}: {}, hit", current_access_time, access);
                    cache_hits += 1;
                } else {
                    eprintln!("Cache Write {}: {}, miss", current_access_time, access);
                    cache_misses += 1;
                }
            } else {
                // cache.read_and_allocate(address, current_access_time);
                if cache.is_read_and_allocate_hit(address, current_access_time) {
                    eprintln!("Cache Read {}: {}, hit", current_access_time, access);
                    cache_hits += 1;
                } else {
                    eprintln!("Cache Read {}: {}, miss", current_access_time, access);
                    cache_misses += 1;
                }
            }
            // println!("{:#?}", cache);
        } else {
            eprintln!("{}: {}, fault", current_access_time, access);
            break;
        }
    }
    eprintln!(
        "Page table hits: {}, misses: {}, hit rate: {:.2}%",
        page_tables_hits,
        page_tables_misses,
        page_tables_hits as f32 / (page_tables_hits + page_tables_misses) as f32 * 100.0
    );
    eprintln!(
        "Cache hits: {}, misses: {}, hit rate: {:.2}%",
        cache_hits,
        cache_misses,
        cache_hits as f32 / (cache_hits + cache_misses) as f32 * 100.0
    );
    */
}

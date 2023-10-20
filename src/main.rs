use memory_hierarchy::*;
use log::info;

fn main() {
    env_logger::init();

    let config = SimulatorConfig::default();

    // Check for command line arguments
    let args: Vec<String> = std::env::args().collect();
    let trace = if args.len() > 1 {
        let filename = &args[1];
        info!("Reading trace from file \"{}\"...", filename);
        Trace::from_file(filename)
    } else {
        info!("Reading trace from stdin...");
        Trace::from_stdin()
    };
    info!("Done reading trace");

    let mut sim = Simulator::from(config);
    println!("{}", sim.simulate(trace));
}

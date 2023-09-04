use memory_hierarchy::*;

fn main() {
    let config = Config::default();
    println!("Config:\n{}", config);

    let trace = Trace::from_stdin();
    println!("Trace: {:?}", trace);
}
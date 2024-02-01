extern crate spaceflake;

fn main() {
    let mut node = spaceflake::Node::new(1);
    let worker = node.new_worker();
    let sf = worker.generate();
    match sf {
        Ok(value) => {
            println!("Generated Spaceflake: {:#?}", value.decompose())
        }
        Err(error) => {
            println!("Error: {}", error)
        }
    }
}

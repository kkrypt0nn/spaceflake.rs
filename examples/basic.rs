extern crate spaceflake;

fn main() {
    let mut node = spaceflake::Node::new(1);
    let mut worker = node.new_worker();
    let sf = worker.generate();
    match sf {
        Ok(mut value) => {
            println!("Generated Spaceflake: {:#?}", value.decompose())
        }
        Err(error) => {
            println!("Error: {}", error)
        }
    }
}

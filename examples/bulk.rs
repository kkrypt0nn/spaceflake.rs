extern crate spaceflake;

fn main() {
    let settings = spaceflake::BulkGeneratorSettings::new(2_000_000);
    let mut spaceflakes = spaceflake::bulk_generate(settings);
    match spaceflakes {
        Ok(mut value) => {
            println!("Successfully generated {} Spaceflakes", value.len());
            println!("{:#?}", value[1337331].decompose());
        }
        Err(error) => {
            println!("Error: {}", error)
        }
    }

    let node_one = spaceflake::Node::new(1);
    spaceflakes = node_one.bulk_generate(1_000_000);
    match spaceflakes {
        Ok(mut value) => {
            println!("Successfully generated {} Spaceflakes", value.len());
            println!("{:#?}", value[7331].decompose());
        }
        Err(error) => {
            println!("Error: {}", error)
        }
    }

    let mut node_two = spaceflake::Node::new(2);
    let mut worker = node_two.new_worker();
    spaceflakes = worker.bulk_generate(500_000);
    match spaceflakes {
        Ok(mut value) => {
            println!("Successfully generated {} Spaceflakes", value.len());
            println!("{:#?}", value[1337].decompose());
        }
        Err(error) => {
            println!("Error: {}", error)
        }
    }
}

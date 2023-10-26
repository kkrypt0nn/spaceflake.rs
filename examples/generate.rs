extern crate spaceflake;

fn main() {
    let mut settings = spaceflake::GeneratorSettings {
        base_epoch: 1640995200000,
        ..Default::default()
    };
    let mut sf = spaceflake::generate(settings);
    match sf {
        Ok(mut value) => {
            println!("Generated Spaceflake: {:#?}", value.decompose())
        }
        Err(error) => {
            println!("Error: {}", error)
        }
    }

    settings.node_id = 5;
    settings.worker_id = 5;
    settings.sequence = 1337;
    sf = spaceflake::generate(settings);
    match sf {
        Ok(mut value) => {
            println!("Generated Spaceflake: {:#?}", value.decompose())
        }
        Err(error) => {
            println!("Error: {}", error)
        }
    }
}

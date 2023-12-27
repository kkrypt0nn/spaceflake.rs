extern crate spaceflake;

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::thread;
    use std::time::Duration;

    use spaceflake::Spaceflake;

    #[test]
    fn bulk_generation() {
        let mut spaceflakes: HashMap<String, Spaceflake> = HashMap::new();
        let settings = spaceflake::BulkGeneratorSettings::new(1_000_000);
        let bulk = spaceflake::bulk_generate(settings).expect("Failed generating the Spaceflakes");
        for spaceflake in bulk {
            if spaceflakes.contains_key(spaceflake.string_id().as_str()) {
                panic!("Spaceflake ID {} is a duplicate", spaceflake.id);
            }
            spaceflakes.insert(spaceflake.string_id(), spaceflake);
        }
    }

    #[test]
    fn bulk_generation_node() {
        let mut spaceflakes: HashMap<String, Spaceflake> = HashMap::new();
        let node = spaceflake::Node::new(1);
        let bulk = node
            .bulk_generate(1_000_000)
            .expect("Failed generating the Spaceflakes");
        for spaceflake in bulk {
            if spaceflakes.contains_key(spaceflake.string_id().as_str()) {
                panic!("Spaceflake ID {} is a duplicate", spaceflake.id);
            }
            spaceflakes.insert(spaceflake.string_id(), spaceflake);
        }
    }

    #[test]
    fn bulk_generation_worker() {
        let mut spaceflakes: HashMap<String, Spaceflake> = HashMap::new();
        let mut node = spaceflake::Node::new(1);
        let mut worker = node.new_worker();
        let bulk = worker
            .bulk_generate(1_000_000)
            .expect("Failed generating the Spaceflakes");
        for spaceflake in bulk {
            if spaceflakes.contains_key(spaceflake.string_id().as_str()) {
                panic!("Spaceflake ID {} is a duplicate", spaceflake.id);
            }
            spaceflakes.insert(spaceflake.string_id(), spaceflake);
        }
    }

    #[test]
    fn generate_at() {
        let mut node = spaceflake::Node::new(1);
        let mut worker = node.new_worker();
        let sf = worker.generate_at(1532180612064).unwrap();
        assert_eq!(sf.time(), 1532180612064);
    }

    #[test]
    fn generate_future() {
        let mut node = spaceflake::Node::new(1);
        let mut worker = node.new_worker();
        let sf = worker.generate_at(2662196938000).unwrap_err();
        assert_eq!(
            sf,
            "The current time must be greater than the time you want to generate the Spaceflake at"
        );
    }

    #[test]
    fn worker_unique() {
        let mut spaceflakes: HashMap<String, Spaceflake> = HashMap::new();
        let mut node = spaceflake::Node::new(1);
        let mut worker = node.new_worker();

        for _ in 0..1000 {
            let sf = worker.generate().expect("Failed generating the Spaceflake");
            if spaceflakes.contains_key(sf.string_id().as_str()) {
                panic!("Spaceflake ID {} is a duplicate", sf.id);
            }
            spaceflakes.insert(sf.string_id(), sf);
        }
    }

    #[test]
    fn same_timestamp_different_base_epoch() {
        let mut node = spaceflake::Node::new(1);
        let mut worker = node.new_worker();
        let sf1 = worker.generate().expect("Failed generating the Spaceflake");
        worker.base_epoch = 1672531200000; // Sunday, January 1, 2023 12:00:00 AM GMT
        let sf2 = worker.generate().expect("Failed generating the Spaceflake");
        // Thanks Windows
        if (sf1.time() > sf2.time() + 5) || (sf1.time() < sf2.time() - 5) {
            panic!(
                "Timestamps of the generated Spaceflakes are not the same, or at least not close"
            )
        }
    }

    #[test]
    fn generate_unique() {
        let mut spaceflakes: HashMap<String, Spaceflake> = HashMap::new();
        let settings = spaceflake::GeneratorSettings::default();

        for _ in 0..1000 {
            let sf = spaceflake::generate(settings).expect("Failed generating the Spaceflake");
            if spaceflakes.contains_key(sf.string_id().as_str()) {
                panic!("Spaceflake ID {} is a duplicate", sf.id);
            }
            spaceflakes.insert(sf.string_id(), sf);
            // When using random there is a chance that the sequence will be twice the same due to Rust's speed, hence using a worker is better. We wait a millisecond to make sure it's different.
            thread::sleep(Duration::from_millis(1));
        }
    }
}

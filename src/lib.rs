#![allow(clippy::needless_doctest_main)]

use rand::Rng;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::{fmt, thread};

/// The default epoch used **with milliseconds**, which is the 1st of January 2015 at 12:00:00 AM GMT.
pub const EPOCH: u64 = 1420070400000;

/// The maximum number that can be set with 5 bits.
const MAX_5_BITS: u64 = 31;

/// The maximum number that can be set with 12 bits.
const MAX_12_BITS: u64 = 4095;

/// The maximum amount of milliseconds for clock drift tolerance.
const CLOCK_DRIFT_TOLERANCE_MS: u64 = 10;

/// A Spaceflake is the internal name for a Snowflake ID.
///
/// Apart from being a crystal of snow, a snowflake is a form of unique identifier which is being used in distributed computing. It has specific parts and is 64 bits long in binary.
/// ![A Spaceflake structure](https://raw.githubusercontent.com/kkrypt0nn/spaceflake.rs/main/assets/spaceflake_structure.png)
#[derive(Debug)]
pub struct Spaceflake {
    /// The  base epoch that was used to generate the Spaceflake, default is [`EPOCH`].
    base_epoch: u64,
    /// The decimal representation of the Spaceflake.
    pub id: u64,
}

/// The display implementation of a Spaceflake. Will just return its ID.
impl fmt::Display for Spaceflake {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

/// The default implementation of a Spaceflake.
impl Spaceflake {
    fn new(id: u64, base_epoch: u64) -> Self {
        Spaceflake { base_epoch, id }
    }

    /// Returns the time at which the Spaceflake has been generated.
    pub fn time(&self) -> u64 {
        (self.id >> 22) + self.base_epoch
    }

    /// Returns the node ID of the Spaceflake.
    pub fn node_id(&self) -> u64 {
        (self.id & 0x3E0000) >> 17
    }

    /// Returns the worker ID of the Spaceflake.
    pub fn worker_id(&self) -> u64 {
        (self.id & 0x1F000) >> 12
    }

    /// Returns the sequence of the Spaceflake.
    pub fn sequence(&self) -> u64 {
        self.id & 0xFFF
    }

    /// Returns the ID of the Spaceflake as a string.
    pub fn string_id(&self) -> String {
        self.to_string()
    }

    /// Returns the ID in binary of the Spaceflake as a string.
    #[deprecated(since = "1.1.0", note = "Use `to_binary`")]
    pub fn binary_id(&self) -> String {
        self.to_binary()
    }

    /// Returns the ID in binary of the Spaceflake as a string.
    pub fn to_binary(&self) -> String {
        pad_left(decimal_binary(self.id), 64)
    }

    /// Returns a hashmap of key-values with each part of the Spaceflake.
    ///
    /// # Example
    ///
    /// ```rust
    /// fn main() {
    ///     let mut sf = spaceflake::generate(spaceflake::GeneratorSettings::default()).unwrap();
    ///     println!("{:#?}", sf.decompose());
    /// }
    /// ```
    ///
    /// Which will result in some output like
    /// ```json
    /// {
    ///     "id": 1165925685034747967,
    ///     "time": 1698048745164,
    ///     "sequence": 2111,
    ///     "node_id": 0,
    ///     "worker_id": 0,
    /// }
    /// ```
    pub fn decompose(&self) -> HashMap<String, u64> {
        HashMap::<String, u64>::from([
            ("id".to_string(), self.id),
            ("node_id".to_string(), self.node_id()),
            ("sequence".to_string(), self.sequence()),
            ("time".to_string(), self.time()),
            ("worker_id".to_string(), self.worker_id()),
        ])
    }

    /// Returns a hashmap of key-values with each part of the Spaceflake as binary.
    ///
    /// # Example
    ///
    /// ```rust
    /// fn main() {
    ///     let mut sf = spaceflake::generate(spaceflake::GeneratorSettings::default()).unwrap();
    ///     println!("{:#?}", sf.decompose_binary());
    /// }
    /// ```
    ///
    /// Which will result in some output like
    /// ```json
    /// {
    ///     "node_id": "00000",
    ///     "time": "11000101101011011100101111001111011001100",
    ///     "id": "0001000000101110001100110011101110110011000000000000100000111111",
    ///     "worker_id": "00000",
    ///     "sequence": "100000111111",
    /// }
    /// ```
    pub fn decompose_binary(&self) -> HashMap<String, String> {
        HashMap::<String, String>::from([
            ("id".to_string(), pad_left(decimal_binary(self.id), 64)),
            (
                "node_id".to_string(),
                pad_left(decimal_binary(self.node_id()), 5),
            ),
            (
                "sequence".to_string(),
                pad_left(decimal_binary(self.sequence()), 12),
            ),
            (
                "time".to_string(),
                pad_left(decimal_binary(self.time()), 41),
            ),
            (
                "worker_id".to_string(),
                pad_left(decimal_binary(self.worker_id()), 5),
            ),
        ])
    }
}

/// A node holds multiple [`Worker`] structures and has a, ideally, unique ID given.
#[derive(Debug)]
pub struct Node {
    /// The ID of the node, ideally it should be unique and not be used multiple times within an application.
    pub id: u64,
    /// The list of workers the node holds, which will then be responsible to generate the Spaceflakes.
    workers: Vec<Worker>,
}

/// The default implementation of a node.
impl Node {
    /// Create a new node for the given ID.
    pub fn new(id: u64) -> Self {
        if id > MAX_5_BITS {
            panic!("Node ID must be less than {}", MAX_5_BITS);
        }

        Node {
            id,
            workers: Vec::<Worker>::new(),
        }
    }

    /// Create a new worker and push it to the list of workers of the node to generate Spaceflakes.
    pub fn new_worker(&mut self) -> Worker {
        let worker = Worker::new((self.workers.len() + 1) as u64, self.id);
        self.workers.push(worker.clone());
        worker
    }

    /// Remove a worker given its ID from the list of workers.
    pub fn remove_worker(&mut self, id: u64) {
        if let Some(index) = self.workers.iter().position(|w| w.id == id) {
            self.workers.remove(index);
        }
    }

    /// Returns the list of workers the node is currently holding.
    pub fn get_workers(&self) -> Vec<Worker> {
        self.workers.clone()
    }

    /// Generate an amount of Spaceflakes on the node.
    ///
    /// The workers will automatically scale, so there is no need to add new workers to the node.
    pub fn bulk_generate(&self, amount: usize) -> Result<Vec<Spaceflake>, String> {
        let mut node = Node::new(self.id);
        let mut worker = node.new_worker();
        let mut spaceflakes = Vec::<Spaceflake>::with_capacity(amount);

        for i in 1..=amount {
            if i % ((MAX_12_BITS as usize * MAX_5_BITS as usize) + 1) == 0 {
                thread::sleep(Duration::from_millis(1));
                node.workers.clear();
                worker = node.new_worker();
            } else if i % MAX_12_BITS as usize == 0
                && i % (MAX_12_BITS as usize * MAX_5_BITS as usize) != 0
            {
                let new_worker = node.new_worker();
                worker = new_worker;
            }
            match generate_on_node_and_worker(node.id, worker.clone(), None) {
                Ok(spaceflake) => spaceflakes.push(spaceflake),
                Err(error) => return Err(error),
            };
        }

        Ok(spaceflakes)
    }
}

/// A worker is the a structure that is responsible to generate the Spaceflake.
#[derive(Debug, Clone)]
pub struct Worker {
    /// The ID of the worker, ideally it should be unique and not be used multiple times within an application.
    pub id: u64,
    /// The base epoch that will be used to generate the Spaceflakes, default is [`EPOCH`].
    pub base_epoch: u64,
    /// The node ID to which the worker belongs to.
    pub node_id: u64,
    /// The sequence of the worker, which is usually an incremented number but can be anything.
    ///
    /// If set to 0, it will be the incremented number.
    pub sequence: u64,
    /// The incremented number of the worker, used for the sequence.
    increment: Arc<Mutex<u64>>,
    /// The timestamp of the most recently generated Spaceflake, used to prevent clock drifting.
    last_timestamp: u64,
}

/// The default implementation of a worker.
impl Worker {
    fn new(id: u64, node_id: u64) -> Self {
        if id > MAX_12_BITS {
            panic!("Worker ID must be less than {}", MAX_12_BITS);
        }

        Worker {
            id,
            base_epoch: EPOCH,
            node_id,
            sequence: 0,
            increment: Arc::new(Mutex::new(0)),
            last_timestamp: 0,
        }
    }

    /// Generate a new Spaceflake on this worker.
    pub fn generate(&self) -> Result<Spaceflake, String> {
        generate_on_node_and_worker(self.node_id, self.clone(), None)
    }

    /// Generate a new Spaceflake on this worker at a specific time.
    pub fn generate_at(&self, at: u64) -> Result<Spaceflake, String> {
        generate_on_node_and_worker(self.node_id, self.clone(), Option::from(at))
    }

    /// Generate an amount of Spaceflakes on the worker.
    ///
    /// It will automatically sleep of a millisecond, only if needed, to prevent duplicated Spaceflakes to get generated.
    pub fn bulk_generate(&self, amount: usize) -> Result<Vec<Spaceflake>, String> {
        let mut spaceflakes = Vec::<Spaceflake>::new();

        for i in 1..=amount {
            if i % (MAX_12_BITS as usize + 1) == 0 {
                thread::sleep(Duration::from_millis(1));
            }
            match generate_on_node_and_worker(self.node_id, self.clone(), None) {
                Ok(spaceflake) => {
                    spaceflakes.push(spaceflake);
                }
                Err(error) => return Err(error),
            };
        }

        Ok(spaceflakes)
    }
}

/// Settings to bulk generate Spaceflakes easily.
#[derive(Debug)]
pub struct BulkGeneratorSettings {
    /// The amount of Spaceflakes to generate.
    amount: usize,
    /// The base epoch that will be used to generate the Spaceflakes, default is [`EPOCH`].
    pub base_epoch: u64,
}

/// The default implementation of a bulk generator settings.
impl BulkGeneratorSettings {
    pub fn new(amount: usize) -> Self {
        BulkGeneratorSettings {
            amount,
            base_epoch: EPOCH,
        }
    }
}

/// Generate an amount of Spaceflakes for the given settings.
///
/// Nodes and workers will be automatically scaled, and the function will also sleep of a millisecond if needed.
pub fn bulk_generate(settings: BulkGeneratorSettings) -> Result<Vec<Spaceflake>, String> {
    let mut node = Node::new(1);
    let mut worker = node.new_worker();
    worker.base_epoch = settings.base_epoch;
    let mut spaceflakes = Vec::<Spaceflake>::new();
    for i in 1..=settings.amount {
        if i % ((MAX_12_BITS * MAX_5_BITS * MAX_5_BITS) as usize) == 0 {
            thread::sleep(Duration::from_millis(1));
            let mut new_node = Node::new(1);
            let mut new_worker = new_node.new_worker();
            new_worker.base_epoch = settings.base_epoch;
            node = new_node;
            worker = new_worker;
        } else if node.workers.len() % MAX_5_BITS as usize == 0
            && i % ((MAX_5_BITS * MAX_12_BITS) as usize) == 0
        {
            let mut new_node = Node::new(1);
            let mut new_worker = new_node.new_worker();
            new_worker.base_epoch = settings.base_epoch;
            node = new_node;
            worker = new_worker;
        } else if i % MAX_12_BITS as usize == 0 {
            let mut new_worker = node.new_worker();
            new_worker.base_epoch = settings.base_epoch;
            worker = new_worker;
        }

        match generate_on_node_and_worker(node.id, worker.clone(), None) {
            Ok(spaceflake) => {
                spaceflakes.push(spaceflake);
            }
            Err(error) => return Err(error),
        };
    }

    Ok(spaceflakes)
}

/// Settings to generate Spaceflakes normally.
#[derive(Debug, Clone, Copy)]
pub struct GeneratorSettings {
    /// The base epoch that will be used to generate the Spaceflakes, default is [`EPOCH`].
    pub base_epoch: u64,
    /// The node ID for which the Spaceflake will be generated.
    pub node_id: u64,
    /// The worker ID for which the Spaceflake will be generated.
    pub worker_id: u64,
    /// The sequence of the generated Spaceflake.
    pub sequence: u64,
}

/// The default implementation of a generator settings.
impl GeneratorSettings {
    /// Create a new generator settings for the given node and worker IDs.
    pub fn new(node_id: u64, worker_id: u64) -> Self {
        if node_id > MAX_5_BITS {
            panic!("Node ID must be less than {}", MAX_5_BITS);
        }
        if worker_id > MAX_12_BITS {
            panic!("Worker ID must be less than {}", MAX_12_BITS);
        }

        GeneratorSettings {
            base_epoch: EPOCH,
            node_id,
            worker_id,
            sequence: 0,
        }
    }
}

/// The default implementation of a generator settings.
impl Default for GeneratorSettings {
    fn default() -> Self {
        Self::new(0, 0)
    }
}

/// Generate a Spaceflake for the given settings.
///
/// If the sequence is set to `0`, which is default, it it will get randomly generated.
pub fn generate(settings: GeneratorSettings) -> Result<Spaceflake, String> {
    let mut worker = Worker::new(settings.worker_id, settings.node_id);
    if settings.sequence == 0 {
        worker.sequence = rand::rng().random_range(1..=MAX_12_BITS);
    } else {
        worker.sequence = settings.sequence;
    }
    generate_on_node_and_worker(settings.node_id, worker, None)
}

/// Generate a Spaceflake for the given settings at a specific time.
///
/// If the sequence is set to `0`, which is default, it it will get randomly generated.
pub fn generate_at(settings: GeneratorSettings, at: u64) -> Result<Spaceflake, String> {
    let mut worker = Worker::new(settings.worker_id, settings.node_id);
    if settings.sequence == 0 {
        worker.sequence = rand::rng().random_range(1..=MAX_12_BITS);
    } else {
        worker.sequence = settings.sequence;
    }
    generate_on_node_and_worker(settings.node_id, worker, Option::from(at))
}

/// Parse the time of a Spaceflake ID.
pub fn parse_time(spaceflake_id: u64, base_epoch: u64) -> u64 {
    (spaceflake_id >> 22) + base_epoch
}

/// Parse the node ID of a Spaceflake ID.
pub fn parse_node_id(spaceflake_id: u64) -> u64 {
    (spaceflake_id & 0x3E0000) >> 17
}

/// Parse the worker ID of a Spaceflake ID.
pub fn parse_worker_id(spaceflake_id: u64) -> u64 {
    (spaceflake_id & 0x1F000) >> 12
}

/// Parse the sequence of a Spaceflake ID.
pub fn parse_sequence(spaceflake_id: u64) -> u64 {
    spaceflake_id & 0xFFF
}

/// Decompose a Spaceflake ID, and get a key-value hashmap with each part of a Spaceflake.
pub fn decompose(spaceflake_id: u64, base_epoch: u64) -> HashMap<String, u64> {
    Spaceflake::new(spaceflake_id, base_epoch).decompose()
}

/// Decompose a Spaceflake ID, and get a key-value hashmap with each part of a Spaceflake in binary.
pub fn decompose_binary(spaceflake_id: u64, base_epoch: u64) -> HashMap<String, String> {
    Spaceflake::new(spaceflake_id, base_epoch).decompose_binary()
}

/// Generates a Spaceflake for a given worker and node ID.
fn generate_on_node_and_worker(
    node_id: u64,
    mut worker: Worker,
    at: Option<u64>,
) -> Result<Spaceflake, String> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards?")
        .as_millis() as u64;

    let generate_at = at.unwrap_or(now);

    if node_id > MAX_5_BITS {
        return Err(format!("Node ID must be less than {}", MAX_5_BITS));
    }
    if worker.id > MAX_12_BITS {
        return Err(format!("Worker ID must be less than {}", MAX_12_BITS));
    }
    if worker.base_epoch > generate_at {
        return Err(String::from(
            "Base epoch must be less than the time you want to generate the Spaceflake at",
        ));
    }
    if worker.base_epoch > now {
        return Err(String::from(
            "Base epoch must be less than or equals to current epoch time",
        ));
    }
    if generate_at > now {
        return Err(String::from(
            "The current time must be greater than the time you want to generate the Spaceflake at",
        ));
    }

    let mut milliseconds = generate_at - worker.base_epoch;

    if milliseconds < worker.last_timestamp {
        let delta = worker.last_timestamp - milliseconds;
        if delta >= CLOCK_DRIFT_TOLERANCE_MS {
            return Err(format!("clock moved backwards by {}ms", delta));
        }
        thread::sleep(Duration::from_millis(delta + 1));

        let now_after_sleep = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards?")
            .as_millis() as u64;
        milliseconds = now_after_sleep - worker.base_epoch;
    }
    worker.last_timestamp = milliseconds;

    let mut increment = worker.increment.lock().unwrap();
    if *increment >= MAX_12_BITS {
        *increment = 0
    }
    *increment += 1;

    let base = pad_left(decimal_binary(milliseconds), 41);
    let node_id = pad_left(decimal_binary(node_id), 5);
    let worker_id = pad_left(decimal_binary(worker.id), 5);
    let mut actual_sequence = worker.sequence;
    if worker.sequence == 0 {
        actual_sequence = *increment
    }
    drop(increment);
    let sequence = pad_left(decimal_binary(actual_sequence), 12);
    let id = binary_decimal(format!("0{}{}{}{}", base, node_id, worker_id, sequence));

    Ok(Spaceflake::new(id, worker.base_epoch))
}

/// Convert a decimal number to a binary number.
fn decimal_binary(n: u64) -> String {
    format!("{:b}", n).to_string()
}

/// Convert a binary number to a decimal number.
fn binary_decimal(n: String) -> u64 {
    u64::from_str_radix(&n, 2).unwrap()
}

/// Add zeroes to the left of the string for the given width.
fn pad_left(string: String, width: usize) -> String {
    format!("{:0>1$}", string, width)
}

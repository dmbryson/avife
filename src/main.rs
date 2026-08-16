use anyhow::Result;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

mod config;
mod cpu;
mod environment;
mod genome;
mod organism;
mod phenotype;
mod population;
mod stats;

use crate::config::Config;
use crate::environment::Environment;
use crate::genome::{Genome, InstructionSet};
use crate::population::Population;
use crate::stats::Stats;

struct World {
    config: Config,
    environment: Environment,
    instruction_set: InstructionSet,
    rng: ChaCha8Rng,
    stats: Stats,
}

impl World {
    fn new(config: Config) -> Self {
        let seed = config.seed;
        World {
            config,
            environment: Environment::default(),
            instruction_set: InstructionSet::default_set(),
            rng: ChaCha8Rng::seed_from_u64(seed),
            stats: Stats::new(),
        }
    }
}

fn main() -> Result<()> {
    let config = Config {
        seed: 12,
        world_x: 100,
        world_y: 100,
        mutation_rate: 0.0075,
        ave_time_slice: 30,
        updates: 10000,
    };

    println!("random seed: {}", config.seed);

    let seed_org = Genome::load("/Users/dmbryson/Downloads/organism-heads-100.org")?;
    println!("seed organism: {}", seed_org.as_short_string());

    let mut world = World::new(config);

    let mut population = Population::new(&mut world, seed_org.clone());

    let mut update = 0;
    while update < world.config.updates {
        update += 1;

        population.process_update(&mut world);

        println!(
            "UD: {:<6}  Gen: {:<9.7}  Fit: {:<9.7}  Orgs: {:<6}  ",
            update,
            world.stats.generation.mean(),
            world.stats.fitness.mean(),
            population.get_num_organisms(),
        );
    }

    Ok(())
}

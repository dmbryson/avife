use rand::Rng;
use rand::distr::Distribution;
use rand::distr::weighted::WeightedIndex;
use rand::seq::IndexedRandom;

use crate::World;
use crate::genome::Genome;
use crate::organism::{Offspring, Organism, OrganismContext};
use crate::phenotype::Phenotype;

pub struct PopulationCell {
    #[allow(dead_code)]
    idx: usize,

    neighbors: Vec<usize>,
    inputs: Vec<i64>,

    organism: Option<Organism>,
    octx: Option<OrganismContext>,
}

impl PopulationCell {
    fn new<R: Rng>(idx: usize, rng: &mut R) -> Self {
        let inputs: Vec<i64> = vec![
            (0x0F << 56 | (rng.next_u64() >> 8)) as i64,
            (0x33 << 56 | (rng.next_u64() >> 8)) as i64,
            (0x55 << 56 | (rng.next_u64() >> 8)) as i64,
        ];
        PopulationCell {
            idx,
            neighbors: Vec::new(),
            inputs,
            organism: None,
            octx: None,
        }
    }

    fn has_organism(&self) -> bool {
        self.organism.is_some()
    }
}

struct Scheduler {
    dist: WeightedIndex<f64>,
}

impl Scheduler {
    fn new(cells: &[PopulationCell]) -> Self {
        let weights: Vec<f64> = cells.iter().map(|cell| {
            if let Some(organism) = &cell.organism {
                organism.merit
            } else {
                0.0
            }
        }).collect();
        Scheduler {
            dist: WeightedIndex::new(&weights).unwrap()
        }
    }

    fn update(&mut self, idx: usize, merit: f64) {
        self.dist.update_weights(&[(idx, &merit)]).unwrap();
    }

    fn next<R: Rng>(&self, rng: &mut R) -> usize {
        self.dist.sample(rng)
    }
}

pub struct Population {
    cells: Vec<PopulationCell>,
    world_x: usize,
    world_y: usize,
    num_organisms: usize,
    scheduler: Scheduler,
}

impl Population {
    pub fn new(world: &mut World, initial_genome: Genome) -> Self {
        let genome_length = initial_genome.len();
        let mut cells: Vec<PopulationCell> = (0..world.config.world_x * world.config.world_y)
            .map(|i| PopulationCell::new(i as usize, &mut world.rng))
            .collect();
        cells[0].organism = Some(Organism::new(initial_genome));
        let octx = OrganismContext::new(cells[0].inputs.clone(), -1, genome_length, world.environment.tasks.len(), world.environment.reactions.len());
        cells[0].octx = Some(octx);

        let scheduler = Scheduler::new(&cells);

        let mut population = Population {
            cells,
            world_x: world.config.world_x as usize,
            world_y: world.config.world_y as usize,
            num_organisms: 1,
            scheduler,
        };
        population.assign_torus_neighbors();
        population
    }

    pub fn process_update(&mut self, world: &mut World) {
        let ud_size = world.config.ave_time_slice as usize * self.num_organisms;
        for _ in 0..ud_size {
            // schedule organism
            let cell_idx = self.scheduler.next(&mut world.rng);

            // exec step, handle side effects
            let cell = &mut self.cells[cell_idx];
            let octx = cell.octx.as_mut().expect("cell should contain organism context");
            let organism = cell.organism.as_mut().expect("cell should contain organism");

            if let Some(offspring) = organism.process_step(world, octx) {
                self.activate_offspring(world, offspring, cell_idx);
            }
        }

        // process stats
        world.stats.reset_update();
        self.cells.iter().for_each(|cell| {
            if let Some(octx) = cell.octx.as_ref() {
                let fitness = octx.phenotype.fitness;
                let merit = octx.phenotype.merit;

                world.stats.fitness.add(fitness, 1.0);
                world.stats.merit.add(merit, 1.0);
                world.stats.generation.add(octx.phenotype.generation as f64, 1.0);
                world.stats.max_fitness = world.stats.max_fitness.max(fitness);
                world.stats.min_fitness = world.stats.min_fitness.min(fitness);
                world.stats.max_merit = world.stats.max_merit.max(merit);
                world.stats.min_merit = world.stats.min_merit.min(merit);
            } 
        })
    }

    pub fn get_num_organisms(&self) -> usize { self.num_organisms }

    fn activate_offspring(&mut self, world: &mut World, offspring: Offspring, cell_idx: usize) {
        let cell = &self.cells[cell_idx];

        let mut candidate_cells: Vec<usize> = Vec::new();

        // PREFER_EMPTY (default on)
        cell.neighbors.iter().for_each(|neighbor_idx| {
            if !self.cells[*neighbor_idx].has_organism() {
                candidate_cells.push(*neighbor_idx);
            }
        });

        // If no candidates, add all connections
        if candidate_cells.is_empty() {
            candidate_cells.extend(cell.neighbors.iter().copied());

            // ALLOW_PARENT (default on)
            candidate_cells.push(cell_idx);
        }

        let target_idx = if candidate_cells.is_empty() {
            // no candidates other than the parent, so parent it is
            cell_idx
        } else {
            candidate_cells.choose(&mut world.rng).copied()
                .expect("candidate_cells should be non-empty")
        };

        // determine parent merit
        let parent_phenotype = &mut self.cells[cell_idx].octx.as_mut().expect("cell has parent org").phenotype;
        parent_phenotype.divide_reset();
        let merit = parent_phenotype.merit;

        if target_idx != cell_idx { // parent is alive
            // adjust parent schedule
            self.scheduler.update(cell_idx, merit);
        }

        // create the offspring
        let mut offspring_org = Organism::new(offspring.genome);
        let offspring_phenotype = Phenotype::setup_offspring(parent_phenotype, offspring_org.initial_genome.len());

        // actually activate offspring
        let offspring_cell = &mut self.cells[target_idx];
        offspring_org.merit = merit;

        if !offspring_cell.has_organism() {
            self.num_organisms += 1;
        }
        let octx = OrganismContext::for_offspring(offspring_cell.inputs.clone(), offspring_phenotype);
        offspring_cell.organism = Some(offspring_org);
        offspring_cell.octx = Some(octx);
        self.scheduler.update(target_idx, merit);

        world.stats.tot_organisms += 1;
    }

    fn index(&self, x: isize, y: isize) -> usize {
        let dx = x.rem_euclid(self.world_x as isize) as usize;
        let dy = y.rem_euclid(self.world_y as isize) as usize;
        dx * self.world_x + dy
    }

    fn assign_torus_neighbors(&mut self) {
        const OFFSETS: [(isize, isize); 8] = [
            (-1, -1),
            (-1, 0),
            (-1, 1),
            (0, -1),
            (0, 1),
            (1, -1),
            (1, 0),
            (1, 1),
        ];

        for x in 0..self.world_x {
            for y in 0..self.world_y {
                let idx = self.index(x as isize, y as isize);

                let neighbor_indices: Vec<usize> = OFFSETS
                    .iter()
                    .map(|(dx, dy)| self.index(x as isize + dx, y as isize + dy))
                    .collect();

                self.cells[idx].neighbors = neighbor_indices;
            }
        }
    }
}

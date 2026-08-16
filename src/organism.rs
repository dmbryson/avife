use crate::World;
use crate::cpu::Cpu;
use crate::genome::Genome;
use crate::phenotype::Phenotype;

pub struct OrganismContext {
    inputs: Vec<i64>,
    input_pointer: usize,
    pub phenotype: Phenotype,
}

impl OrganismContext {
    pub fn new(inputs: Vec<i64>, parent_generation: i64, genome_length: usize, num_tasks: usize, num_reactions: usize) -> OrganismContext {
        OrganismContext{
            inputs,
            input_pointer: 0,
            phenotype: Phenotype::new(parent_generation, genome_length, num_tasks, num_reactions),
        }
    }
    
    pub fn for_offspring(inputs: Vec<i64>, phenotype: Phenotype) -> Self {
        OrganismContext { inputs, input_pointer: 0, phenotype }
    }

    pub fn process_output(&mut self, world: &mut World, output: i64) {
        let reaction_result = world.environment.process_output(output, &self.inputs, &self.phenotype.cur_reaction_count);

        for (idx, done) in reaction_result.tasks_done.iter().enumerate() {
            if *done { self.phenotype.cur_task_count[idx] += 1; }
        }

        for (idx, done) in reaction_result.reactions_done.iter().enumerate() {
            if *done { self.phenotype.cur_reaction_count[idx] += 1; }
        }

        self.phenotype.cur_bonus *= reaction_result.bonus_mult;
        self.phenotype.cur_bonus += reaction_result.bonus_add;
    }

    pub fn get_next_input(&mut self) -> i64 {
        self.input_pointer %= self.inputs.len();
        let input = self.inputs[self.input_pointer];
        self.input_pointer += 1;
        input
    }
}

pub struct Organism {
    pub initial_genome: Genome,
    pub cpu: Cpu,
    pub merit: f64,
}

impl Organism {
    pub fn new(initial_genome: Genome) -> Organism {
        let initial_merit = initial_genome.len() as f64;
        let cpu_genome = initial_genome.clone();
        Organism{
            initial_genome,
            cpu: Cpu::new(cpu_genome),
            merit: initial_merit,
        }
    }

    pub fn process_step(&mut self, world: &mut World, octx: &mut OrganismContext) -> Option<Offspring> {
        self.cpu.single_process(world, octx)
    }
}

pub struct Offspring {
    pub genome: Genome,
}

impl Offspring {
    pub fn new(genome: Genome) -> Offspring {
        Offspring{genome}
    }
}
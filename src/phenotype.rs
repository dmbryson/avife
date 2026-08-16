pub struct Phenotype {
    // 1. calculated values from last divide
    pub merit: f64,
    pub genome_length: usize,
    pub gestation_time: u64,
    pub gestation_start: u64,
    pub fitness: f64,

    // 2. in progress
    pub cur_bonus: f64,
    pub cur_task_count: Vec<u64>,
    pub cur_reaction_count: Vec<u64>,

    // 3. previous
    pub last_bonus: f64,
    pub last_task_count: Vec<u64>,
    pub last_reaction_count: Vec<u64>,

    // 4. organism's life stats
    pub num_divides: u64,
    pub generation: i64,
    pub time_used: u64,
}

impl Phenotype {
    pub fn new(parent_generation: i64, genome_length: usize, num_tasks: usize, num_reactions: usize) -> Self {
        Phenotype {
            merit: 0.0,
            genome_length: genome_length,
            gestation_time: 0,
            gestation_start: 0,
            fitness: 0.0,

            cur_bonus: 1.0,
            cur_task_count: vec![0; num_tasks],
            cur_reaction_count: vec![0; num_reactions],

            last_bonus: 0.0,
            last_task_count: vec![0; num_tasks],
            last_reaction_count: vec![0; num_reactions],

            num_divides: 0,
            generation: parent_generation + 1,
            time_used: 0,
        }
    }

    pub fn setup_offspring(parent: &Phenotype, genome_length: usize) -> Self {
        Phenotype {
            merit: parent.merit,
            genome_length,
            gestation_time: parent.gestation_time,
            gestation_start: 0,
            fitness: parent.fitness,

            cur_bonus: 1.0,
            cur_task_count: vec![0; parent.cur_task_count.len()],
            cur_reaction_count: vec![0; parent.cur_reaction_count.len()],

            last_bonus: parent.last_bonus,
            last_task_count: parent.last_task_count.clone(),
            last_reaction_count: parent.last_reaction_count.clone(),

            num_divides: 0,
            generation: parent.generation + 1,
            time_used: 0,
        }
    }

    pub fn divide_reset(&mut self) {
        let merit_base = self.calc_size_merit();
        self.merit = merit_base * self.cur_bonus;

        self.gestation_time = self.time_used - self.gestation_start;
        self.gestation_start = self.time_used;
        self.fitness = self.calc_fitness(merit_base);

        self.last_bonus = self.cur_bonus;
        self.last_task_count.copy_from_slice(&self.cur_task_count);  // alternative would be a std::mem::swap()

        self.cur_bonus = 1.0; // DEFAULT_BONUS
        self.cur_task_count.fill(0);

        self.num_divides += 1;
    }

    fn calc_fitness(&self, merit_base: f64) -> f64 {
        if self.gestation_time == 0 { return 0.0 }
        merit_base * self.cur_bonus / self.gestation_time as f64
    }

    pub fn calc_size_merit(&self) -> f64 {
        // only support FULL_SIZE right now
        self.genome_length as f64
    }
}
use average::WeightedMean;

pub struct Stats {
    // ---------  Per Update ---------
    pub merit: WeightedMean,
    pub generation: WeightedMean,
    pub fitness: WeightedMean,

    // ---------  Dominant Genotype  ---------
    pub max_fitness: f64,
    pub max_merit: f64,
    pub min_fitness: f64,
    pub min_merit: f64,

    // ---------  Lifetime  ---------
    pub tot_organisms: u64,
}

impl Stats {
    pub fn new() -> Self {
        Stats {
            merit: WeightedMean::new(),
            generation: WeightedMean::new(),
            fitness: WeightedMean::new(),

            max_fitness: f64::NEG_INFINITY,
            max_merit: f64::NEG_INFINITY,
            min_fitness: f64::INFINITY,
            min_merit: f64::INFINITY,

            tot_organisms: 0,
        }
    }

    pub fn reset_update(&mut self) {
        self.merit = WeightedMean::new();
        self.generation = WeightedMean::new();
        self.fitness = WeightedMean::new();

        self.max_fitness = f64::NEG_INFINITY;
        self.min_fitness = f64::INFINITY;
        self.max_merit = f64::NEG_INFINITY;
        self.min_merit = f64::INFINITY;
    }
}
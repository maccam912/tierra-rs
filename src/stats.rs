use std::collections::HashMap;

/// Statistics tracker for the simulation
#[derive(Debug, Clone)]
pub struct Statistics {
    /// Total number of instructions executed
    pub total_instructions: u64,

    /// Total number of organisms created
    pub total_organisms_created: u64,

    /// Total number of organisms died
    pub total_organisms_died: u64,

    /// Current population
    pub current_population: usize,

    /// Mutations applied
    pub total_mutations: u64,

    /// Failed replications
    pub failed_replications: u64,

    /// Successful replications
    pub successful_replications: u64,

    /// Size distribution (size -> count)
    pub size_distribution: HashMap<usize, usize>,

    /// Generation distribution
    pub generation_distribution: HashMap<usize, usize>,

    /// Memory usage
    pub memory_used: usize,
    pub memory_total: usize,

    /// History for graphing
    pub population_history: Vec<usize>,
    pub max_history_size: usize,
}

impl Statistics {
    pub fn new(memory_total: usize) -> Self {
        Self {
            total_instructions: 0,
            total_organisms_created: 0,
            total_organisms_died: 0,
            current_population: 0,
            total_mutations: 0,
            failed_replications: 0,
            successful_replications: 0,
            size_distribution: HashMap::new(),
            generation_distribution: HashMap::new(),
            memory_used: 0,
            memory_total,
            population_history: Vec::new(),
            max_history_size: 1000,
        }
    }

    /// Record an instruction execution
    pub fn record_instruction(&mut self) {
        self.total_instructions += 1;
    }

    /// Record a new organism
    pub fn record_birth(&mut self, size: usize, generation: usize) {
        self.total_organisms_created += 1;
        self.current_population += 1;
        *self.size_distribution.entry(size).or_insert(0) += 1;
        *self.generation_distribution.entry(generation).or_insert(0) += 1;
    }

    /// Record an organism death
    pub fn record_death(&mut self, size: usize, generation: usize) {
        self.total_organisms_died += 1;
        self.current_population = self.current_population.saturating_sub(1);

        if let Some(count) = self.size_distribution.get_mut(&size) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                self.size_distribution.remove(&size);
            }
        }

        if let Some(count) = self.generation_distribution.get_mut(&generation) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                self.generation_distribution.remove(&generation);
            }
        }
    }

    /// Record a mutation
    pub fn record_mutation(&mut self) {
        self.total_mutations += 1;
    }

    /// Record a replication attempt
    pub fn record_replication(&mut self, success: bool) {
        if success {
            self.successful_replications += 1;
        } else {
            self.failed_replications += 1;
        }
    }

    /// Update memory usage
    pub fn update_memory_usage(&mut self, used: usize) {
        self.memory_used = used;
    }

    /// Update population history for graphing
    pub fn update_history(&mut self, population: usize) {
        self.population_history.push(population);
        if self.population_history.len() > self.max_history_size {
            self.population_history.remove(0);
        }
    }

    /// Get the replication success rate
    pub fn replication_success_rate(&self) -> f64 {
        let total = self.successful_replications + self.failed_replications;
        if total > 0 {
            self.successful_replications as f64 / total as f64
        } else {
            0.0
        }
    }

    /// Get memory usage percentage
    pub fn memory_usage_percent(&self) -> f64 {
        if self.memory_total > 0 {
            (self.memory_used as f64 / self.memory_total as f64) * 100.0
        } else {
            0.0
        }
    }

    /// Get the most common organism size
    pub fn most_common_size(&self) -> Option<usize> {
        self.size_distribution
            .iter()
            .max_by_key(|(_, &count)| count)
            .map(|(&size, _)| size)
    }

    /// Get the highest generation
    pub fn highest_generation(&self) -> usize {
        self.generation_distribution.keys().max().copied().unwrap_or(0)
    }
}

impl Default for Statistics {
    fn default() -> Self {
        Self::new(65536)
    }
}

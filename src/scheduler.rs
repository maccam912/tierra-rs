use crate::organism::Organism;
use rand::Rng;

/// Scheduler for managing CPU time allocation to organisms
pub struct Scheduler {
    /// Current organism index being executed
    pub current_index: usize,

    /// Time slice size (instructions per organism per turn)
    pub time_slice: usize,
}

impl Scheduler {
    pub fn new(time_slice: usize) -> Self {
        Self {
            current_index: 0,
            time_slice,
        }
    }

    /// Select the next organism to execute
    pub fn select_next(&mut self, organisms: &mut [Organism], rng: &mut impl Rng) -> Option<usize> {
        if organisms.is_empty() {
            return None;
        }

        // Simple round-robin scheduling with random start position occasionally
        if rng.gen::<f64>() < 0.1 {
            // 10% chance to pick a random organism
            self.current_index = rng.gen_range(0..organisms.len());
        }

        // Find next alive organism
        let start_index = self.current_index;
        loop {
            if organisms[self.current_index].alive {
                let idx = self.current_index;
                organisms[idx].reset_energy(self.time_slice);

                // Move to next for next time
                self.current_index = (self.current_index + 1) % organisms.len();

                return Some(idx);
            }

            self.current_index = (self.current_index + 1) % organisms.len();

            // If we've checked all organisms, none are alive
            if self.current_index == start_index {
                return None;
            }
        }
    }

    /// Clean up dead organisms from the population
    pub fn reap_dead(organisms: &mut Vec<Organism>) -> usize {
        let initial_count = organisms.len();
        organisms.retain(|o| o.alive);
        initial_count - organisms.len()
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new(25) // Default time slice of 25 instructions
    }
}

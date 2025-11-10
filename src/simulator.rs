use crate::cpu::{CPU, ExecutionResult};
use crate::instruction::Instruction;
use crate::memory::Memory;
use crate::organism::Organism;
use crate::scheduler::Scheduler;
use crate::stats::Statistics;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

/// Configuration for the simulation
#[derive(Debug, Clone)]
pub struct SimulationConfig {
    pub memory_size: usize,
    pub mutation_rate: f64,
    pub max_population: usize,
    pub time_slice: usize,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            memory_size: 65536,
            mutation_rate: 0.001,
            max_population: 200,
            time_slice: 25,
        }
    }
}

/// Main simulation engine
pub struct Simulator {
    pub memory: Memory,
    pub organisms: Vec<Organism>,
    pub cpu: CPU,
    pub scheduler: Scheduler,
    pub stats: Statistics,
    pub config: SimulationConfig,
    pub rng: StdRng,
    next_organism_id: usize,
    pub running: bool,
}

impl Simulator {
    pub fn new(config: SimulationConfig) -> Self {
        let memory = Memory::new(config.memory_size);
        let stats = Statistics::new(config.memory_size);
        let scheduler = Scheduler::new(config.time_slice);

        Self {
            memory,
            organisms: Vec::new(),
            cpu: CPU::new(),
            scheduler,
            stats,
            config,
            rng: StdRng::from_entropy(),
            next_organism_id: 0,
            running: false,
        }
    }

    /// Initialize the simulation with the ancestor organism
    pub fn initialize_with_ancestor(&mut self) {
        // The ancestor is a simple self-replicating program
        let ancestor = create_ancestor();

        // Place it in memory
        let size = ancestor.len();
        if let Some(addr) = self.memory.allocate(size, &mut self.rng) {
            for (i, &inst) in ancestor.iter().enumerate() {
                self.memory.write(addr + i, inst);
            }

            // Create the organism
            let organism = Organism::new(self.next_organism_id, addr, size, 0, None);
            self.next_organism_id += 1;
            self.organisms.push(organism);
            self.stats.record_birth(size, 0);
        }
    }

    /// Step the simulation forward by one time slice
    pub fn step(&mut self) {
        if let Some(organism_idx) = self.find_next_organism() {
            // Execute time slice for this organism
            for _ in 0..self.config.time_slice {
                let organism = &mut self.organisms[organism_idx];

                if !organism.alive || !organism.consume_energy() {
                    break;
                }

                let result = self.cpu.execute_instruction(organism, &mut self.memory, &mut self.rng);
                self.stats.record_instruction();

                match result {
                    ExecutionResult::Continue => {}
                    ExecutionResult::Dead => {
                        let org = &self.organisms[organism_idx];
                        self.stats.record_death(org.size, org.generation);
                        self.memory.free(org.address, org.size);
                        break;
                    }
                    ExecutionResult::Malloc(size) => {
                        // Store the address in BX if successful
                        if let Some(addr) = self.memory.allocate(size, &mut self.rng) {
                            self.organisms[organism_idx].bx = addr;
                        } else {
                            self.organisms[organism_idx].errors += 1;
                        }
                        // Increment IP after malloc (instruction pointer was not advanced in execute_instruction)
                        self.organisms[organism_idx].increment_ip();
                    }
                    ExecutionResult::Divide => {
                        self.handle_divide(organism_idx);
                        break;
                    }
                }
            }
        }

        // Periodically clean up dead organisms
        if self.stats.total_instructions % 1000 == 0 {
            let reaped = Scheduler::reap_dead(&mut self.organisms);
            if reaped > 0 {
                // Update stats if needed
            }
        }

        // Update statistics
        if self.stats.total_instructions % 100 == 0 {
            self.update_stats();
        }
    }

    /// Handle organism division (reproduction)
    fn handle_divide(&mut self, parent_idx: usize) {
        let parent = &self.organisms[parent_idx];

        // Check if population limit reached
        if self.organisms.len() >= self.config.max_population {
            self.stats.record_replication(false);
            return;
        }

        // The offspring location is typically in BX register
        let offspring_addr = parent.bx;
        let offspring_size = parent.cx; // Size is often in CX

        // Validate offspring
        if offspring_size == 0 || offspring_size > self.config.memory_size / 10 {
            self.stats.record_replication(false);
            return;
        }

        // Copy genome from parent to offspring location with mutations
        let parent_addr = parent.address;
        let parent_size = parent.size;

        for i in 0..parent_size.min(offspring_size) {
            let inst = self.memory.read(parent_addr + i);
            self.memory.write(offspring_addr + i, inst);

            // Apply mutations
            if self.rng.gen::<f64>() < self.config.mutation_rate {
                self.memory.maybe_mutate(offspring_addr + i, 1.0, &mut self.rng);
                self.stats.record_mutation();
            }
        }

        // Create new organism
        let parent_id = parent.id;
        let parent_generation = parent.generation;

        let offspring = Organism::new(
            self.next_organism_id,
            offspring_addr,
            offspring_size,
            parent_generation + 1,
            Some(parent_id),
        );

        self.next_organism_id += 1;
        self.organisms.push(offspring);
        self.stats.record_birth(offspring_size, parent_generation + 1);
        self.stats.record_replication(true);

        // Mark memory as allocated
        self.memory.mark_allocated(offspring_addr, offspring_size, true);
    }

    /// Find the next organism to execute
    fn find_next_organism(&mut self) -> Option<usize> {
        if self.organisms.is_empty() {
            return None;
        }

        // Use scheduler to select next organism
        let current_idx = self.scheduler.current_index % self.organisms.len();

        // Find next alive organism
        for offset in 0..self.organisms.len() {
            let idx = (current_idx + offset) % self.organisms.len();
            if self.organisms[idx].alive {
                self.scheduler.current_index = (idx + 1) % self.organisms.len();
                // Reset energy for the new time slice
                self.organisms[idx].reset_energy(self.config.time_slice);
                return Some(idx);
            }
        }

        None
    }

    /// Update statistics
    fn update_stats(&mut self) {
        let alive_count = self.organisms.iter().filter(|o| o.alive).count();
        let memory_used = self.memory.size() - self.memory.count_free_cells();

        self.stats.update_memory_usage(memory_used);
        self.stats.update_history(alive_count);
    }

    /// Run multiple simulation steps
    pub fn run_steps(&mut self, steps: usize) {
        for _ in 0..steps {
            self.step();
        }
    }

    /// Reset the simulation
    pub fn reset(&mut self) {
        self.memory = Memory::new(self.config.memory_size);
        self.organisms.clear();
        self.stats = Statistics::new(self.config.memory_size);
        self.next_organism_id = 0;
        self.running = false;
    }
}

/// Create the ancestor organism - a simple self-replicating program
/// This is a minimal version that just allocates memory and divides
fn create_ancestor() -> Vec<Instruction> {
    use Instruction::*;

    // Create a simple ancestor that:
    // 1. Sets AX to organism size (for MallocA)
    // 2. Calls MallocA (stores offspring address in BX)
    // 3. Sets CX to organism size (for Divide)
    // 4. Calls Divide
    // handle_divide will copy the parent to offspring location

    // Fixed size approach:
    // 4 (start) + 64 (IncA) + 1 (MallocA) + 2 (PushA/PopC) + 12 (IncC) + 1 (Divide) + 4 (end) = 88
    // But we want AX to be 88 for the allocation
    // So we need 88 IncA instructions, making total = 4 + 88 + 1 + 2 + 12 + 1 + 4 = 112

    // Simpler: hardcode a reasonable size like 80
    let size = 80;
    let num_inc_a = size - 1 - 2 - 12 - 1 - 8; // Subtract other instructions
    let num_inc_c = 12; // Add this to CX to get from AX to actual size

    let mut instructions = vec![];

    // Start marker
    instructions.extend_from_slice(&[Nop1, Nop1, Nop1, Nop1]);

    // Set AX to size - num_inc_c - 8 (for the other instructions)
    for _ in 0..num_inc_a {
        instructions.push(IncA);
    }

    // MallocA - allocate AX bytes, address in BX
    instructions.push(MallocA);

    // Move size from AX to CX for Divide
    instructions.push(PushA);
    instructions.push(PopC);

    // Add the extra instructions to CX to match actual size
    for _ in 0..num_inc_c {
        instructions.push(IncC);
    }

    // Divide
    instructions.push(Divide);

    // End marker
    instructions.extend_from_slice(&[Nop0, Nop0, Nop0, Nop0]);

    instructions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulation_reaches_population_of_two() {
        let config = SimulationConfig {
            memory_size: 65536,
            mutation_rate: 0.0, // No mutations for testing
            max_population: 200,
            time_slice: 25,
        };

        let mut sim = Simulator::new(config);
        sim.initialize_with_ancestor();

        println!("Starting test...");
        println!("Ancestor size: {}", sim.organisms[0].size);
        println!("Ancestor address: {}", sim.organisms[0].address);

        // The simulation should reach a population of at least 2
        // We'll run for a maximum number of steps to avoid infinite loops
        let max_steps = 100000;
        let mut steps = 0;

        while steps < max_steps {
            sim.step();
            steps += 1;

            let alive_count = sim.organisms.iter().filter(|o| o.alive).count();

            if alive_count >= 2 {
                println!("✓ Reached population of {} after {} steps", alive_count, steps);
                println!("  Total instructions: {}", sim.stats.total_instructions);
                println!("  Successful replications: {}", sim.stats.successful_replications);
                println!("  Failed replications: {}", sim.stats.failed_replications);
                return;
            }

            // Print progress every 100 steps
            if steps % 100 == 0 {
                println!("Step {}: population = {}, instructions = {}",
                    steps, alive_count, sim.stats.total_instructions);
                if !sim.organisms.is_empty() {
                    let org = &sim.organisms[0];
                    println!("  Organism 0: IP={}, errors={}, cycles={}, energy={}",
                        org.ip, org.errors, org.cycles, org.energy);
                    println!("  Registers: AX={}, BX={}, CX={}, DX={}",
                        org.ax, org.bx, org.cx, org.dx);
                }
            }
        }

        panic!("Simulation did not reach population of 2 after {} steps. Current population: {}",
            max_steps, sim.organisms.iter().filter(|o| o.alive).count());
    }
}

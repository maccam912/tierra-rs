use crate::instruction::Instruction;

/// Represents a living organism in the Tierra simulation
#[derive(Debug, Clone)]
pub struct Organism {
    /// Unique identifier
    pub id: usize,

    /// Current instruction pointer
    pub ip: usize,

    /// Start address in memory
    pub address: usize,

    /// Size of the organism in memory
    pub size: usize,

    /// Registers
    pub ax: usize,  // General purpose / size register
    pub bx: usize,  // General purpose
    pub cx: usize,  // General purpose / counter
    pub dx: usize,  // General purpose / data

    /// Stack for procedure calls
    pub stack: Vec<usize>,

    /// Genome length (for statistics)
    pub genome_length: usize,

    /// Generation number
    pub generation: usize,

    /// Parent ID
    pub parent_id: Option<usize>,

    /// Number of CPU cycles executed
    pub cycles: usize,

    /// Number of errors encountered
    pub errors: usize,

    /// Is the organism alive?
    pub alive: bool,

    /// Energy/time slice counter
    pub energy: usize,
}

impl Organism {
    /// Create a new organism
    pub fn new(id: usize, address: usize, size: usize, generation: usize, parent_id: Option<usize>) -> Self {
        Self {
            id,
            ip: address,
            address,
            size,
            ax: 0,
            bx: 0,
            cx: 0,
            dx: 0,
            stack: Vec::new(),
            genome_length: size,
            generation,
            parent_id,
            cycles: 0,
            errors: 0,
            alive: true,
            energy: 100, // Initial energy allocation
        }
    }

    /// Increment the instruction pointer
    pub fn increment_ip(&mut self) {
        // Use saturating_sub to prevent overflow if IP is somehow less than address
        let offset = self.ip.saturating_sub(self.address);
        self.ip = self.address + ((offset + 1) % self.size);
        self.cycles += 1;
    }

    /// Set the instruction pointer to a new address (for jumps)
    pub fn set_ip(&mut self, addr: usize) {
        // Normalize the address to be within the organism's memory bounds
        if addr >= self.address && addr < self.address + self.size {
            self.ip = addr;
        } else {
            // If out of bounds, wrap it to the organism's memory space
            let offset = addr % self.size;
            self.ip = self.address + offset;
        }
    }

    /// Push a value onto the stack
    pub fn push(&mut self, value: usize) -> Result<(), String> {
        if self.stack.len() >= 10 {
            self.errors += 1;
            return Err("Stack overflow".to_string());
        }
        self.stack.push(value);
        Ok(())
    }

    /// Pop a value from the stack
    pub fn pop(&mut self) -> Result<usize, String> {
        self.stack.pop().ok_or_else(|| {
            self.errors += 1;
            "Stack underflow".to_string()
        })
    }

    /// Kill the organism
    pub fn kill(&mut self) {
        self.alive = false;
    }

    /// Reset energy for new time slice
    pub fn reset_energy(&mut self, amount: usize) {
        self.energy = amount;
    }

    /// Consume one unit of energy
    pub fn consume_energy(&mut self) -> bool {
        if self.energy > 0 {
            self.energy -= 1;
            true
        } else {
            false
        }
    }

    /// Check if organism is within its memory bounds
    pub fn is_address_valid(&self, addr: usize) -> bool {
        addr >= self.address && addr < self.address + self.size
    }

    /// Collect a template starting at current IP
    pub fn collect_template(&self, memory: &[Instruction], max_length: usize) -> Vec<Instruction> {
        let mut template = Vec::new();
        let mut pos = self.ip;

        for _ in 0..max_length {
            let inst = memory[pos % memory.len()];
            if inst.is_template() {
                template.push(inst);
                pos += 1;
            } else {
                break;
            }
        }

        template
    }
}

/// Statistics about organism populations
#[derive(Debug, Clone, Default)]
pub struct PopulationStats {
    pub total_organisms: usize,
    pub alive_organisms: usize,
    pub total_genomes: usize,
    pub average_size: f64,
    pub average_generation: f64,
    pub oldest_generation: usize,
}

impl PopulationStats {
    pub fn from_organisms(organisms: &[Organism]) -> Self {
        let alive: Vec<_> = organisms.iter().filter(|o| o.alive).collect();
        let alive_count = alive.len();

        let total_size: usize = alive.iter().map(|o| o.size).sum();
        let total_gen: usize = alive.iter().map(|o| o.generation).sum();
        let oldest_gen = alive.iter().map(|o| o.generation).max().unwrap_or(0);

        Self {
            total_organisms: organisms.len(),
            alive_organisms: alive_count,
            total_genomes: alive_count,
            average_size: if alive_count > 0 {
                total_size as f64 / alive_count as f64
            } else {
                0.0
            },
            average_generation: if alive_count > 0 {
                total_gen as f64 / alive_count as f64
            } else {
                0.0
            },
            oldest_generation: oldest_gen,
        }
    }
}

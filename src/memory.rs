use crate::instruction::Instruction;
use rand::Rng;

/// The memory "soup" where organisms live
pub struct Memory {
    data: Vec<Instruction>,
    size: usize,
    // Track which memory cells are allocated
    allocated: Vec<bool>,
}

impl Memory {
    /// Create a new memory soup of given size
    pub fn new(size: usize) -> Self {
        Self {
            data: vec![Instruction::Nop0; size],
            size,
            allocated: vec![false; size],
        }
    }

    /// Get the size of memory
    pub fn size(&self) -> usize {
        self.size
    }

    /// Read an instruction at an address (wraps around)
    pub fn read(&self, addr: usize) -> Instruction {
        self.data[addr % self.size]
    }

    /// Write an instruction at an address (wraps around)
    pub fn write(&mut self, addr: usize, inst: Instruction) {
        self.data[addr % self.size] = inst;
    }

    /// Normalize an address to be within bounds
    pub fn normalize_addr(&self, addr: usize) -> usize {
        addr % self.size
    }

    /// Find the next template match in forward direction
    /// Returns the address after the template
    pub fn find_template_forward(&self, start: usize, template: &[Instruction], max_search: usize) -> Option<usize> {
        if template.is_empty() {
            return None;
        }

        let complement: Vec<Instruction> = template
            .iter()
            .filter_map(|inst| inst.complement())
            .collect();

        if complement.is_empty() {
            return None;
        }

        for offset in 1..=max_search {
            let addr = self.normalize_addr(start + offset);
            let mut matches = true;

            for (i, &comp_inst) in complement.iter().enumerate() {
                if self.read(addr + i) != comp_inst {
                    matches = false;
                    break;
                }
            }

            if matches {
                return Some(self.normalize_addr(addr + complement.len()));
            }
        }

        None
    }

    /// Find the next template match in backward direction
    /// Returns the address after the template
    pub fn find_template_backward(&self, start: usize, template: &[Instruction], max_search: usize) -> Option<usize> {
        if template.is_empty() {
            return None;
        }

        let complement: Vec<Instruction> = template
            .iter()
            .filter_map(|inst| inst.complement())
            .collect();

        if complement.is_empty() {
            return None;
        }

        for offset in 1..=max_search {
            let addr = if start >= offset {
                start - offset
            } else {
                self.size - (offset - start)
            };

            let mut matches = true;

            for (i, &comp_inst) in complement.iter().enumerate() {
                if self.read(addr + i) != comp_inst {
                    matches = false;
                    break;
                }
            }

            if matches {
                return Some(self.normalize_addr(addr + complement.len()));
            }
        }

        None
    }

    /// Allocate a contiguous block of memory
    /// Returns the start address if successful
    pub fn allocate(&mut self, size: usize, rng: &mut impl Rng) -> Option<usize> {
        if size == 0 || size > self.size {
            return None;
        }

        // Try random positions
        for _ in 0..100 {
            let start = rng.gen_range(0..self.size);
            if self.is_range_free(start, size) {
                self.mark_allocated(start, size, true);
                return Some(start);
            }
        }

        // Linear search as fallback
        for start in 0..self.size {
            if self.is_range_free(start, size) {
                self.mark_allocated(start, size, true);
                return Some(start);
            }
        }

        None
    }

    /// Check if a memory range is free
    fn is_range_free(&self, start: usize, size: usize) -> bool {
        for i in 0..size {
            if self.allocated[self.normalize_addr(start + i)] {
                return false;
            }
        }
        true
    }

    /// Mark a range as allocated or free
    pub fn mark_allocated(&mut self, start: usize, size: usize, allocated: bool) {
        for i in 0..size {
            let addr = self.normalize_addr(start + i);
            self.allocated[addr] = allocated;
        }
    }

    /// Free a memory block
    pub fn free(&mut self, start: usize, size: usize) {
        self.mark_allocated(start, size, false);
    }

    /// Copy a block of memory from source to destination
    pub fn copy_block(&mut self, src: usize, dst: usize, size: usize) {
        let mut buffer = Vec::with_capacity(size);
        for i in 0..size {
            buffer.push(self.read(src + i));
        }
        for (i, &inst) in buffer.iter().enumerate() {
            self.write(dst + i, inst);
        }
    }

    /// Apply mutation to a memory cell with given probability
    pub fn maybe_mutate(&mut self, addr: usize, mutation_rate: f64, rng: &mut impl Rng) {
        if rng.gen::<f64>() < mutation_rate {
            let random_byte = rng.gen::<u8>() % 27; // We have 27 instructions
            let random_inst = Instruction::from_u8(random_byte);
            self.write(addr, random_inst);
        }
    }

    /// Get a slice view of memory for visualization
    pub fn get_slice(&self, start: usize, len: usize) -> Vec<Instruction> {
        (0..len)
            .map(|i| self.read(start + i))
            .collect()
    }

    /// Count free cells
    pub fn count_free_cells(&self) -> usize {
        self.allocated.iter().filter(|&&x| !x).count()
    }
}

use crate::instruction::Instruction;
use crate::memory::Memory;
use crate::organism::Organism;
use rand::Rng;

/// The CPU that executes organism instructions
pub struct CPU {
    /// Maximum search distance for template matching
    pub max_search: usize,
}

impl CPU {
    pub fn new() -> Self {
        Self {
            max_search: 200, // Maximum distance to search for templates
        }
    }

    /// Execute one instruction for the given organism
    /// Returns true if the organism should continue, false if it should be removed
    pub fn execute_instruction(
        &mut self,
        organism: &mut Organism,
        memory: &mut Memory,
        _rng: &mut impl Rng,
    ) -> ExecutionResult {
        if !organism.alive {
            return ExecutionResult::Dead;
        }

        let inst = memory.read(organism.ip);
        let mut advance_ip = true;

        match inst {
            Instruction::Nop0 | Instruction::Nop1 => {
                // No operation - just advance
            }

            Instruction::IfCZ => {
                // If CX is zero, execute next instruction, otherwise skip it
                if organism.cx != 0 {
                    organism.increment_ip();
                }
            }

            Instruction::JmpB => {
                // Jump backward to template complement
                organism.increment_ip();
                let template = self.read_template(organism, memory);

                if let Some(addr) = memory.find_template_backward(organism.ip, &template, self.max_search) {
                    organism.set_ip(addr);
                    advance_ip = false;
                } else {
                    organism.errors += 1;
                }
            }

            Instruction::JmpF => {
                // Jump forward to template complement
                organism.increment_ip();
                let template = self.read_template(organism, memory);

                if let Some(addr) = memory.find_template_forward(organism.ip, &template, self.max_search) {
                    organism.set_ip(addr);
                    advance_ip = false;
                } else {
                    organism.errors += 1;
                }
            }

            Instruction::Call => {
                // Call procedure at template
                organism.increment_ip();
                let template = self.read_template(organism, memory);

                if let Some(addr) = memory.find_template_forward(organism.ip, &template, self.max_search) {
                    if organism.push(organism.ip).is_ok() {
                        organism.set_ip(addr);
                        advance_ip = false;
                    }
                } else {
                    organism.errors += 1;
                }
            }

            Instruction::Ret => {
                // Return from procedure
                if let Ok(addr) = organism.pop() {
                    organism.set_ip(addr);
                    advance_ip = false;
                }
            }

            Instruction::MovDC => {
                // Move data from [CX] to DX
                let addr = organism.address + (organism.cx % organism.size);
                organism.dx = memory.read(addr).to_u8() as usize;
            }

            Instruction::MovCD => {
                // Move data from DX to [CX]
                let addr = organism.address + (organism.cx % organism.size);
                let inst = Instruction::from_u8((organism.dx % 27) as u8);

                // Only allow writing within organism's own memory
                if organism.is_address_valid(addr) {
                    memory.write(addr, inst);
                } else {
                    organism.errors += 1;
                }
            }

            Instruction::Adr => {
                // Get current address
                organism.ax = organism.ip;
            }

            Instruction::AdrB => {
                // Address of nearest template backward
                organism.increment_ip();
                let template = self.read_template(organism, memory);

                if let Some(addr) = memory.find_template_backward(organism.ip, &template, self.max_search) {
                    organism.ax = addr;
                } else {
                    organism.errors += 1;
                }
                advance_ip = false;
            }

            Instruction::AdrF => {
                // Address of nearest template forward
                organism.increment_ip();
                let template = self.read_template(organism, memory);

                if let Some(addr) = memory.find_template_forward(organism.ip, &template, self.max_search) {
                    organism.ax = addr;
                } else {
                    organism.errors += 1;
                }
                advance_ip = false;
            }

            Instruction::IncA => organism.ax = organism.ax.wrapping_add(1) % memory.size(),
            Instruction::IncB => organism.bx = organism.bx.wrapping_add(1) % memory.size(),
            Instruction::IncC => organism.cx = organism.cx.wrapping_add(1) % memory.size(),
            Instruction::DecC => organism.cx = if organism.cx > 0 { organism.cx - 1 } else { memory.size() - 1 },

            Instruction::MallocA => {
                // Allocate memory block of size AX
                return ExecutionResult::Malloc(organism.ax);
            }

            Instruction::Divide => {
                // Divide organism (create offspring)
                return ExecutionResult::Divide;
            }

            Instruction::PushA => { let _ = organism.push(organism.ax); }
            Instruction::PushB => { let _ = organism.push(organism.bx); }
            Instruction::PushC => { let _ = organism.push(organism.cx); }
            Instruction::PushD => { let _ = organism.push(organism.dx); }

            Instruction::PopA => { organism.ax = organism.pop().unwrap_or(0); }
            Instruction::PopB => { organism.bx = organism.pop().unwrap_or(0); }
            Instruction::PopC => { organism.cx = organism.pop().unwrap_or(0); }
            Instruction::PopD => { organism.dx = organism.pop().unwrap_or(0); }

            Instruction::Halt => {
                organism.kill();
                return ExecutionResult::Dead;
            }
        }

        if advance_ip {
            organism.increment_ip();
        }

        ExecutionResult::Continue
    }

    /// Read a template starting at the current IP
    fn read_template(&self, organism: &Organism, memory: &Memory) -> Vec<Instruction> {
        let mut template = Vec::new();
        let mut pos = organism.ip;

        for _ in 0..10 {  // Maximum template length
            let inst = memory.read(pos);
            if inst.is_template() {
                template.push(inst);
                pos = memory.normalize_addr(pos + 1);
            } else {
                break;
            }
        }

        template
    }
}

impl Default for CPU {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of executing an instruction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionResult {
    Continue,       // Continue execution
    Dead,          // Organism is dead
    Malloc(usize), // Request memory allocation
    Divide,        // Request division (create offspring)
}

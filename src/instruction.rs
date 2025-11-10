/// Tierra instruction set - simplified assembly-like operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Instruction {
    // Template matching and addressing
    Nop0 = 0,      // No operation, also used for templates
    Nop1 = 1,      // No operation, also used for templates

    // Flow control
    IfCZ = 2,      // If CX register is zero, execute next instruction
    JmpB = 3,      // Jump backward to template complement
    JmpF = 4,      // Jump forward to template complement
    Call = 5,      // Call procedure at template
    Ret = 6,       // Return from procedure

    // Data movement
    MovDC = 7,     // Move data from [CX] to DX
    MovCD = 8,     // Move data from DX to [CX]
    Adr = 9,       // Address: AX = CX (get current address)
    AdrB = 10,     // Address of nearest template backward
    AdrF = 11,     // Address of nearest template forward

    // Arithmetic
    IncA = 12,     // Increment AX
    IncB = 13,     // Increment BX
    IncC = 14,     // Increment CX
    DecC = 15,     // Decrement CX

    // Memory allocation
    MallocA = 16,  // Allocate memory block of size AX
    Divide = 17,   // Divide organism (offspring)

    // Stack operations
    PushA = 18,    // Push AX to stack
    PushB = 19,    // Push BX to stack
    PushC = 20,    // Push CX to stack
    PushD = 21,    // Push DX to stack
    PopA = 22,     // Pop from stack to AX
    PopB = 23,     // Pop from stack to BX
    PopC = 24,     // Pop from stack to CX
    PopD = 25,     // Pop from stack to DX

    // Control
    Halt = 26,     // Kill the organism
}

impl Instruction {
    /// Convert a u8 to an instruction, with invalid values becoming Nop0
    pub fn from_u8(byte: u8) -> Self {
        match byte {
            0 => Instruction::Nop0,
            1 => Instruction::Nop1,
            2 => Instruction::IfCZ,
            3 => Instruction::JmpB,
            4 => Instruction::JmpF,
            5 => Instruction::Call,
            6 => Instruction::Ret,
            7 => Instruction::MovDC,
            8 => Instruction::MovCD,
            9 => Instruction::Adr,
            10 => Instruction::AdrB,
            11 => Instruction::AdrF,
            12 => Instruction::IncA,
            13 => Instruction::IncB,
            14 => Instruction::IncC,
            15 => Instruction::DecC,
            16 => Instruction::MallocA,
            17 => Instruction::Divide,
            18 => Instruction::PushA,
            19 => Instruction::PushB,
            20 => Instruction::PushC,
            21 => Instruction::PushD,
            22 => Instruction::PopA,
            23 => Instruction::PopB,
            24 => Instruction::PopC,
            25 => Instruction::PopD,
            26 => Instruction::Halt,
            _ => Instruction::Nop0, // Invalid instructions become Nop0
        }
    }

    /// Convert instruction to u8
    pub fn to_u8(self) -> u8 {
        self as u8
    }

    /// Check if this instruction is a template marker (Nop0 or Nop1)
    pub fn is_template(&self) -> bool {
        matches!(self, Instruction::Nop0 | Instruction::Nop1)
    }

    /// Get the complement of a template instruction
    pub fn complement(&self) -> Option<Self> {
        match self {
            Instruction::Nop0 => Some(Instruction::Nop1),
            Instruction::Nop1 => Some(Instruction::Nop0),
            _ => None,
        }
    }
}

impl Default for Instruction {
    fn default() -> Self {
        Instruction::Nop0
    }
}

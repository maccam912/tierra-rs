# Tierra-rs

A Rust implementation of Tierra, the classic artificial life simulation created by Thomas S. Ray in the early 1990s. This simulation demonstrates evolution through natural selection in a population of self-replicating computer programs.

## Overview

Tierra simulates an ecosystem where digital organisms (self-replicating programs) live in a shared memory space called the "soup". These organisms compete for:
- CPU time (execution cycles)
- Memory space
- Successful replication

Through mutations during replication and natural selection, the organisms evolve over time, often producing smaller, more efficient variants and sometimes exhibiting parasitic behavior.

## Features

- **Virtual CPU**: Custom instruction set with 27 instructions designed for self-replication
- **Memory Management**: Dynamic memory allocation and deallocation ("the soup")
- **Scheduler**: Time-slicing execution model for fair CPU distribution
- **Mutations**: Configurable mutation rate during replication
- **Statistics Tracking**: Real-time population, generation, and evolution metrics
- **Interactive GUI**: Built with egui for visualization and control
  - Live memory visualization
  - Population graphs
  - Organism inspector
  - Runtime configuration controls

## Building and Running

### Prerequisites

- Rust 1.70 or later
- Cargo (comes with Rust)

### Build

```bash
cargo build --release
```

### Run

```bash
cargo run --release
```

## Usage

### Controls

- **▶ Run / ⏸ Pause**: Start or pause the simulation
- **⏭ Step**: Execute one simulation step
- **🔄 Reset**: Reset the simulation and reinitialize with the ancestor

### Configuration

You can adjust the following parameters in real-time:

- **Steps/frame**: How many simulation steps to execute per frame (1-1000)
- **Mutation Rate**: Probability of mutation per instruction during replication (0.0-0.1)
- **Max Population**: Maximum number of organisms allowed (10-500)
- **Time Slice**: Number of instructions each organism gets per turn (1-100)

### Understanding the Display

#### Left Panel - Statistics
- Population count and history
- Total instructions executed
- Birth/death counts
- Mutation statistics
- Replication success rate
- Memory usage
- Population graph over time

#### Center Panel - Memory Visualization
- Each pixel represents one instruction in memory
- Colors indicate instruction types:
  - Gray: Nop (template markers)
  - Light Blue: Jump/Call instructions
  - Light Green: Data movement
  - Light Red: Arithmetic operations
  - Light Yellow: Memory allocation
  - Orange: Stack operations
- Yellow borders indicate organism boundaries

#### Right Panel - Organisms
- List of all living organisms
- Shows: ID, size, generation, address, cycles, errors

## Architecture

### Core Components

1. **Instruction Set** (`instruction.rs`): 27-instruction ISA optimized for self-replication
2. **Memory** (`memory.rs`): Circular memory buffer with allocation tracking
3. **CPU** (`cpu.rs`): Virtual CPU that executes organism instructions
4. **Organism** (`organism.rs`): Represents a living digital creature with registers, stack, and state
5. **Scheduler** (`scheduler.rs`): Round-robin scheduler with time slicing
6. **Simulator** (`simulator.rs`): Main simulation engine coordinating all components
7. **Statistics** (`stats.rs`): Tracks population dynamics and evolution metrics
8. **UI** (`ui.rs`): egui-based graphical interface

### The Ancestor

The simulation starts with a single "ancestor" organism - a hand-crafted self-replicating program that:
1. Calculates its own size using template matching
2. Allocates memory for offspring
3. Copies itself to the new location
4. Applies random mutations
5. Divides to create the offspring

## Evolution Dynamics

Over time, you may observe:

- **Size optimization**: Organisms may evolve to be smaller and replicate faster
- **Parasitism**: Some organisms may evolve to exploit others' code
- **Population cycles**: Boom and bust cycles as resources become scarce
- **Generation diversity**: Multiple lineages with different strategies

## Technical Details

### Instruction Set

The ISA includes:
- Template matching for addressless jumps
- Stack operations for procedure calls
- Memory allocation/deallocation
- Conditional execution
- Self-inspection capabilities

### Replication Mechanism

1. Organism executes `MallocA` to allocate memory (size in AX register)
2. Copies its genome to the allocated space using memory operations
3. Mutations may occur with probability `mutation_rate`
4. Executes `Divide` to create the offspring as a new organism

### Memory Model

- Circular address space (wraps around)
- Allocation tracking prevents overwrites
- Template-based addressing allows position-independent code

## Credits

Based on the original Tierra by Thomas S. Ray (1990-1992).

This implementation is an educational recreation in Rust with modern visualization.

## License

MIT License - feel free to use, modify, and distribute.

## Contributing

Contributions are welcome! Some ideas for improvements:

- Additional instruction types
- More sophisticated scheduling algorithms
- Save/load functionality for interesting genomes
- Network-based distributed simulation
- Analysis tools for evolutionary trees
- Performance optimizations
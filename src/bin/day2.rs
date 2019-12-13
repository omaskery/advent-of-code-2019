use failure::{Fail, ResultExt};
use std::path::Path;
use advent_of_code_2019::load_file;
use crate::ComputerError::InstructionDecodeFailed;
use std::error::Error;

#[derive(Copy, Clone, Debug, Fail, Eq, PartialEq)]
enum ComputerError {
    #[fail(display = "unknown opcode")]
    UnknownOpcode,
    #[fail(display = "instruction decode failed")]
    InstructionDecodeFailed,
    #[fail(display = "attempted to interact with memory with an invalid address")]
    MemoryOperationOutOfBounds,
}

trait Memory {
    fn read_slot(&self, slot: usize) -> Result<i32, ComputerError>;
    fn write_slot(&mut self, slot: usize, value: i32) -> Result<(), ComputerError>;
    fn read_stream_from<'a>(&'a self, slot: usize) -> Result<Box<dyn Iterator<Item=i32> + 'a>, ComputerError>;
}

pub struct SimpleMemory {
    memory: Vec<i32>,
}

impl SimpleMemory {
    pub fn from_literal(memory: &[i32]) -> SimpleMemory {
        SimpleMemory {
            memory: Vec::from(memory),
        }
    }

    fn from_memory_file<P: AsRef<Path>>(path: P) -> Result<SimpleMemory, Box<dyn Error>> {
        let memory_file_contents = load_file(path)?;

        Ok(SimpleMemory {
            memory: memory_file_contents.split(",")
                .filter_map(|value| value.parse::<i32>().ok())
                .collect(),
        })
    }

    fn validate_slot(&self, slot: usize) -> Result<(), ComputerError> {
        if slot >= self.memory.len() {
            Err(ComputerError::MemoryOperationOutOfBounds)
        } else {
            Ok(())
        }
    }
}

impl Memory for SimpleMemory {
    fn read_slot(&self, slot: usize) -> Result<i32, ComputerError> {
        self.validate_slot(slot)?;
        Ok(self.memory[slot])
    }

    fn write_slot(&mut self, slot: usize, value: i32) -> Result<(), ComputerError> {
        self.validate_slot(slot)?;
        self.memory[slot] = value;
        println!("  write [{}] = {}", slot, value);
        Ok(())
    }

    fn read_stream_from<'a>(&'a self, slot: usize) -> Result<Box<dyn Iterator<Item=i32> + 'a>, ComputerError> {
        self.validate_slot(slot)?;
        Ok(Box::new(self.memory[slot..].iter().cloned()))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Opcode {
    Add,
    Multiply,
    Halt,
}

impl Opcode {
    fn decode(raw: i32) -> Result<Opcode, ComputerError> {
        match raw {
            1 => Ok(Opcode::Add),
            2 => Ok(Opcode::Multiply),
            99 => Ok(Opcode::Halt),
            _ => Err(ComputerError::UnknownOpcode),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Instruction {
    Add(usize, usize, usize),
    Multiply(usize, usize, usize),
    Halt,
}

struct ExecuteResult {
    advance_by: usize,
}

impl Instruction {
    fn decode<I: Iterator<Item=i32>>(instruction_stream: &mut I) -> Result<Instruction, ComputerError> {
        let opcode = match instruction_stream.next() {
            Some(raw_opcode) => Opcode::decode(raw_opcode)?,
            None => return Err(InstructionDecodeFailed),
        };

        let instruction = match opcode {
            Opcode::Add => {
                let values = Instruction::next_n_values(instruction_stream, 3)?;
                Instruction::Add(values[0] as usize, values[1] as usize, values[2] as usize)
            }
            Opcode::Multiply => {
                let values = Instruction::next_n_values(instruction_stream, 3)?;
                Instruction::Multiply(values[0] as usize, values[1] as usize, values[2] as usize)
            }
            Opcode::Halt => Instruction::Halt,
        };

        Ok(instruction)
    }

    fn next_n_values<I: Iterator<Item=i32>>(stream: &mut I, n: usize) -> Result<Vec<i32>, ComputerError> {
        let result = stream.take(n).collect::<Vec<_>>();
        if result.len() < n {
            Err(ComputerError::InstructionDecodeFailed)
        } else {
            Ok(result)
        }
    }
}

struct Computer<'a, M: Memory> {
    instruction_pointer: usize,
    cycle_count: usize,
    halted: bool,
    memory: &'a mut M,
}

impl<'a, M: Memory> Computer<'a, M> {
    fn new(memory: &'a mut M) -> Computer<'a, M> {
        Computer {
            instruction_pointer: 0,
            cycle_count: 0,
            halted: false,
            memory,
        }
    }

    fn step(&mut self) -> Result<(), ComputerError> {
        println!("[{}] executing at slot {}", self.cycle_count, self.instruction_pointer);
        let instruction = {
            let mut memory_at_instruction_pointer = self.memory.read_stream_from(self.instruction_pointer)?;
            Instruction::decode(&mut memory_at_instruction_pointer)?
        };
        println!("  decoded: {:?}", instruction);
        let result = self.execute(instruction)?;
        println!("  advancing by {}", result.advance_by);
        self.instruction_pointer += result.advance_by;
        self.cycle_count += 1;
        Ok(())
    }

    fn execute(&mut self, instruction: Instruction) -> Result<ExecuteResult, ComputerError> {
        let advance_by = match instruction {
            Instruction::Add(a, b, result) => {
                self.memory.write_slot(result, self.memory.read_slot(a)? + self.memory.read_slot(b)?)?;
                4
            }
            Instruction::Multiply(a, b, result) => {
                self.memory.write_slot(result, self.memory.read_slot(a)? * self.memory.read_slot(b)?)?;
                4
            }
            Instruction::Halt => {
                self.halted = true;
                0
            }
        };

        Ok(ExecuteResult {
            advance_by,
        })
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut memory = SimpleMemory::from_memory_file("input/day2.txt")?;

    memory.write_slot(1, 12).compat()?;
    memory.write_slot(2, 2).compat()?;

    let mut computer = Computer::new(&mut memory);

    while !computer.halted {
        computer.step().compat()?;
    }

    let result = memory.read_slot(0).compat()?;
    println!("result in slot 0: {}", result);

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{SimpleMemory, Memory, ComputerError, Instruction, Computer};

    #[test]
    fn can_read_memory() {
        let memory = SimpleMemory::from_literal(&[1, 2, 3]);
        assert_eq!(memory.read_slot(0), Ok(1));
        assert_eq!(memory.read_slot(1), Ok(2));
        assert_eq!(memory.read_slot(2), Ok(3));
        assert_eq!(memory.read_slot(3), Err(ComputerError::MemoryOperationOutOfBounds));
    }

    #[test]
    fn can_stream_memory() {
        let memory = SimpleMemory::from_literal(&[1, 2, 3]);
        let mut stream = memory.read_stream_from(1).unwrap();
        assert_eq!(stream.next(), Some(2));
        assert_eq!(stream.next(), Some(3));
        assert_eq!(stream.next(), None);
    }

    #[test]
    fn can_write_memory() {
        let mut memory = SimpleMemory::from_literal(&[4, 6, 8]);
        memory.write_slot(1, 12).unwrap();
        assert_eq!(memory.read_slot(1), Ok(12));
        assert_eq!(memory.write_slot(3, 10), Err(ComputerError::MemoryOperationOutOfBounds));
    }

    #[test]
    fn can_decode_add() {
        let memory = SimpleMemory::from_literal(&[1, 2, 3, 4]);
        let mut stream = memory.read_stream_from(0).unwrap();
        let result = Instruction::decode(&mut stream);
        assert_eq!(result, Ok(Instruction::Add(2, 3, 4)));
    }

    #[test]
    fn can_decode_multiply() {
        let memory = SimpleMemory::from_literal(&[2, 4, 8, 10]);
        let mut stream = memory.read_stream_from(0).unwrap();
        let result = Instruction::decode(&mut stream);
        assert_eq!(result, Ok(Instruction::Multiply(4, 8, 10)));
    }

    #[test]
    fn can_decode_halt() {
        let memory = SimpleMemory::from_literal(&[99]);
        let mut stream = memory.read_stream_from(0).unwrap();
        let result = Instruction::decode(&mut stream);
        assert_eq!(result, Ok(Instruction::Halt));
    }

    #[test]
    fn can_execute_halt() {
        let mut memory = SimpleMemory::from_literal(&[99]);
        let mut computer = Computer::new(&mut memory).unwrap();
        assert_eq!(computer.halted, false);
        computer.step().unwrap();
        assert_eq!(computer.instruction_pointer, 0);
        assert!(computer.halted);
    }

    #[test]
    fn can_execute_add() {
        let mut memory = SimpleMemory::from_literal(&[1, 4, 5, 6, 10, 22, 3]);
        let mut computer = Computer::new(&mut memory).unwrap();
        computer.step().unwrap();
        assert_eq!(computer.instruction_pointer, 4);
        assert_eq!(computer.memory.read_slot(6), Ok(32));
    }

    #[test]
    fn can_execute_multiply() {
        let mut memory = SimpleMemory::from_literal(&[2, 4, 5, 6, 10, 22, 3]);
        let mut computer = Computer::new(&mut memory).unwrap();
        computer.step().unwrap();
        assert_eq!(computer.instruction_pointer, 4);
        assert_eq!(computer.memory.read_slot(6), Ok(220));
    }

    #[test]
    fn can_execute_sequential_instructions() {
        let mut memory = SimpleMemory::from_literal(&[
            1, 9, 10, 11,
            2, 9, 11, 11,
            99,
            10, 22, 3
        ]);
        let mut computer = Computer::new(&mut memory).unwrap();
        computer.step().unwrap();
        assert_eq!(computer.instruction_pointer, 4);
        assert_eq!(computer.memory.read_slot(11), Ok(32));
        computer.step().unwrap();
        assert_eq!(computer.instruction_pointer, 8);
        assert_eq!(computer.memory.read_slot(11), Ok(320));
        computer.step().unwrap();
        assert_eq!(computer.instruction_pointer, 8);
        assert!(computer.halted);
    }

    #[test]
    fn verify_example_programs() {
        fn run_until_halted(computer: &Computer<SimpleMemory>) -> bool {
            computer.halted
        }
        verify_computer_program(
            &[1, 0, 0, 0, 99],
            &[2, 0, 0, 0, 99],
            run_until_halted,
        );
        verify_computer_program(
            &[2, 3, 0, 3, 99],
            &[2, 3, 0, 6, 99],
            run_until_halted,
        );
        verify_computer_program(
            &[2, 4, 4, 5, 99, 0],
            &[2, 4, 4, 5, 99, 9801],
            run_until_halted,
        );
        verify_computer_program(
            &[1, 1, 1, 4, 99, 5, 6, 0, 99],
            &[30, 1, 1, 4, 2, 5, 6, 0, 99],
            run_until_halted,
        );
    }

    fn verify_computer_program<F: Fn(&Computer<SimpleMemory>) -> bool>(initial_memory: &[i32], goal_memory: &[i32], stop_predicate: F) {
        let mut memory = SimpleMemory::from_literal(initial_memory);
        let mut computer = Computer::new(&mut memory).unwrap();
        while !stop_predicate(&computer) {
            computer.step().unwrap();
        }
        let final_memory = computer.memory.read_stream_from(0).unwrap().collect::<Vec<_>>();
        assert_eq!(goal_memory, final_memory.as_slice());
    }
}

use failure::Fail;
use std::path::Path;
use crate::load_file;
use std::error::Error;
use std::io::{stdin, stdout, Write};

type Address = usize;
type MemoryValue = i32;

#[derive(Copy, Clone, Debug, Fail, Eq, PartialEq)]
pub enum ComputerError {
    #[fail(display = "unknown opcode")]
    UnknownOpcode,
    #[fail(display = "instruction decode failed")]
    InstructionDecodeFailed,
    #[fail(display = "attempted to interact with memory with an invalid address")]
    MemoryOperationOutOfBounds,
    #[fail(display = "unknown parameter mode")]
    UnknownParameterMode,
    #[fail(display = "parameter specifying a destination address was flagged as immediate mode")]
    WriteParameterCannotBeImmediateMode,
    #[fail(display = "IO error while attempting to read from input")]
    FailedToGetInput,
}

pub trait Memory {
    fn read_slot(&self, slot: Address) -> Result<MemoryValue, ComputerError>;
    fn write_slot(&mut self, slot: Address, value: MemoryValue) -> Result<(), ComputerError>;
    fn read_stream_from<'a>(&'a self, slot: Address) -> Result<Box<dyn Iterator<Item=MemoryValue> + 'a>, ComputerError>;
}

pub struct SimpleMemory {
    memory: Vec<MemoryValue>,
}

impl SimpleMemory {
    pub fn from_literal(memory: &[MemoryValue]) -> SimpleMemory {
        SimpleMemory {
            memory: Vec::from(memory),
        }
    }

    pub fn from_memory_file<P: AsRef<Path>>(path: P) -> Result<SimpleMemory, Box<dyn Error>> {
        let memory_file_contents = load_file(path)?;

        Ok(SimpleMemory {
            memory: memory_file_contents.split(",")
                .filter_map(|value| value.parse::<MemoryValue>().ok())
                .collect(),
        })
    }

    pub fn validate_slot(&self, slot: Address) -> Result<(), ComputerError> {
        if slot >= self.memory.len() {
            Err(ComputerError::MemoryOperationOutOfBounds)
        } else {
            Ok(())
        }
    }
}

impl Memory for SimpleMemory {
    fn read_slot(&self, slot: Address) -> Result<MemoryValue, ComputerError> {
        self.validate_slot(slot)?;
        Ok(self.memory[slot])
    }

    fn write_slot(&mut self, slot: Address, value: MemoryValue) -> Result<(), ComputerError> {
        self.validate_slot(slot)?;
        self.memory[slot] = value;
        println!("  write [{}] = {}", slot, value);
        Ok(())
    }

    fn read_stream_from<'a>(&'a self, slot: Address) -> Result<Box<dyn Iterator<Item=MemoryValue> + 'a>, ComputerError> {
        self.validate_slot(slot)?;
        Ok(Box::new(self.memory[slot..].iter().cloned()))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ParameterMode {
    Position,
    Immediate,
}

impl ParameterMode {
    pub fn decode(raw: MemoryValue) -> Result<ParameterMode, ComputerError> {
        match raw % 10 {
            0 => Ok(ParameterMode::Position),
            1 => Ok(ParameterMode::Immediate),
            _ => Err(ComputerError::UnknownParameterMode),
        }
    }

    pub fn decode_all(mut raw: MemoryValue) -> Result<Vec<ParameterMode>, ComputerError> {
        let mut modes = Vec::new();
        while raw > 0 {
            modes.push(ParameterMode::decode(raw)?);
            raw /= 10;
        }
        Ok(modes)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Opcode {
    Add,
    Multiply,
    Input,
    Output,
    Halt,
}

impl Opcode {
    pub fn decode(raw: MemoryValue) -> Result<Opcode, ComputerError> {
        match raw % 100 {
            1 => Ok(Opcode::Add),
            2 => Ok(Opcode::Multiply),
            3 => Ok(Opcode::Input),
            4 => Ok(Opcode::Output),
            99 => Ok(Opcode::Halt),
            _ => Err(ComputerError::UnknownOpcode),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InstructionHeader {
    opcode: Opcode,
    parameter_modes: Vec<ParameterMode>,
}

impl InstructionHeader {
    pub fn decode(raw: MemoryValue) -> Result<InstructionHeader, ComputerError> {
        Ok(InstructionHeader {
            opcode: Opcode::decode(raw)?,
            parameter_modes: ParameterMode::decode_all(raw / 100)?,
        })
    }

    pub fn get_mode_of_parameter(&self, n: usize) -> ParameterMode {
        *self.parameter_modes.iter().nth(n).unwrap_or(&ParameterMode::Position)
    }

    pub fn wrap_parameter(&self, n: usize, value: MemoryValue) -> Parameter {
        match self.get_mode_of_parameter(n) {
            ParameterMode::Position => Parameter::Position(value as Address),
            ParameterMode::Immediate => Parameter::Immediate(value),
        }
    }

    pub fn wrap_parameters<I: Iterator<Item=MemoryValue>>(&self, stream: &mut I, n: usize) -> Result<Vec<Parameter>, ComputerError> {
        Ok(Instruction::next_n_values(stream, n)?
            .into_iter()
            .enumerate()
            .map(|(index, value)| self.wrap_parameter(index, value))
            .collect())
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Parameter {
    Position(Address),
    Immediate(MemoryValue),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Instruction {
    Add(Parameter, Parameter, Parameter),
    Multiply(Parameter, Parameter, Parameter),
    Input(Parameter),
    Output(Parameter),
    Halt,
}

pub struct ExecuteResult {
    advance_by: Address,
}

impl Instruction {
    pub fn decode<I: Iterator<Item=MemoryValue>>(instruction_stream: &mut I) -> Result<Instruction, ComputerError> {
        let header = match instruction_stream.next() {
            Some(raw_opcode) => InstructionHeader::decode(raw_opcode)?,
            None => return Err(ComputerError::InstructionDecodeFailed),
        };

        let instruction = match header {
            InstructionHeader { opcode: Opcode::Add, .. } => {
                let parameters = header.wrap_parameters(instruction_stream, 3)?;
                Instruction::Add(
                    parameters[0],
                    parameters[1],
                    parameters[2]
                )
            },
            InstructionHeader { opcode: Opcode::Multiply, .. } => {
                let parameters = header.wrap_parameters(instruction_stream, 3)?;
                Instruction::Multiply(
                    parameters[0],
                    parameters[1],
                    parameters[2]
                )
            },
            InstructionHeader { opcode: Opcode::Input, .. } => {
                let parameters = header.wrap_parameters(instruction_stream, 1)?;
                Instruction::Input(
                    parameters[0]
                )
            },
            InstructionHeader { opcode: Opcode::Output, .. } => {
                let parameters = header.wrap_parameters(instruction_stream, 1)?;
                Instruction::Output(
                    parameters[0]
                )
            },
            InstructionHeader { opcode: Opcode::Halt, .. } => Instruction::Halt,
        };

        Ok(instruction)
    }

    fn next_n_values<I: Iterator<Item=MemoryValue>>(stream: &mut I, n: usize) -> Result<Vec<MemoryValue>, ComputerError> {
        let result = stream.take(n).collect::<Vec<_>>();
        if result.len() < n {
            Err(ComputerError::InstructionDecodeFailed)
        } else {
            Ok(result)
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum RecordedIO {
    UserInput(MemoryValue),
    Output(MemoryValue),
}

pub struct Computer<'a, M: Memory> {
    instruction_pointer: Address,
    cycle_count: usize,
    pub halted: bool,
    memory: &'a mut M,
    pub io_record: Vec<RecordedIO>,
}

impl<'a, M: Memory> Computer<'a, M> {
    pub fn new(memory: &'a mut M) -> Computer<'a, M> {
        Computer {
            instruction_pointer: 0,
            cycle_count: 0,
            halted: false,
            memory,
            io_record: Vec::new(),
        }
    }

    pub fn step(&mut self) -> Result<(), ComputerError> {
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
                self.perform_write(result, self.perform_read(a)? + self.perform_read(b)?)?;
                4
            },
            Instruction::Multiply(a, b, result) => {
                self.perform_write(result, self.perform_read(a)? * self.perform_read(b)?)?;
                4
            },
            Instruction::Input(destination) => {
                let mut user_input = String::new();
                print!("  INPUT> ");
                stdout().flush().map_err(|_| ComputerError::FailedToGetInput)?;
                stdin().read_line(&mut user_input)
                    .map_err(|_| ComputerError::FailedToGetInput)?;
                user_input = user_input.trim().into();
                let value = user_input.parse::<MemoryValue>()
                    .map_err(|_| ComputerError::FailedToGetInput)?;
                self.io_record.push(RecordedIO::UserInput(value));
                self.perform_write(destination, value)?;
                2
            },
            Instruction::Output(source) => {
                let value = self.perform_read(source)?;
                self.io_record.push(RecordedIO::Output(value));
                println!("  OUTPUT VALUE: {}", value);
                2
            },
            Instruction::Halt => {
                self.halted = true;
                0
            },
        };

        Ok(ExecuteResult {
            advance_by,
        })
    }

    fn perform_read(&self, source: Parameter) -> Result<MemoryValue, ComputerError> {
        match source {
            Parameter::Position(address) => self.memory.read_slot(address),
            Parameter::Immediate(value) => Ok(value),
        }
    }

    fn perform_write(&mut self, destination: Parameter, value: MemoryValue) -> Result<(), ComputerError> {
        match destination {
            Parameter::Position(address) => self.memory.write_slot(address, value),
            Parameter::Immediate(_) => Err(ComputerError::WriteParameterCannotBeImmediateMode),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::intcode::{Computer, SimpleMemory, Memory, Instruction, ComputerError, Parameter};

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
        assert_eq!(result, Ok(Instruction::Add(
            Parameter::Position(2),
            Parameter::Position(3),
            Parameter::Position(4)
        )));
    }

    #[test]
    fn can_decode_multiply() {
        let memory = SimpleMemory::from_literal(&[2, 4, 8, 10]);
        let mut stream = memory.read_stream_from(0).unwrap();
        let result = Instruction::decode(&mut stream);
        assert_eq!(result, Ok(Instruction::Multiply(
            Parameter::Position(4),
            Parameter::Position(8),
            Parameter::Position(10)
        )));
    }

    #[test]
    fn validate_position_and_immediate_parameters() {
        let memory = SimpleMemory::from_literal(&[101, 2, 3, 4]);
        let mut stream = memory.read_stream_from(0).unwrap();
        let result = Instruction::decode(&mut stream);
        assert_eq!(result, Ok(Instruction::Add(
            Parameter::Immediate(2),
            Parameter::Position(3),
            Parameter::Position(4)
        )));

        let memory = SimpleMemory::from_literal(&[1001, 2, 3, 4]);
        let mut stream = memory.read_stream_from(0).unwrap();
        let result = Instruction::decode(&mut stream);
        assert_eq!(result, Ok(Instruction::Add(
            Parameter::Position(2),
            Parameter::Immediate(3),
            Parameter::Position(4)
        )));

        let memory = SimpleMemory::from_literal(&[10001, 2, 3, 4]);
        let mut stream = memory.read_stream_from(0).unwrap();
        let result = Instruction::decode(&mut stream);
        assert_eq!(result, Ok(Instruction::Add(
            Parameter::Position(2),
            Parameter::Position(3),
            Parameter::Immediate(4)
        )));

        let memory = SimpleMemory::from_literal(&[10002, 2, 3, 4]);
        let mut stream = memory.read_stream_from(0).unwrap();
        let result = Instruction::decode(&mut stream);
        assert_eq!(result, Ok(Instruction::Multiply(
            Parameter::Position(2),
            Parameter::Position(3),
            Parameter::Immediate(4)
        )));
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
        let mut computer = Computer::new(&mut memory);
        assert_eq!(computer.halted, false);
        computer.step().unwrap();
        assert_eq!(computer.instruction_pointer, 0);
        assert!(computer.halted);
    }

    #[test]
    fn can_execute_add() {
        let mut memory = SimpleMemory::from_literal(&[1, 4, 5, 6, 10, 22, 3]);
        let mut computer = Computer::new(&mut memory);
        computer.step().unwrap();
        assert_eq!(computer.instruction_pointer, 4);
        assert_eq!(computer.memory.read_slot(6), Ok(32));
    }

    #[test]
    fn can_execute_multiply() {
        let mut memory = SimpleMemory::from_literal(&[2, 4, 5, 6, 10, 22, 3]);
        let mut computer = Computer::new(&mut memory);
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
        let mut computer = Computer::new(&mut memory);
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
        let mut computer = Computer::new(&mut memory);
        while !stop_predicate(&computer) {
            computer.step().unwrap();
        }
        let final_memory = computer.memory.read_stream_from(0).unwrap().collect::<Vec<_>>();
        assert_eq!(goal_memory, final_memory.as_slice());
    }
}

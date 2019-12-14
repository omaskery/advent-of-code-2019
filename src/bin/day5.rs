use std::error::Error;
use advent_of_code_2019::intcode::{SimpleMemory, Computer};
use failure::ResultExt;

fn main() -> Result<(), Box<dyn Error>> {
    let mut memory = SimpleMemory::from_memory_file("input/day5.txt")?;

    let mut computer = Computer::new(&mut memory);

    while !computer.halted {
        computer.step().compat()?;
    }

    println!("IO record:");
    for event in computer.io_record.iter() {
        println!("  {:?}", event);
    }

    Ok(())
}

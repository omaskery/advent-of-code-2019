use std::error::Error;
use advent_of_code_2019::intcode::{SimpleMemory, Computer, Memory};
use failure::ResultExt;

fn main() -> Result<(), Box<dyn Error>> {
    let result = run_gravity_assist_with_parameters(12, 2)?;

    let goal_result = 19690720;
    let mut necessary_parameters = None;
    'outer: for noun in 0..=99 {
        for verb in 0..=99 {
            let current_result = run_gravity_assist_with_parameters(noun, verb)?;
            if current_result == goal_result {
                necessary_parameters = Some((noun, verb));
                break 'outer;
            }
        }
    }

    println!("initial result in slot 0: {}", result);
    if let Some((noun, verb)) = necessary_parameters {
        println!("necessary parameters for part b: noun={} verb={}", noun, verb);
        println!("  so 100 * noun + verb = {}", 100 * noun + verb);
    }

    Ok(())
}

fn run_gravity_assist_with_parameters(noun: i32, verb: i32) -> Result<i32, Box<dyn Error>> {
    let mut memory = SimpleMemory::from_memory_file("input/day2.txt")?;

    memory.write_slot(1, noun).compat()?;
    memory.write_slot(2, verb).compat()?;

    let mut computer = Computer::new(&mut memory);

    while !computer.halted {
        computer.step().compat()?;
    }

    Ok(memory.read_slot(0).compat()?)
}

use std::error::Error;
use advent_of_code_2019::{fuel_for_module_of_mass, load_lines_from};

fn main() -> Result<(), Box<dyn Error>> {
    let lines = load_lines_from("input/day1_a.txt")?;

    let module_weights = lines.iter()
        .filter_map(|line| line.parse::<u32>().ok());

    let fuel_required: u32 = module_weights.map(fuel_for_module_of_mass)
        .sum();

    println!("total fuel required: {}", fuel_required);

    Ok(())
}

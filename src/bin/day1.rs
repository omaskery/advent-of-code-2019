use std::error::Error;
use advent_of_code_2019::{fuel_for_mass, load_lines_from, fuel_for_module_of_mass};

fn main() -> Result<(), Box<dyn Error>> {
    let lines = load_lines_from("input/day1.txt")?;

    let module_weights = lines.iter()
        .filter_map(|line| line.parse::<i32>().ok())
        .collect::<Vec<_>>();

    let fuel_required_naive: i32 = module_weights.iter().cloned().map(fuel_for_mass)
        .sum();

    let fuel_required: i32 = module_weights.iter().cloned().map(fuel_for_module_of_mass)
        .sum();

    println!("total fuel required: {}", fuel_required_naive);
    println!("  (NOT accounting for extra weight of fuel)");
    println!("total fuel required: {}", fuel_required);
    println!("  (accounting for extra weight of fuel)");

    Ok(())
}

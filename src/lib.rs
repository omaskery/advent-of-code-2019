use std::io::{BufRead, Read};
use std::error::Error;
use std::path::Path;

pub fn fuel_for_mass(mass: i32) -> i32 {
    (mass / 3) - 2
}

pub fn fuel_for_module_of_mass(mass: i32) -> i32 {
    let mut total = fuel_for_mass(mass);
    let mut unaccounted_fuel = total;
    loop {
        let extra_fuel = fuel_for_mass(unaccounted_fuel);
        if extra_fuel <= 0 {
            break
        }

        total += extra_fuel;
        unaccounted_fuel = extra_fuel;
    }
    total
}

pub fn load_lines_from<P: AsRef<Path>>(path: P) -> Result<Vec<String>, Box<dyn Error>> {
    let file = std::fs::File::open(path)?;
    let buf_reader = std::io::BufReader::new(file);
    Ok(buf_reader.
        lines()
        .filter_map(|line| line.ok())
        .collect())
}

pub fn load_file<P: AsRef<Path>>(path: P) -> Result<String, Box<dyn Error>> {
    let file = std::fs::File::open(path)?;
    let mut buf_reader = std::io::BufReader::new(file);
    let mut result = String::new();
    buf_reader.read_to_string(&mut result)?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use crate::{fuel_for_mass, fuel_for_module_of_mass};

    #[test]
    fn can_convert_mass_to_fuel() {
        assert_eq!(fuel_for_mass(12), 2);
        assert_eq!(fuel_for_mass(14), 2);
        assert_eq!(fuel_for_mass(1969), 654);
        assert_eq!(fuel_for_mass(100756), 33583);
    }

    #[test]
    fn can_convert_module_mass_to_fuel() {
        assert_eq!(fuel_for_module_of_mass(12), 2);
        assert_eq!(fuel_for_module_of_mass(1969), 966);
        assert_eq!(fuel_for_module_of_mass(100756), 50346);
    }
}

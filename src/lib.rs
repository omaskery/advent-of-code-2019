use std::io::BufRead;
use std::error::Error;
use std::path::Path;

pub fn fuel_for_module_of_mass(mass: u32) -> u32 {
    (mass / 3) - 2
}

pub fn load_lines_from<P: AsRef<Path>>(path: P) -> Result<Vec<String>, Box<dyn Error>> {
    let file = std::fs::File::open(path)?;
    let buf_reader = std::io::BufReader::new(file);
    Ok(buf_reader.
        lines()
        .filter_map(|line| line.ok())
        .collect())
}

#[cfg(test)]
mod tests {
    use crate::fuel_for_module_of_mass;

    #[test]
    fn can_convert_mass_to_fuel() {
        assert_eq!(fuel_for_module_of_mass(12), 2);
        assert_eq!(fuel_for_module_of_mass(14), 2);
        assert_eq!(fuel_for_module_of_mass(1969), 654);
        assert_eq!(fuel_for_module_of_mass(100756), 33583);
    }
}

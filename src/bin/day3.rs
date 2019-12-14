use std::error::Error;
use advent_of_code_2019::load_lines_from;
use failure::{Fail, ResultExt};
use std::str::FromStr;
use std::collections::HashSet;
use std::cmp::Ordering;

#[derive(Copy, Clone, Debug, Eq, Fail, PartialEq)]
enum WiringError {
    #[fail(display = "invalid wiring instruction")]
    InvalidWiringInstruction,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum WiringDirection {
    Right,
    Up,
    Left,
    Down,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
struct WiringInstruction {
    direction: WiringDirection,
    distance: i32,
}

impl WiringInstruction {
    fn right(distance: i32) -> WiringInstruction {
        WiringInstruction {
            direction: WiringDirection::Right,
            distance,
        }
    }

    fn up(distance: i32) -> WiringInstruction {
        WiringInstruction {
            direction: WiringDirection::Up,
            distance,
        }
    }

    fn left(distance: i32) -> WiringInstruction {
        WiringInstruction {
            direction: WiringDirection::Left,
            distance,
        }
    }

    fn down(distance: i32) -> WiringInstruction {
        WiringInstruction {
            direction: WiringDirection::Down,
            distance,
        }
    }
}

impl FromStr for WiringInstruction {
    type Err = WiringError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() < 2 {
            return Err(WiringError::InvalidWiringInstruction);
        }
        let direction = match s.chars()
            .nth(0)
            .ok_or(WiringError::InvalidWiringInstruction)? {
            'u' | 'U' => WiringDirection::Up,
            'd' | 'D' => WiringDirection::Down,
            'l' | 'L' => WiringDirection::Left,
            'r' | 'R' => WiringDirection::Right,
            _ => return Err(WiringError::InvalidWiringInstruction),
        };
        let distance = s[1..].parse::<i32>()
            .map_err(|_| WiringError::InvalidWiringInstruction)?;
        Ok(WiringInstruction {
            direction,
            distance,
        })
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
struct Wire {
    instructions: Vec<WiringInstruction>,
}

impl Wire {
    fn new(instructions: Vec<WiringInstruction>) -> Wire {
        Wire {
            instructions,
        }
    }

    fn to_coords(&self) -> Vec<(i32, i32)> {
        let mut position = (0, 0);
        let mut coords = Vec::new();
        for instruction in self.instructions.iter() {
            for _ in 0..instruction.distance {
                let offset = match instruction.direction {
                    WiringDirection::Up => (0, 1),
                    WiringDirection::Down => (0, -1),
                    WiringDirection::Left => (-1, 0),
                    WiringDirection::Right => (1, 0),
                };
                let coordinate = (
                    position.0 + offset.0,
                    position.1 + offset.1,
                );
                position = coordinate;
                coords.push(coordinate);
            }
        }
        coords
    }
}

impl FromStr for Wire {
    type Err = WiringError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Wire{
            instructions: s.split(",")
                .map(|part| part.parse())
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

fn manhatten_distance_between(a: (i32, i32), b: (i32, i32)) -> i32 {
    (b.0 - a.0) + (b.1 - a.1)
}

fn main() -> Result<(), Box<dyn Error>> {
    let wires = load_lines_from("input/day3.txt")?.iter()
        .map(|line| line.parse::<Wire>())
        .collect::<Result<Vec<_>, _>>().compat()?;

    for wire in wires.iter() {
        println!("wire: {:?}", wire.instructions);
    }

    let coordinate_sets = wires.iter()
        .map(|wire| wire.to_coords().iter().cloned().collect::<HashSet<_>>())
        .collect::<Vec<_>>();

    if coordinate_sets.len() < 2 {
        panic!("expected at least 2 coordinate sets");
    }

    let wire_a_coordinate_set = &coordinate_sets[0];
    let wire_b_coordinate_set = &coordinate_sets[1];

    let mut cross_overs = Vec::new();
    for intersection in wire_a_coordinate_set.intersection(wire_b_coordinate_set) {
        if intersection.0 != 0 && intersection.1 != 0 {
            cross_overs.push(*intersection);
        }
    }

    if cross_overs.len() < 1 {
        panic!("expected at least 1 non-origin cross-over!");
    }

    let central_port = (0, 0);
    cross_overs.sort_by(|a, b| -> Ordering {
        manhatten_distance_between(central_port, *a).cmp(&manhatten_distance_between(central_port, *b))
    });

    let nearest_cross_over = cross_overs[0];
    let distance_to_nearest = manhatten_distance_between(central_port, nearest_cross_over);
    println!("nearest cross-over is {:?} with a distance of {}", nearest_cross_over, distance_to_nearest);

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{WiringInstruction, WiringError, Wire};

    #[test]
    fn can_parse_wiring_instructions() {
        assert_eq!("R10".parse(), Ok(WiringInstruction::right(10)));
        assert_eq!("U15".parse(), Ok(WiringInstruction::up(15)));
        assert_eq!("L4".parse(), Ok(WiringInstruction::left(4)));
        assert_eq!("D923".parse(), Ok(WiringInstruction::down(923)));

        assert_eq!("A".parse::<WiringInstruction>(), Err(WiringError::InvalidWiringInstruction));
        assert_eq!("".parse::<WiringInstruction>(), Err(WiringError::InvalidWiringInstruction));
        assert_eq!("A50".parse::<WiringInstruction>(), Err(WiringError::InvalidWiringInstruction));
        assert_eq!("D".parse::<WiringInstruction>(), Err(WiringError::InvalidWiringInstruction));
        assert_eq!("U".parse::<WiringInstruction>(), Err(WiringError::InvalidWiringInstruction));
        assert_eq!("L".parse::<WiringInstruction>(), Err(WiringError::InvalidWiringInstruction));
        assert_eq!("R".parse::<WiringInstruction>(), Err(WiringError::InvalidWiringInstruction));
        assert_eq!("RR".parse::<WiringInstruction>(), Err(WiringError::InvalidWiringInstruction));
    }

    #[test]
    fn can_parse_wire() {
        assert_eq!("R1".parse::<Wire>(), Ok(Wire::new(vec![
            WiringInstruction::right(1),
        ])));
        assert_eq!("R1,U5".parse::<Wire>(), Ok(Wire::new(vec![
            WiringInstruction::right(1),
            WiringInstruction::up(5),
        ])));
    }
}

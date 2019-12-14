use std::error::Error;
use advent_of_code_2019::load_lines_from;
use failure::{Fail, ResultExt};
use std::str::FromStr;
use std::collections::HashSet;
use std::cmp::Ordering;

#[derive(Copy, Clone, Debug, Eq, Fail, PartialEq)]
pub enum WiringError {
    #[fail(display = "invalid wiring instruction")]
    InvalidWiringInstruction,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum WiringDirection {
    Right,
    Up,
    Left,
    Down,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct WiringInstruction {
    direction: WiringDirection,
    distance: i32,
}

impl WiringInstruction {
    pub fn right(distance: i32) -> WiringInstruction {
        WiringInstruction {
            direction: WiringDirection::Right,
            distance,
        }
    }

    pub fn up(distance: i32) -> WiringInstruction {
        WiringInstruction {
            direction: WiringDirection::Up,
            distance,
        }
    }

    pub fn left(distance: i32) -> WiringInstruction {
        WiringInstruction {
            direction: WiringDirection::Left,
            distance,
        }
    }

    pub fn down(distance: i32) -> WiringInstruction {
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
pub struct Wire {
    instructions: Vec<WiringInstruction>,
}

impl Wire {
    pub fn new(instructions: Vec<WiringInstruction>) -> Wire {
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
    (b.0 - a.0).abs() + (b.1 - a.1).abs()
}

fn signal_time_to_point(wire: &[(i32, i32)], point: (i32, i32)) -> Option<usize> {
    wire.iter()
        .enumerate()
        .find(|(_, wire_point)| **wire_point == point)
        .map(|(index, _)| index + 1)
}

fn parse_wire_strings(lines: &[String]) -> Result<Vec<Wire>, WiringError> {
    lines.iter()
        .map(|line| line.parse::<Wire>())
        .collect::<Result<Vec<_>, _>>()
}

fn main() -> Result<(), Box<dyn Error>> {
    let wires = parse_wire_strings(&load_lines_from("input/day3.txt")?)
        .compat()?;

    for wire in wires.iter() {
        println!("wire: {:?}", wire.instructions);
    }

    let AnalysisOutcome { cross_over_signal_times, nearest_cross_over, distance_to_nearest, nearest_cross_over_in_signal_time } = analyse_circuit(wires);

    println!("nearest cross-over is {:?} with a distance of {}", nearest_cross_over, distance_to_nearest);

    println!("nearest cross-over (accounting for signal time) is {:?} with a signal time of {}", nearest_cross_over_in_signal_time.0, nearest_cross_over_in_signal_time.1);

    println!("there are cross overs at:");
    for (pos, time) in cross_over_signal_times.iter() {
        println!("  - {:?} (signal time: {})", *pos, time);
    }

    Ok(())
}

struct AnalysisOutcome {
    cross_over_signal_times: Vec<((i32, i32), usize)>,
    nearest_cross_over: (i32, i32),
    distance_to_nearest: i32,
    nearest_cross_over_in_signal_time: ((i32, i32), usize),
}

fn analyse_circuit(wires: Vec<Wire>) -> AnalysisOutcome {
    let coordinates = wires.iter()
        .map(|wire| wire.to_coords())
        .collect::<Vec<_>>();
    let coordinate_sets = coordinates.iter()
        .map(|c| c.iter().cloned().collect::<HashSet<_>>())
        .collect::<Vec<_>>();
    if coordinate_sets.len() < 2 {
        panic!("expected at least 2 coordinate sets");
    }
    let wire_a_coordinates = &coordinates[0];
    let wire_b_coordinates = &coordinates[1];
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
    let mut cross_over_signal_times = cross_overs.iter()
        .map(|cross_over| Some((*cross_over, signal_time_to_point(wire_a_coordinates, *cross_over)? + signal_time_to_point(wire_b_coordinates, *cross_over)?)))
        .collect::<Option<Vec<_>>>()
        .expect("failed to find cross overs in one of the wires");
    cross_over_signal_times.sort_by(|a, b| a.1.cmp(&b.1));
    let nearest_cross_over = cross_overs[0];
    let distance_to_nearest = manhatten_distance_between(central_port, nearest_cross_over);
    let nearest_cross_over_in_signal_time = cross_over_signal_times[0];
    AnalysisOutcome {
        cross_over_signal_times,
        nearest_cross_over,
        distance_to_nearest,
        nearest_cross_over_in_signal_time
    }
}

#[cfg(test)]
mod tests {
    use crate::{WiringInstruction, WiringError, Wire, analyse_circuit, parse_wire_strings, signal_time_to_point};

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

    #[test]
    fn can_calculate_closest() {
        assert_eq!(analyse_circuit(parse_wire_strings(&[
            "R8,U5,L5,D3".into(),
            "U7,R6,D4,L4".into(),
        ]).expect("failed to parse test case")).distance_to_nearest, 6);
        assert_eq!(analyse_circuit(parse_wire_strings(&[
            "R75,D30,R83,U83,L12,D49,R71,U7,L72".into(),
            "U62,R66,U55,R34,D71,R55,D58,R83".into(),
        ]).expect("failed to parse test case")).distance_to_nearest, 159);
        assert_eq!(analyse_circuit(parse_wire_strings(&[
            "R98,U47,R26,D63,R33,U87,L62,D20,R33,U53,R51".into(),
            "U98,R91,D20,R16,D67,R40,U7,R15,U6,R7".into(),
        ]).expect("failed to parse test case")).distance_to_nearest, 135);
    }

    #[test]
    fn can_calculate_steps_for_signal_path() {
        assert_eq!(
            signal_time_to_point(
                &"R8,U5,L5,D3".parse::<Wire>().expect("failed to parse test case").to_coords(),
                (3, 3)
            ),
            Some(20)
        );
        assert_eq!(
            signal_time_to_point(
                &"R8,U5,L5,D3".parse::<Wire>().expect("failed to parse test case").to_coords(),
                (6, 5)
            ),
            Some(15)
        );
    }
}

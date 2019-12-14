use std::error::Error;

const SIX_DIGIT_RANGE: (i32, i32) = (111111, 999999);

fn is_six_digit_number(number: i32) -> bool {
    within_range(SIX_DIGIT_RANGE, number)
}

fn within_range(range: (i32, i32), number: i32) -> bool {
    let (low, high) = range;
    number >= low && number <= high
}

fn group_adjacent_identical_digits(number: i32) -> Vec<(char, usize)> {
    let as_string = format!("{}", number);
    let mut groups: Vec<(char, usize)> = Vec::new();
    for digit in as_string.chars() {
        match groups.last_mut() {
            Some((value, count)) if *value == digit => *count += 1,
            _ => groups.push((digit, 1usize)),
        }
    }
    groups
}

fn has_only_two_adjacent_digits(number: i32) -> bool {
    let groups = group_adjacent_identical_digits(number);
    groups.into_iter().any(|(_, count)| count == 2)
}

fn digits_never_decrease(number: i32) -> bool {
    let as_string = format!("{}", number);
    let sorted= {
        let mut copy = as_string.chars().collect::<Vec<_>>();
        copy.sort();
        copy.into_iter().collect::<String>()
    };
    as_string == sorted
}

fn validate_value(range: (i32, i32), number: i32) -> bool {
    is_six_digit_number(number) &&
        within_range(range, number) &&
        has_only_two_adjacent_digits(number) &&
        digits_never_decrease(number)
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_range = (372304, 847060);

    let (low, high) = input_range;
    let mut possibilities = 0;
    for value in low..=high {
        if validate_value(input_range, value) {
            possibilities += 1;
        }
    }

    println!("between {} and {} there are {} possibilities", low, high, possibilities);

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{has_only_two_adjacent_digits, digits_never_decrease, validate_value, SIX_DIGIT_RANGE};

    #[test]
    fn has_only_two_adjacent_digits_works() {
        assert_eq!(has_only_two_adjacent_digits(123456), false);
        assert_eq!(has_only_two_adjacent_digits(122456), true);
        assert_eq!(has_only_two_adjacent_digits(112233), true);
        assert_eq!(has_only_two_adjacent_digits(112223), true);
        assert_eq!(has_only_two_adjacent_digits(111222), false);
    }

    #[test]
    fn digits_never_decrease_works() {
        assert_eq!(digits_never_decrease(123456), true);
        assert_eq!(digits_never_decrease(123455), true);
        assert_eq!(digits_never_decrease(123454), false);
        assert_eq!(digits_never_decrease(222222), true);
        assert_eq!(digits_never_decrease(212222), false);
        assert_eq!(digits_never_decrease(221222), false);
        assert_eq!(digits_never_decrease(222122), false);
        assert_eq!(digits_never_decrease(222212), false);
        assert_eq!(digits_never_decrease(222221), false);
    }

    #[test]
    fn matches_test_cases() {
        assert_eq!(validate_value(SIX_DIGIT_RANGE, 112233), true);
        assert_eq!(validate_value(SIX_DIGIT_RANGE, 123444), false);
    }
}

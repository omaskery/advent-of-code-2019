use std::error::Error;

fn is_six_digit_number(number: i32) -> bool {
    within_range((111111, 999999), number)
}

fn within_range(range: (i32, i32), number: i32) -> bool {
    let (low, high) = range;
    number >= low && number <= high
}

fn has_two_adjacent_digits(number: i32) -> bool {
    format!("{}", number).chars()
        .collect::<Vec<_>>()
        .windows(2)
        .any(|slice| slice[0] == slice[1])
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
        has_two_adjacent_digits(number) &&
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
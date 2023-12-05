use std::collections::HashMap;

use lazy_static::lazy_static;

advent_of_code::solution!(1);

pub fn part_one(input: &str) -> Option<u32> {
    let result = input
        .lines()
        .map(|line| {
            let first = line.chars().find(char::is_ascii_digit).unwrap();
            let last = line.chars().rev().find(char::is_ascii_digit).unwrap();
            (first as u8 - b'0', last as u8 - b'0')
        })
        .map(|(first, last)| first as u32 * 10 + last as u32)
        .sum();

    Some(result)
}

// Direction of string traversal
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
enum Direction {
    F,
    B,
}

static DIGITS: &str = "123456789";

lazy_static! {

    /// This is a map of digit strings in the read direction to the digit value
    static ref NUMBERS: HashMap<&'static str, u32> = [
        "one", "two", "three", "four", "five", "six", "seven", "eight", "nine",
    ]
    .into_iter()
    .enumerate()
    .flat_map(|(i, name)| {
        let v = i as u32 + 1;
        [
            (&DIGITS[i..=i], v),
            (name, v),
        ]
    })
    .collect::<HashMap<_, _>>();

    /// This is a map of characters that can be the first of a digit to the
    /// lengths of the strings that need to be checked for that digit
    static ref CHAR_SEARCH_LENGTHS: HashMap<(Direction, char), Vec<usize>> =
        NUMBERS
            .keys()
            .fold(HashMap::new(), |mut result, chars| {
                use Direction as D;

                let first_c = chars.chars().next().unwrap();
                let last_c = chars.chars().last().unwrap();
                let char_len = chars.len();

                [(D::F, first_c), (D::B, last_c)]
                    .into_iter().for_each(|key| {
                        result
                            .entry(key)
                            .and_modify(|v| v.push(char_len))
                            .or_insert_with(|| vec![char_len]);
                    });

                result
            });
}

// Given a stream of tokens, find the first one that matches a number in the
// map defined above
fn find_first_digit_token<'a, I>(mut tokens: I) -> u32
where
    I: Iterator<Item = &'a str> + 'a,
{
    tokens
        .find_map(|token| NUMBERS.get(token))
        .cloned()
        .unwrap()
}

// Get the tokens from the original string in the given direction and using the
// characters in the order given. We are assuming here that the order of the
// tokens matches the direction requested :-/
fn get_directional_tokens<'a, I>(
    ordered_chars: I,
    original: &'a str,
    direction: Direction,
) -> impl Iterator<Item = &'a str> + 'a
where
    I: Iterator<Item = (usize, char)> + 'a,
{
    use Direction as D;
    ordered_chars
        .filter_map(move |(i, c)| {
            CHAR_SEARCH_LENGTHS
                .get(&(direction, c))
                .map(|lens| (i, lens))
        })
        .flat_map(move |(i, lens)| {
            lens.iter()
                .cloned()
                .filter(move |length| match direction {
                    // This is a bounds check to make sure we don't overflow the
                    // next step in the sequence
                    D::F => i + *length <= original.len(),
                    D::B => i + 1 >= *length,
                })
                .map(move |length| match direction {
                    D::F => &original[i..(i + length)],
                    D::B => &original[(i + 1 - length)..=i],
                })
        })
}

pub fn part_two(input: &str) -> Option<u32> {
    use Direction as D;
    let result = input
        .lines()
        .map(|line| {
            let length = line.len();
            let first = find_first_digit_token(get_directional_tokens(
                line.chars().enumerate(),
                line,
                D::F,
            ));
            let last = find_first_digit_token(get_directional_tokens(
                line.chars()
                    .rev()
                    .enumerate()
                    .map(|(i, c)| (length - i - 1, c)),
                line,
                D::B,
            ));
            (first, last)
        })
        .map(|(first, last)| first * 10 + last)
        .sum();

    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let result =
            part_one(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(142));
    }

    #[test]
    fn test_part_two() {
        let result = part_two(
            &advent_of_code::template::read_extra_example_file(DAY, 1),
        );
        assert_eq!(result, Some(281));
    }
}

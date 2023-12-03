use std::ops::Range;

use itertools::Itertools;
use lazy_static::lazy_static;

advent_of_code::solution!(3);

lazy_static! {
    static ref DIRECTIONS: Vec<(isize, isize)> = (-1isize..=1)
        .flat_map(|x| (-1isize..=1).map(move |y| (x, y)))
        .collect::<Vec<_>>();
}

// We will use this struct as an iterator over the numbers contained in a string
struct NumberExtractor<'a> {
    input: &'a str,
    cursor: usize,
}

impl<'a> NumberExtractor<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, cursor: 0 }
    }
}

// Over complicated method for extracting numbers and positions from a string
impl<'a> Iterator for NumberExtractor<'a> {
    type Item = (Range<usize>, u32);

    fn next(&mut self) -> Option<Self::Item> {
        // This is the only state we need to maintain in this operation. We will
        // walk the digits in the string and combine consecutive digits into a
        // single number
        let mut prev_i_opt: Option<usize> = None;

        let (start_opt, length, num) = self
            .input
            .char_indices()
            .skip(self.cursor)
            .filter(|(_, c)| c.is_ascii_digit())
            .take_while(|(i, _)| {
                let result = if let Some(prev_i) = prev_i_opt {
                    *i == prev_i + 1
                } else {
                    true
                };

                prev_i_opt = Some(*i);

                result
            })
            .fold((None, 0, 0), |(start_opt, length, res), (i, c)| {
                self.cursor = i + 1;
                (
                    start_opt.or(Some(i)),
                    length + 1,
                    res * 10 + (c as u8 - b'0') as u32,
                )
            });

        start_opt.map(|start| (start..(start + length), num))
    }
}

// Get the symbols and positions surrounding a range within an input string
fn get_surrounding_symbols<'a>(
    input: &'a str,
    stride: usize,
    range: &Range<usize>,
) -> impl Iterator<Item = (usize, char)> + 'a {
    range
        .clone()
        .flat_map(move |i| {
            DIRECTIONS.iter().copied().map(move |(x, y)| {
                i.checked_add_signed(x + y * stride as isize)
            })
        })
        .flatten()
        .unique()
        .filter(|i| *i < input.len())
        .map(|i| (i, input.as_bytes()[i] as char))
        .filter(|(_, c)| *c != '.' && !c.is_ascii_digit() && *c != '\n')
}

pub fn part_one(input: &str) -> Option<u32> {
    let width = input.find('\n').unwrap() + 1;

    let result = NumberExtractor::new(input)
        .filter(|(range, _)| {
            // We only need to know if the part is touching a symbol
            get_surrounding_symbols(input, width, range)
                .next()
                .is_some()
        })
        .map(|(_, n)| n)
        .sum();

    Some(result)
}

pub fn part_two(input: &str) -> Option<u32> {
    let width = input.find('\n').unwrap() + 1;

    // Collect all the parts that are touching a gear into a map by the
    // gear that they touch
    let parts_by_gear = NumberExtractor::new(input)
        .flat_map(|(range, num)| {
            get_surrounding_symbols(input, width, &range)
                .map(move |sym| (sym, num))
        })
        .filter(|((_, c), _)| *c == '*')
        .into_group_map_by(|((sym_i, _), _)| *sym_i);

    let result = parts_by_gear
        .values()
        .filter(|number_entries| number_entries.len() == 2)
        .map(|number_entries| {
            number_entries.iter().map(|(_, num)| num).product::<u32>()
        })
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
        assert_eq!(result, Some(4361));
    }

    #[test]
    fn test_part_two() {
        let result =
            part_two(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(467835));
    }
}

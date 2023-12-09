use advent_of_code::ws;
use itertools::Itertools;
use nom::{character::complete::i32, multi::many1, IResult};

advent_of_code::solution!(9);

fn parse_line(line: &str) -> Vec<i32> {
    let result: IResult<_, _> = (many1(ws(i32)))(line);

    result.unwrap().1
}

/// Find the next number in the sequence using the method described applied
/// recursively
fn find_next(nums: &[i32]) -> i32 {
    let mut all_zeroes = true;
    let below = nums
        .iter()
        .tuple_windows()
        .map(|(left, right)| {
            let result = right - left;
            all_zeroes = all_zeroes && result == 0;
            result
        })
        .collect::<Vec<_>>();

    let last = *nums.last().unwrap();

    if all_zeroes {
        last
    } else {
        last + find_next(&below)
    }
}

pub fn part_one(input: &str) -> Option<i32> {
    let result = input
        .lines()
        .map(parse_line)
        .map(|nums| find_next(&nums))
        .sum::<i32>();

    Some(result)
}

pub fn part_two(input: &str) -> Option<i32> {
    let result = input
        .lines()
        .map(parse_line)
        .map(|mut nums| {
            // Part 2 is the same as part 1, but with the number order reversed
            nums.reverse();
            nums
        })
        .map(|nums| find_next(&nums))
        .sum::<i32>();

    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let result =
            part_one(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(114));
    }

    #[test]
    fn test_part_two() {
        let result =
            part_two(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(2));
    }
}

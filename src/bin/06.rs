use advent_of_code::{take_until_inclusive, ws};
use nom::{
    character::complete::{alpha1, u64},
    multi::many1,
    sequence::{preceded, terminated, tuple},
    IResult,
};

advent_of_code::solution!(6);

// Check to see if the number of millis beats the record
fn test_solution(limit: u64, min_dist: u64, time_held: u64) -> bool {
    time_held * (limit - time_held) > min_dist
}

/// Use the quadratic equation to solve
///
///    M = T * (L - T)
///
/// Which is the same as
///
///    0 = T^2 - T * L + M
///
/// Then check around the floating-point solutions to ensure the
/// minimum and maximum solutions returned meet the criteria (They have to be
/// positive integers and produce a distance that is strictly greater than the
/// current best
fn get_real_solutions(limit: u64, min_dist: u64) -> (u64, u64) {
    let a = 1.;
    let b = -(limit as f64);
    let c = min_dist as f64;

    let under_radical = b * b - 4. * a * c;

    // Should only be real solutions
    debug_assert!(under_radical >= 0.);

    let x1 = (-b - under_radical.sqrt()) / (2. * a);
    let x2 = (-b + under_radical.sqrt()) / (2. * a);

    let min = x1.min(x2).floor() as u64;
    let max = x1.max(x2).floor() as u64;

    let min_valid = [-1, 0, 1]
        .into_iter()
        .filter_map(|d| min.checked_add_signed(d))
        .find(|t| test_solution(limit, min_dist, *t))
        .unwrap();

    let max_valid = [1, 0, -1]
        .into_iter()
        .filter_map(|d| max.checked_add_signed(d))
        .find(|t| test_solution(limit, min_dist, *t))
        .unwrap();

    (min_valid, max_valid.min(limit))
}

fn parse(input: &str) -> Vec<(u64, u64)> {
    let result: IResult<_, _> = tuple((
        preceded(
            take_until_inclusive(":"),
            terminated(many1(ws(u64)), alpha1),
        ),
        preceded(take_until_inclusive(":"), many1(ws(u64))),
    ))(input);

    let (top, bottom) = result.unwrap().1;
    top.into_iter().zip(bottom.into_iter()).collect::<Vec<_>>()
}

pub fn part_one(input: &str) -> Option<u64> {
    let races = parse(input);

    let result = races
        .into_iter()
        .map(|(limit, min_dist)| get_real_solutions(limit, min_dist))
        .map(|(low, high)| high - low + 1)
        .product();

    Some(result)
}

pub fn part_two(input: &str) -> Option<u64> {
    // Part 2 is the same as part 1 with spaces removed from the numbers
    let reduced = input.replace(' ', "");
    part_one(&reduced)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let result =
            part_one(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(288));
    }

    #[test]
    fn test_part_two() {
        let result =
            part_two(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(71503));
    }
}

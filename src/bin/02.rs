use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::u32;
use nom::multi::separated_list0;
use nom::sequence::{preceded, terminated, tuple};
use nom::IResult;
advent_of_code::solution!(2);

const RED: &str = "red";
const GREEN: &str = "green";
const BLUE: &str = "blue";

// Hide some of the type complications in this alias
type GameDetails<'a> = Vec<Vec<(u32, &'a str)>>;

// Parse the details of a single game into a list of what's exposed each round
fn parse_views(input: &str) -> IResult<&'_ str, GameDetails<'_>> {
    separated_list0(
        tag("; "),
        separated_list0(
            tag(", "),
            tuple((
                terminated(u32, tag(" ")),
                alt((tag(RED), tag(GREEN), tag(BLUE))),
            )),
        ),
    )(input)
}

// Parse the input into separate game and lists of views
fn parse_games(input: &str) -> Vec<(u32, GameDetails<'_>)> {
    let parse_result: nom::IResult<_, _> = separated_list0(
        tag("\n"),
        tuple((
            preceded(tag("Game "), u32),
            preceded(tag(": "), parse_views),
        )),
    )(input);

    parse_result.unwrap().1
}

pub fn part_one(input: &str) -> Option<u32> {
    let result = parse_games(input)
        .iter()
        .filter(|(_, rounds)| {
            let max_colors =
                rounds.iter().fold((0, 0, 0), |mut accum, view| {
                    view.iter().for_each(|(count, color)| match *color {
                        RED => accum.0 = accum.0.max(*count),
                        GREEN => accum.1 = accum.1.max(*count),
                        BLUE => accum.2 = accum.2.max(*count),
                        _ => unreachable!(),
                    });
                    accum
                });

            matches!(max_colors, (0..=12, 0..=13, 0..=14))
        })
        .map(|(game_index, _)| game_index)
        .sum();

    Some(result)
}

pub fn part_two(input: &str) -> Option<u32> {
    let result = parse_games(input)
        .iter()
        .map(|(_, rounds)| {
            rounds.iter().fold((0, 0, 0), |mut accum, view| {
                view.iter().for_each(|(count, color)| match *color {
                    RED => accum.0 = accum.0.max(*count),
                    GREEN => accum.1 = accum.1.max(*count),
                    BLUE => accum.2 = accum.2.max(*count),
                    _ => unreachable!(),
                });
                accum
            })
        })
        .map(|(r, g, b)| r * g * b)
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
        assert_eq!(result, Some(8));
    }

    #[test]
    fn test_part_two() {
        let result =
            part_two(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(2286));
    }
}

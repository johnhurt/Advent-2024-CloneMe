use advent_of_code::ws;
use nom::bytes::complete::tag;
use nom::character::complete::u32;
use nom::multi::many0;
use nom::sequence::preceded;
use nom::sequence::tuple;
use nom::IResult;
use std::collections::HashMap;
use std::collections::HashSet;

advent_of_code::solution!(4);

// Parse the lines in the input into the number of the card, and the lists of
// winning numbers and guessed numbers for the card
fn parse_cards(
    input: &str,
) -> impl Iterator<Item = (u32, Vec<u32>, Vec<u32>)> + '_ {
    input.lines().map(|line| {
        let result: IResult<&str, _> = tuple((
            preceded(tag("Card"), ws(u32)),
            preceded(tag(":"), many0(ws(u32))),
            preceded(tag("|"), many0(ws(u32))),
        ))(line);

        result.unwrap().1
    })
}

pub fn part_one(input: &str) -> Option<u32> {
    let result = parse_cards(input)
        .map(|(card, winners, guesses)| {
            (card, winners.into_iter().collect::<HashSet<_>>(), guesses)
        })
        .map(|(_, winners, guesses)| {
            guesses
                .into_iter()
                .filter(|guess| winners.contains(guess))
                .count() as u32
        })
        .filter(|correct_guesses| *correct_guesses > 0)
        .map(|correct_guesses| 2_u32.pow(correct_guesses - 1))
        .sum::<u32>();

    Some(result)
}

pub fn part_two(input: &str) -> Option<u32> {
    let mut copies = HashMap::new();

    parse_cards(input)
        .map(|(card, winners, guesses)| {
            (card, winners.into_iter().collect::<HashSet<_>>(), guesses)
        })
        .map(|(card, winners, guesses)| {
            (
                card,
                guesses
                    .into_iter()
                    .filter(|guess| winners.contains(guess))
                    .count() as u32,
            )
        })
        .for_each(|(card, correct)| {
            let card_count = *copies.entry(card).or_insert(1);
            ((card + 1)..=(card + correct)).for_each(|new_card| {
                copies
                    .entry(new_card)
                    .and_modify(|c| *c += card_count)
                    .or_insert(card_count + 1);
            });
        });

    Some(copies.into_values().sum())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let result =
            part_one(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(13));
    }

    #[test]
    fn test_part_two() {
        let result =
            part_two(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(30));
    }
}

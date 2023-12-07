use advent_of_code::ws;
use itertools::Itertools;
use nom::{
    character::complete::anychar, character::complete::u32, combinator::map,
    sequence::tuple, IResult,
};

advent_of_code::solution!(7);

/// Sortable list of kinds of hands
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Kind {
    HighCard,
    Pair,
    TwoPair,
    Three,
    FullHouse,
    Four,
    Five,
}

/// Sortable enumeration of all the cards (based on the problem description)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Card {
    W,
    N(u8),
    T,
    J,
    Q,
    K,
    A,
}

/// Parse a single character into a card
impl From<char> for Card {
    fn from(value: char) -> Self {
        let value = value as u8;
        use Card as C;

        match value {
            b'W' => C::W,
            b'1'..=b'9' => C::N(value - b'0'),
            b'T' => C::T,
            b'J' => C::J,
            b'Q' => C::Q,
            b'K' => C::K,
            b'A' => C::A,
            _ => unreachable!("Unknown character {value}"),
        }
    }
}

/// Struct that is automatically sorted by its cards in order. Thanks, Rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Hand(Card, Card, Card, Card, Card);

/// Struct that is sortable first by hand kind and then by hand
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Game {
    kind: Kind,
    hand: Hand,
    bet: u32,
}

fn parse_hand(input: &str) -> IResult<&'_ str, Hand> {
    map(
        tuple((anychar, anychar, anychar, anychar, anychar)),
        |(c1, c2, c3, c4, c5)| {
            Hand(c1.into(), c2.into(), c3.into(), c4.into(), c5.into())
        },
    )(input)
}

/// Score the given hand by counting the number of cards of each type and then
/// the determines the kind based on the two most repeated cards and the number
/// of wilds
fn score_hand(hand: &Hand) -> Kind {
    let Hand(c1, c2, c3, c4, c5) = hand;
    let mut counts = [c1, c2, c3, c4, c5].iter().counts_by(|c| *c);

    // Handle wilds specially, so don't let them contribute to the normal counts
    let wilds = counts.remove(&Card::W).unwrap_or_default();

    let first_two = counts.values().sorted().rev().tuples().next();

    match (wilds, first_two) {
        (_, None) => Kind::Five,
        (_, Some((4, 1))) => Kind::Four,
        (_, Some((3, 2))) => Kind::FullHouse,
        (1, Some((3, 1))) => Kind::Four,
        (0, Some((3, 1))) => Kind::Three,
        (1, Some((2, 2))) => Kind::FullHouse,
        (0, Some((2, 2))) => Kind::TwoPair,
        (0, Some((2, 1))) => Kind::Pair,
        (1, Some((2, 1))) => Kind::Three,
        (2, Some((2, 1))) => Kind::Four,
        (0, Some((1, 1))) => Kind::HighCard,
        (1, Some((1, 1))) => Kind::Pair,
        (2, Some((1, 1))) => Kind::Three,
        (3, Some((1, 1))) => Kind::Four,
        _ => unreachable!(),
    }
}

fn parse_game(line: &str) -> Game {
    let result: IResult<_, _> = tuple((ws(parse_hand), u32))(line);
    let (hand, bet) = result.unwrap().1;
    let kind = score_hand(&hand);

    Game { kind, hand, bet }
}

fn parse_input(input: &str) -> impl Iterator<Item = Game> + '_ {
    input.lines().map(parse_game)
}

pub fn part_one(input: &str) -> Option<u32> {
    let result = parse_input(input)
        .sorted()
        .enumerate()
        .map(|(i, Game { bet, .. })| (i as u32 + 1) * bet)
        .sum::<u32>();

    Some(result)
}

pub fn part_two(input: &str) -> Option<u32> {
    // Swap "J"-Jacks with "W"-Wilds, and run part 1 again
    let with_wilds = input.replace('J', "W");
    part_one(&with_wilds)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let result =
            part_one(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(6440));
    }

    #[test]
    fn test_part_two() {
        let result =
            part_two(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(5905));
    }
}

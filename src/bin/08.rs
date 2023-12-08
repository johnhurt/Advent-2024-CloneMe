use std::collections::HashMap;
use std::str::FromStr;

use advent_of_code::ws;
use itertools::Itertools;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::alphanumeric1;
use nom::combinator::map;
use nom::multi::{many1, separated_list1};
use nom::sequence::{delimited, preceded, tuple};
use nom::IResult;
use strum_macros::EnumString;

advent_of_code::solution!(8);

#[derive(Debug, Clone, Copy, EnumString)]
enum LR {
    L,
    R,
}

type Node<'a> = (&'a str, (&'a str, &'a str));
type Graph<'a> = HashMap<&'a str, (&'a str, &'a str)>;

fn parse_graph_entry(input: &str) -> IResult<&'_ str, Node<'_>> {
    tuple((
        ws(alphanumeric1),
        delimited(
            tag("= ("),
            tuple((ws(alphanumeric1), preceded(tag(","), ws(alphanumeric1)))),
            tag(")"),
        ),
    ))(input)
}

fn parse(input: &str) -> (Vec<LR>, Graph<'_>) {
    let result: IResult<_, _> = tuple((
        many1(map(alt((tag("L"), tag("R"))), |c| LR::from_str(c).unwrap())),
        map(
            preceded(
                tag("\n\n"),
                separated_list1(tag("\n"), parse_graph_entry),
            ),
            |entries| entries.into_iter().collect::<HashMap<_, _>>(),
        ),
    ))(input);

    result.unwrap().1
}

/// Take a left/right step in the given graph and return the result node
fn graph_step<'a>(graph: &'a Graph<'a>, turn: LR, curr: &'a str) -> &'a str {
    match (turn, graph.get(curr)) {
        (LR::L, Some((next, _))) | (LR::R, Some((_, next))) => next,
        _ => unreachable!("No orphan nodes"),
    }
}

pub fn part_one(input: &str) -> Option<u32> {
    let (turns, graph) = parse(input);

    let mut curr = "AAA";
    let result = turns
        .iter()
        .cycle()
        .take_while_inclusive(|turn| {
            curr = graph_step(&graph, **turn, curr);
            curr != "ZZZ"
        })
        .count();

    Some(result as u32)
}

/// This is straight from from wikipedia -
/// https://en.wikipedia.org/wiki/Extended_Euclidean_algorithm#Pseudocode
fn extended_gcd(a: i128, b: i128) -> (i128, i128, i128) {
    let (mut old_r, mut r) = (a, b);
    let (mut old_s, mut s) = (1, 0);
    let (mut old_t, mut t) = (0, 1);

    while r != 0 {
        let q = old_r / r;
        (old_r, r) = (r, old_r - q * r);
        (old_s, s) = (s, old_s - q * s);
        (old_t, t) = (t, old_t - q * t);
    }

    (old_r, old_s, old_t)
}

/// Represents the repeat of a sequence. The repeat starts at a given point
/// in the pattern (prefix) and then repeats every so ofter after that (cycle)
#[derive(Debug, PartialEq, Clone, Copy)]
struct Repeat {
    prefix: usize,
    cycle: usize,
}

impl Repeat {
    /// Ridiculous method of finding the combined repeat for two repeats by
    /// using math to find integer solutions to p1 + c1 * N = p2 + c2 * M
    /// of the form p3 + LCM(c1, c2) * K. I got this method from this post
    /// https://math.stackexchange.com/questions/20717/
    ///
    /// Note. the repeats in this puzzle are contrived to be c = p + 1, but
    /// I couldn't figure out how to use that fact to find the answer. Instead,
    /// this is the generalized solution
    fn fold(self, other: Self) -> Self {
        let c = other.prefix as i128 - self.prefix as i128;

        let a = self.cycle as i128;
        let b = other.cycle as i128;

        let (gcd, s, _) = extended_gcd(a, -b);
        let gcd = gcd;

        let k = if c % gcd == 0 { c / gcd } else { c };

        let mut signed_prefix = k * s * a + self.prefix as i128;
        let signed_cycle = a * b / gcd.abs();

        let extras = signed_prefix / signed_cycle;

        signed_prefix -= extras * signed_cycle;

        if signed_prefix < 0 {
            signed_prefix += signed_cycle;
        }

        Repeat {
            prefix: signed_prefix.try_into().unwrap(),
            cycle: signed_cycle.try_into().unwrap(),
        }
    }
}

/// This puzzle's repeats are contrived (at least in my input), so that nothing
/// complicated happens. Each ghost only ever visits one end node, and the
/// repeat rate is immediately constant. This function evaluates the ghost's
/// path until it hits the same node twice and then returns the repeat info.
fn find_repeat(graph: &Graph<'_>, start: &'_ str, turns: &[LR]) -> Repeat {
    let mut ends_visited: HashMap<&str, Vec<usize>> = HashMap::new();

    let mut curr = start;

    turns
        .iter()
        .cycle()
        .map(|turn| {
            curr = graph_step(graph, *turn, curr);
            curr
        })
        .enumerate()
        .filter(|(_, c)| c.ends_with('Z'))
        .take_while_inclusive(|(i, curr)| {
            let e = ends_visited
                .entry(curr)
                .and_modify(|visits| visits.push(*i))
                .or_insert(vec![*i]);
            e.len() < 2
        })
        .count();

    ends_visited
        .values()
        .next()
        .and_then(|times| {
            times
                .iter()
                .copied()
                .tuple_windows()
                .map(|(left, right)| Repeat {
                    prefix: left,
                    cycle: right - left,
                })
                .next()
        })
        .unwrap()
}

pub fn part_two(input: &str) -> Option<usize> {
    let (turns, graph) = parse(input);

    let Repeat { prefix, .. } = graph
        .keys()
        .copied()
        .filter(|n| n.ends_with('A'))
        .map(|start| find_repeat(&graph, start, &turns))
        .tree_fold1(|prev, next| prev.fold(next))
        .unwrap();

    // Plus one here because our counts are zero based, but the answer is number
    // of turns
    Some(prefix + 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let result =
            part_one(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(2));
    }

    #[test]
    fn test_part_one_again() {
        let result = part_one(
            &advent_of_code::template::read_extra_example_file(DAY, 1),
        );
        assert_eq!(result, Some(6));
    }

    #[test]
    fn test_part_two() {
        let result = part_two(
            &advent_of_code::template::read_extra_example_file(DAY, 2),
        );
        assert_eq!(result, Some(6));
    }

    #[test]
    fn test_gcd() {
        assert_eq!((-1, 1, 1), extended_gcd(2, -3));
        assert_eq!((1, 3, 2), extended_gcd(5, -7));
        assert_eq!((3, 62947, 44752), extended_gcd(523425, -736236));
    }

    #[test]
    fn test_repeat_fold() {
        assert_eq!(
            Repeat {
                prefix: 17,
                cycle: 35
            },
            Repeat {
                prefix: 2,
                cycle: 5
            }
            .fold(Repeat {
                prefix: 3,
                cycle: 7
            })
        )
    }
}

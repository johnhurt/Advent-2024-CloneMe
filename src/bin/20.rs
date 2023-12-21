use std::collections::{HashMap, HashSet, VecDeque};

use advent_of_code::ws;
use itertools::Itertools;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::alpha1;
use nom::combinator::map;
use nom::multi::separated_list1;
use nom::sequence::{preceded, tuple};
use nom::IResult;

advent_of_code::solution!(20);

const BUTTON: &str = "_button_";
const BROADCAST: &str = "_broadcast_";

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum HiLo {
    #[default]
    Lo,
    Hi,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum FlipState {
    #[default]
    Off,
    On,
}

impl FlipState {
    fn flip(&mut self) -> HiLo {
        use FlipState as F;
        *self = match self {
            F::On => F::Off,
            F::Off => F::On,
        };

        match self {
            F::Off => HiLo::Lo,
            F::On => HiLo::Hi,
        }
    }
}

#[derive(Debug)]
enum Node<'a> {
    Broadcast(Vec<&'a str>),
    Flip {
        outs: Vec<&'a str>,
        state: FlipState,
    },
    Conj {
        ins: HashMap<&'a str, HiLo>,
        outs: Vec<&'a str>,
        prev: Option<HiLo>,
        fired: Option<usize>,
        firings: HashSet<(usize, usize)>,
    },
    Dud,
    Rx(bool, Vec<&'a str>),
}

impl<'a> Node<'a> {
    fn reset(&mut self) {
        match self {
            Node::Flip { ref mut state, .. } => *state = FlipState::Off,
            Node::Conj {
                ref mut ins,
                ref mut prev,
                ref mut fired,
                ref mut firings,
                ..
            } => {
                ins.iter_mut().for_each(|(_, s)| *s = HiLo::Lo);
                *prev = None;
                *fired = None;
                firings.clear();
            }
            Node::Rx(ref mut hit, ..) => *hit = false,
            _ => {}
        }
    }
}

fn parse_broadcast(input: &str) -> IResult<&'_ str, (&'_ str, Node<'_>)> {
    map(
        preceded(tag("broadcaster ->"), separated_list1(tag(","), ws(alpha1))),
        |outs| (BROADCAST, Node::Broadcast(outs)),
    )(input)
}

fn parse_flip_flop(input: &str) -> IResult<&'_ str, (&'_ str, Node<'_>)> {
    map(
        tuple((
            preceded(tag("%"), alpha1),
            preceded(ws(tag("->")), separated_list1(tag(","), ws(alpha1))),
        )),
        |(name, outs)| {
            (
                name,
                Node::Flip {
                    outs,
                    state: Default::default(),
                },
            )
        },
    )(input)
}

fn parse_conjunction(input: &str) -> IResult<&'_ str, (&'_ str, Node<'_>)> {
    map(
        tuple((
            preceded(tag("&"), alpha1),
            preceded(ws(tag("->")), separated_list1(tag(","), ws(alpha1))),
        )),
        |(name, outs)| {
            (
                name,
                Node::Conj {
                    ins: HashMap::new(),
                    outs,
                    prev: None,
                    fired: None,
                    firings: HashSet::new(),
                },
            )
        },
    )(input)
}

fn parse_line(line: &str) -> (&'_ str, Node<'_>) {
    let result: IResult<_, _> =
        alt((parse_broadcast, parse_flip_flop, parse_conjunction))(line);

    result.unwrap().1
}

fn parse(input: &str) -> HashMap<&'_ str, Node<'_>> {
    let mut result = input.lines().map(parse_line).collect::<HashMap<_, _>>();

    let reversed = result
        .iter()
        .flat_map(|(name, node)| match node {
            Node::Flip { outs, .. } | Node::Conj { outs, .. } => {
                outs.iter().map(|out| (*out, *name)).collect::<Vec<_>>()
            }
            _ => vec![],
        })
        .into_group_map();

    reversed
        .into_iter()
        .for_each(|(to, from)| match result.get_mut(&to) {
            Some(Node::Conj { ref mut ins, .. }) => {
                ins.extend(
                    from.into_iter().map(move |inward| (inward, HiLo::Lo)),
                );
            }
            None => {
                if to == "rx" {
                    result.insert(to, Node::Rx(false, from));
                } else {
                    result.insert(to, Node::Dud);
                }
            }
            _ => {}
        });

    result
}

fn click<'a>(nodes: &mut HashMap<&'a str, Node<'a>>) -> (usize, usize) {
    let mut low = 0;
    let mut high = 0;

    let mut queue = VecDeque::new();

    queue.push_back((0, BUTTON, BROADCAST, HiLo::Lo));

    while let Some((gen, from_node, node_name, hilo)) = queue.pop_front() {
        match hilo {
            HiLo::Lo => low += 1,
            HiLo::Hi => high += 1,
        };

        match (hilo, nodes.get_mut(node_name).unwrap()) {
            (HiLo::Lo, Node::Rx(ref mut hit, ..)) => *hit = true,
            (_, Node::Broadcast(outs)) => {
                queue.extend(
                    outs.iter().map(|out| (gen + 1, node_name, *out, HiLo::Lo)),
                );
            }
            (
                HiLo::Lo,
                Node::Flip {
                    outs,
                    ref mut state,
                },
            ) => {
                let next_p = state.flip();
                queue.extend(
                    outs.iter().map(|out| (gen + 1, node_name, *out, next_p)),
                );
            }
            (
                _,
                Node::Conj {
                    ref mut ins,
                    outs,
                    ref mut prev,
                    ref mut fired,
                    ref mut firings,
                },
            ) => {
                if let Some(ref mut in_prev) = ins.get_mut(from_node) {
                    **in_prev = hilo;
                    let all_hi = ins.values().all(|curr| *curr == HiLo::Hi);
                    let next_p = if all_hi { HiLo::Lo } else { HiLo::Hi };

                    *prev = Some(next_p);

                    if all_hi && fired.is_none() {
                        *fired = Some(gen);
                    }
                    if !all_hi && fired.is_some() {
                        firings.insert((fired.take().unwrap(), gen));
                        *fired = None;
                    }

                    // AAAGGGH I wasted so much time because I didn't realize
                    // that VecDeq extends on the the back of the queue!
                    queue.extend(
                        outs.iter()
                            .map(|out| (gen + 1, node_name, *out, next_p)),
                    );
                }
                let _ = ins.entry(from_node).and_modify(|prev| *prev = hilo);
            }
            _ => {}
        }
    }

    (high, low)
}

pub fn part_one(input: &str) -> Option<usize> {
    let mut nodes = parse(input);
    let mut total_high = 0;
    let mut total_low = 0;

    for _ in 0..1000 {
        let (high, low) = click(&mut nodes);
        total_high += high;
        total_low += low;
    }

    Some(total_high * total_low)
}

fn get_deps<'a>(
    name: &'a str,
    nodes: &HashMap<&'a str, Node<'a>>,
) -> Vec<&'a str> {
    match nodes.get(name).unwrap() {
        Node::Rx(_, ins) => ins.clone(),
        Node::Conj { ins, .. } => ins.iter().map(|(i, _)| *i).collect_vec(),
        _ => vec![],
    }
}

fn find_repeat<'a>(
    name: &'a str,
    nodes: &mut HashMap<&'a str, Node<'a>>,
) -> usize {
    nodes.values_mut().for_each(Node::reset);

    let mut result = 0;

    let mut rounds = HashSet::new();

    while let Some(Node::Conj {
        ref mut firings, ..
    }) = nodes.get_mut(name)
    {
        if !firings.is_empty() {
            rounds.insert(firings.drain().collect_vec());
            break;
        }
        if result > 0 && result % 1_000_000 == 0 {
            println!("No repeats after {result} clicks");
        }

        result += 1;
        click(nodes);
    }

    result
}

pub fn part_two(input: &str) -> Option<usize> {
    let mut nodes = parse(input);

    // great-grand children of the rx node are all conjugators, and they feed
    // into a single conjugator, so whenever Highs from these nodes line up
    // the rx node gets a Low
    let ggc = get_deps("rx", &nodes)
        .into_iter()
        .flat_map(|n| get_deps(n, &nodes).into_iter())
        .flat_map(|n| get_deps(n, &nodes).into_iter())
        .collect_vec();

    // Luckily all the frequencies of these nodes line up perfectly, and (for me
    // at least) the values were all co prime, so high signals from these line
    // up only at the product of the individual frequencies
    let repeat_freq = ggc
        .iter()
        .map(|n| (*n, find_repeat(n, &mut nodes)))
        .collect::<HashMap<_, _>>();

    let result = repeat_freq.values().product();

    Some(result)
}
// 19685294027512 + 2 * 86635709125300
// 106321003152812
// 173271418250600
// 211106232532992
// 149814466590512
//
//   2  192956712278112
// lb - 211106232532992
//   3  279592421403412
// ub - 281474976710656
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let result =
            part_one(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(32000000));
    }

    #[test]
    fn test_part_one_again() {
        let result = part_one(
            &advent_of_code::template::read_extra_example_file(DAY, 1),
        );
        assert_eq!(result, Some(11687500));
    }

    #[test]
    fn test_part_two() {
        let result = part_two(
            &advent_of_code::template::read_extra_example_file(DAY, 1),
        );
        assert_eq!(result, Some(1));
    }
}

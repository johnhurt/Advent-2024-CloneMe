use std::collections::{hash_map::Entry, BTreeMap, HashMap, HashSet, VecDeque};

use advent_of_code::ws;
use itertools::Itertools;
use nom::{
    bytes::complete::tag,
    character::complete::alpha1,
    multi::many1,
    sequence::{terminated, tuple},
    IResult,
};

advent_of_code::solution!(25);

type G<'a> = HashMap<&'a str, HashSet<&'a str>>;
type E<'a> = (&'a str, &'a str);

fn parse_line(line: &str) -> (&'_ str, Vec<&'_ str>) {
    let result: IResult<_, _> =
        tuple((terminated(alpha1, tag(":")), many1(ws(alpha1))))(line);

    result.unwrap().1
}

fn parse(input: &str) -> G {
    input
        .lines()
        .map(parse_line)
        .flat_map(|(from, tos)| {
            tos.into_iter()
                .flat_map(move |too| [(from, too), (too, from)])
        })
        .into_group_map()
        .into_iter()
        .map(|(k, vs)| (k, vs.into_iter().collect::<HashSet<_>>()))
        .collect::<HashMap<_, _>>()
}

fn search_from<'a>(node: &'a str, g: &G<'a>) -> HashMap<&'a str, usize> {
    let mut to_do = VecDeque::from(vec![(node, 0)]);
    let mut seen = HashMap::new();

    let mut max_depth = 0;

    while let Some((curr, dist)) = to_do.pop_front() {
        if let Entry::Vacant(e) = seen.entry(curr) {
            e.insert(dist);
        } else {
            continue;
        }

        max_depth = max_depth.max(dist);

        to_do.extend(g.get(curr).unwrap().iter().map(|n| (*n, dist + 1)));
    }

    seen
}

fn remove<'a>(e: E<'a>, g: &mut G<'a>) {
    g.get_mut(e.0).unwrap().remove(e.1);
    g.get_mut(e.1).unwrap().remove(e.0);
}

fn add<'a>(e: E<'a>, g: &mut G<'a>) {
    g.get_mut(e.0).unwrap().insert(e.1);
    g.get_mut(e.1).unwrap().insert(e.0);
}

fn try_cut<'a>(
    e1: E<'a>,
    e2: E<'a>,
    e3: E<'a>,
    g: &mut G<'a>,
) -> Option<(usize, usize)> {
    let total = g.len();
    let es = vec![e1, e2, e3];

    es.iter().copied().for_each(|e| remove(e, g));

    let searched_from = search_from(e1.0, g).len();

    if searched_from != total {
        return Some((searched_from, total - searched_from));
    }

    es.iter().copied().for_each(|e| add(e, g));
    None
}

pub fn part_one(input: &str) -> Option<usize> {
    let mut g = parse(input);

    let possible_edges = g
        .keys()
        .map(|n| (search_from(n, &g).into_values().max().unwrap(), n))
        .into_group_map()
        .into_iter()
        .collect::<BTreeMap<_, _>>()
        .into_iter()
        .flat_map(|(_, v)| v.into_iter())
        .flat_map(|n1| g.get(n1).unwrap().iter().map(|n2| (*n1, *n2)))
        .take(100)
        .collect_vec();

    for i in 0..possible_edges.len() {
        for j in (i + 1)..possible_edges.len() {
            for k in (j + 1)..possible_edges.len() {
                let e1 = possible_edges.get(i).unwrap();
                let e2 = possible_edges.get(j).unwrap();
                let e3 = possible_edges.get(k).unwrap();

                if let Some((p1, p2)) = try_cut(*e1, *e2, *e3, &mut g) {
                    return Some(p1 * p2);
                }
            }
        }
    }

    unreachable!()
}

pub fn part_two(_: &str) -> Option<u32> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let result =
            part_one(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(54));
    }

    #[test]
    fn test_part_two() {
        let result =
            part_two(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, None);
    }
}

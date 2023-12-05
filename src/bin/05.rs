use advent_of_code::ws;
use itertools::Itertools;
use nom::{
    bytes::complete::tag,
    bytes::complete::take_until,
    character::complete::u64,
    combinator::{map, opt},
    multi::{many0, separated_list1},
    sequence::{preceded, terminated, tuple},
    IResult,
};
use std::collections::BTreeMap;

advent_of_code::solution!(5);

/// Represents one line in the map-description section
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MapEntry {
    src_start: u64,
    dest_start: u64,
    len: u64,
}

impl MapEntry {
    // Apply this mapping against the given key. If the key is within the source
    // range, it will be mapped. If it isn't, it won't
    fn apply(&self, key: u64) -> u64 {
        if (self.src_start..(self.src_start + self.len)).contains(&key) {
            self.dest_start + (key - self.src_start)
        } else {
            key
        }
    }

    // Reverse this range, so the mapping can be applied in reverse
    fn reverse(&self) -> Self {
        MapEntry {
            src_start: self.dest_start,
            dest_start: self.src_start,
            len: self.len,
        }
    }
}

// Parse the part of the input that lists the seeds
fn parse_seeds(input: &str) -> IResult<&'_ str, Vec<u64>> {
    terminated(
        preceded(tag("seeds: "), separated_list1(tag(" "), u64)),
        tag("\n\n"),
    )(input)
}

fn parse_map_entries(input: &str) -> IResult<&'_ str, Vec<MapEntry>> {
    separated_list1(
        tag("\n"),
        map(
            tuple((u64, ws(u64), u64)),
            |(dest_start, src_start, len)| MapEntry {
                src_start,
                dest_start,
                len,
            },
        ),
    )(input)
}

// Parse a single section of the input that describes a mapping
fn parse_map(input: &str) -> IResult<&'_ str, BTreeMap<u64, MapEntry>> {
    terminated(
        preceded(
            preceded(take_until("\n"), tag("\n")),
            map(parse_map_entries, |entries| {
                entries
                    .into_iter()
                    .map(|e| (e.src_start, e))
                    .collect::<BTreeMap<_, _>>()
            }),
        ),
        opt(tag("\n\n")),
    )(input)
}

// Evaluate the mapping of a key against a collection of mappings. If the key
// falls in one of the mapped ranges, it will be mapped as described. If it
// doesn't fall within one of the ranges, it will be left unchanged
fn eval_single_map(map: &BTreeMap<u64, MapEntry>, key: u64) -> u64 {
    if let Some((_, entry)) = map.range(..=key).next_back() {
        entry.apply(key)
    } else {
        key
    }
}

// The mappings in this problem are applied one after another in a chain. We
// don't need to know anything about the intermediate values. We just need the
// order to be correct
struct MapChain(Vec<BTreeMap<u64, MapEntry>>);

impl MapChain {
    // parse the chain of maps from the input
    fn parse(input: &str) -> IResult<&'_ str, Self> {
        map(many0(parse_map), Self)(input)
    }

    // evaluate all of the mappings in order for the given key
    fn eval(&self, start_key: u64) -> u64 {
        self.0
            .iter()
            .fold(start_key, |key, map| eval_single_map(map, key))
    }

    // Get a map that is the reverse of this one
    fn reverse(&self) -> Self {
        let reversed_maps = self
            .0
            .iter()
            .rev()
            .map(|entries| {
                entries
                    .values()
                    .map(MapEntry::reverse)
                    .map(|e| (e.src_start, e))
                    .collect::<BTreeMap<_, _>>()
            })
            .collect_vec();

        Self(reversed_maps)
    }
}

fn parse_input(input: &str) -> (Vec<u64>, MapChain) {
    let result: IResult<_, _> = tuple((parse_seeds, MapChain::parse))(input);

    result.unwrap().1
}

pub fn part_one(input: &str) -> Option<u64> {
    let (seeds, maps) = parse_input(input);
    seeds.into_iter().map(|seed| maps.eval(seed)).min()
}

pub fn part_two(input: &str) -> Option<u64> {
    let (seeds, maps) = parse_input(input);
    let seed_ranges = seeds
        .into_iter()
        .tuples()
        .map(|(start, len)| (start, start..(start + len)))
        .collect::<BTreeMap<_, _>>();

    let rev_maps = maps.reverse();

    // search up through locations until we find one with a valid seed
    (0..)
        .map(|location| (location, rev_maps.eval(location)))
        .find(|(_, seed)| {
            seed_ranges
                .range(..=seed)
                .next_back()
                .filter(|(_, range)| range.contains(seed))
                .is_some()
        })
        .map(|(location, _)| location)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let result =
            part_one(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(35));
    }

    #[test]
    fn test_part_two() {
        let result =
            part_two(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(46));
    }
}

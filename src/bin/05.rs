use advent_of_code::{intersection, ws};
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
use std::{collections::BTreeMap, ops::Range};

advent_of_code::solution!(5);

/// Represents one line in the map-description section
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MapEntry {
    src_start: u64,
    dest_start: u64,
    len: u64,
}

// This is just to make clippy happy. This type is used below to do tracking
type BeforeDuringAfter =
    (Option<Range<u64>>, Option<Range<u64>>, Option<Range<u64>>);

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

    // Apply this mapping to a range instead of a single value.
    fn apply_range(&self, range: &Range<u64>) -> Range<u64> {
        self.apply(range.start)..self.apply(range.end - 1) + 1
    }

    // Get the range of the source that is mapped by this mapping
    fn src_range(&self) -> Range<u64> {
        self.src_start..(self.src_start + self.len)
    }

    // Track how the given range is mapped by this entry (before, during, after)
    fn track_ranges(&self, range: Range<u64>) -> BeforeDuringAfter {
        let src_range = self.src_range();
        let before = intersection(&range, &(0..src_range.start));
        let during_opt = intersection(&src_range, &range);
        let during_mapped = during_opt.map(|during| self.apply_range(&during));
        let after = intersection(&range, &(src_range.end..range.end));

        (before, during_mapped, after)
    }
}

// Parse the part of the input that lists the seeds
fn parse_seeds(input: &str) -> IResult<&'_ str, Vec<u64>> {
    terminated(
        preceded(tag("seeds: "), separated_list1(tag(" "), u64)),
        tag("\n\n"),
    )(input)
}

// Parse the whole section of the input that contains the map entries including
// the title line
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

// Trace how the the given set of ranges are mapped by the all the mappings in
// in the given ordered set. This is done by checking how each range is mapped/
// split by each mapping and then combining them in the end into a new sorted
// map of ranges
fn trace_single_map_ranges(
    ranges: &BTreeMap<u64, Range<u64>>,
    map: &BTreeMap<u64, MapEntry>,
) -> BTreeMap<u64, Range<u64>> {
    ranges
        .values()
        .map(|in_range| {

            let first = map.range(..=in_range.start).next_back();
            let rest = map.range(in_range.clone());

            (
                in_range,
                first.into_iter().chain(rest).map(|(_, e)| e)
            )
        })
        .flat_map(|(in_range, entries)| {
            let mut rest = Some(in_range.clone());
            let mut result = vec![];

            for entry in entries {
                if rest.is_none() {
                    break;
                }
                match entry.track_ranges(rest.unwrap()) {
                    (Some(before), Some(during), after) => {
                        rest = after;
                        result.push(before);
                        result.push(during);
                    }
                    (None, Some(during), after) => {
                        rest = after;
                        result.push(during);
                    }
                    (Some(_), None, _) => {
                        unreachable!("We shouldn't see any before range if range is empty");
                    }
                    (None, None, after) => {
                        rest = after;
                    }
                }
            }

            if let Some(after) = rest {
                result.push(after);
            }

            result
        })
        .map(|e| (e.start, e))
        .collect::<BTreeMap<_, _>>()
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

    // trace and record how all the given ranges are mapped and split by all the
    // mappings in this chain
    fn trace_ranges(
        &self,
        ranges: BTreeMap<u64, Range<u64>>,
    ) -> BTreeMap<u64, Range<u64>> {
        self.0.iter().fold(ranges, |in_ranges, map| {
            trace_single_map_ranges(&in_ranges, map)
        })
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

    maps.trace_ranges(seed_ranges).keys().copied().next()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_trace() {
        let e = MapEntry {
            src_start: 64,
            dest_start: 68,
            len: 13,
        };

        let (before, during, after) = e.track_ranges(74..95);

        assert_eq!(before, None);
        assert_eq!(during, Some(78..81));
        assert_eq!(after, Some(77..95));
    }

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

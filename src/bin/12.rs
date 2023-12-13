use std::collections::HashMap;

use advent_of_code::ws;
use nom::bytes::complete::{tag, take_until};
use nom::character::complete::u8;
use nom::combinator::map;
use nom::multi::separated_list1;
use nom::sequence::tuple;
use nom::IResult;

advent_of_code::solution!(12);

/// Parse a line into its masks and the list of contiguous sizes
fn parse_line(line: &str) -> (SpringMasks, Vec<u8>) {
    let result: IResult<_, _> = tuple((
        map(take_until(" "), SpringMasks::parse),
        ws(separated_list1(tag(","), u8)),
    ))(line);

    result.unwrap().1
}

/// Enumeration of the state a spring can be in. `Good` should be here, but it
/// isn't used in the solution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Bad,
    Unknown,
}

/// Store the information about the working and non-working springs as bit masks
/// where the bits go from least significant to most significant when going
/// from right to left. This is assuming that no rows are bigger than 32s. The
/// len tells how far into the significant bits the mask goes.
#[derive(Debug, Default, Clone, Copy)]
struct SpringMasks {
    len: u8,
    known_good: u128,
    known_bad: u128,
    unknown: u128,
}

impl SpringMasks {
    fn parse(value: &str) -> Self {
        let len = value.len() as u8;
        let result = SpringMasks {
            len,
            ..SpringMasks::default()
        };

        let (result, _) =
            value
                .chars()
                .rev()
                .fold((result, 1), |(mut masks, cursor), c| {
                    match c {
                        '.' => masks.known_good |= cursor,
                        '#' => masks.known_bad |= cursor,
                        '?' => masks.unknown |= cursor,
                        _ => unreachable!(),
                    };
                    (masks, cursor << 1)
                });

        result
    }

    /// Take the raw input and concatenate it 5 times to get the input for part
    /// 2.
    fn quintuple(self) -> Self {
        let Self {
            mut len,
            mut known_bad,
            mut known_good,
            mut unknown,
        } = self;

        let shift = len as u128 + 1;
        len = 5 * len + 4;

        for _ in 0..4 {
            known_bad |= known_bad << shift;
            known_good |= known_good << shift;
            unknown |= unknown << shift;
            unknown |= 1 << (shift - 1)
        }

        Self {
            len,
            known_bad,
            known_good,
            unknown,
        }
    }

    /// Get the index of the next spring that is either known bad or unknown
    fn next_known_bad_or_unknown(&self, start: u8) -> Option<(Kind, u8)> {
        if start > self.len {
            return None;
        }

        let shift = 128 - self.len + start;

        (shift < 128).then(|| {
            let known_bad = self.known_bad << shift;
            let unknown = self.unknown << shift;

            let leading_bad = known_bad.leading_zeros() as u8;
            let leading_unknown = unknown.leading_zeros() as u8;

            if leading_bad < leading_unknown {
                (Kind::Bad, leading_bad + start)
            } else {
                (Kind::Unknown, leading_unknown + start)
            }
        })
    }

    /// Check to see if it's valid to place a segment with the given length
    /// at the given starting point. This means the segment doesn't overlap any
    /// known-good springs and the springs on either side aren't known-bad
    fn check_segment_placement(&self, start: u8, seg_len: u8) -> bool {
        (start < self.len) && {
            let mut seg_mask: u128 = (1 << seg_len) - 1;
            let seg_shift = self.len - start - seg_len;
            seg_mask <<= seg_shift;

            let mut bound_check_mask: u128 = ((1 << seg_len) << 1) + 1;
            let bound_check_shift =
                self.len as i8 - start as i8 + 1 - seg_len as i8 - 2;

            if bound_check_shift > 0 {
                bound_check_mask <<= bound_check_shift as u8;
            } else {
                bound_check_mask >>= (-bound_check_shift) as u8;
            }

            self.known_good & seg_mask == 0
                && self.known_bad & bound_check_mask == 0
        }
    }

    /// Check the region after (and including) the given start point, and
    /// return true if it has any springs that are known bad. This is used
    /// in the solution checker to verify that when the solution runs out of
    /// of segments, there are no springs that need to be part of segments
    /// remaining
    fn any_bad_after(&self, start: u8) -> bool {
        if start > self.len {
            return false;
        }
        let mask_len = self.len - start;
        let mask: u128 = (1 << mask_len) - 1;

        self.known_bad & mask > 0
    }
}

/// Mechanism for iterating over all the possible positions a segment of a
/// given size can take
struct ValidPositionIter<'a> {
    masks: &'a SpringMasks,
    next_check: u8,
    seg_len: u8,
    max_start: u8,
    seen_bad: bool,
}

impl<'a> ValidPositionIter<'a> {
    fn new(masks: &'a SpringMasks, state: State, segments: &'a [u8]) -> Self {
        let seg_len = segments[state.segment_start as usize];
        let remains = &segments[(state.segment_start as usize + 1)..];
        let max_start = masks.len
            - (remains.iter().sum::<u8>() + remains.len() as u8 + seg_len);

        Self {
            masks,
            next_check: state.spring_start,
            seg_len,
            max_start,
            seen_bad: false,
        }
    }
}

impl<'a> Iterator for ValidPositionIter<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((kind, next_p)) =
            self.masks.next_known_bad_or_unknown(self.next_check)
        {
            self.next_check = next_p + 1;

            if self.seen_bad {
                // If a known-bad spring has been seen by this iterator already,
                // there are no more possible positions because positions past
                // the known-bad spring will create an orphaned bad spring
                return None;
            }

            if kind == Kind::Bad {
                self.seen_bad = true;
            }

            if next_p <= self.max_start
                && self.masks.check_segment_placement(next_p, self.seg_len)
            {
                return Some(next_p);
            }
        }

        None
    }
}

/// This is our hash key for working through dynamic programming for this
/// puzzle. At each step we need to know where we are starting and which
/// segments remain to be placed
#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq)]
struct State {
    spring_start: u8,
    segment_start: u8,
}

/// Dynamic programming solution. We count all the possible (and valid)
/// positions of each segment recursively. This is a brute force way of checking
/// all combinations of segment positions, but it isn't horribly slow because
/// we cache the results each time, and the search space is pretty small
fn recursive_solver(
    state: State,
    masks: &SpringMasks,
    segments: &'_ [u8],
    cache: &'_ mut HashMap<State, usize>,
) -> usize {
    if let Some(cached) = cache.get(&state).copied() {
        cached
    } else if let Some(seg_len) =
        segments.get(state.segment_start as usize).copied()
    {
        let seg_pos_iter = ValidPositionIter::new(masks, state, segments);

        let result = seg_pos_iter
            .map(|seg_start| {
                let new_state = State {
                    spring_start: seg_start + seg_len + 1,
                    segment_start: state.segment_start + 1,
                };
                recursive_solver(new_state, masks, segments, cache)
            })
            .sum();

        cache.insert(state, result);

        result
    } else if masks.any_bad_after(state.spring_start) {
        0
    } else {
        1
    }
}

pub fn part_one(input: &str) -> Option<usize> {
    let result = input
        .lines()
        .map(parse_line)
        .map(|(masks, segments)| {
            let mut cache = HashMap::new();
            let state = State::default();
            recursive_solver(state, &masks, &segments, &mut cache)
        })
        .sum::<usize>();

    Some(result)
}

pub fn part_two(input: &str) -> Option<usize> {
    let result = input
        .lines()
        .map(parse_line)
        .map(|(mask, segments)| (mask.quintuple(), segments.repeat(5)))
        .map(|(masks, segments)| {
            let mut cache = HashMap::new();
            let state = State::default();
            recursive_solver(state, &masks, &segments, &mut cache)
        })
        .sum::<usize>();

    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recursive_solve() {
        let masks = SpringMasks::parse("#.??#?#??.");
        let segments = vec![1u8, 3];
        let mut cache = HashMap::new();
        let state = State::default();
        let result = recursive_solver(state, &masks, &segments, &mut cache);

        assert_eq!(result, 1);
    }

    #[test]
    fn test_recursive_solve_2() {
        let masks = SpringMasks::parse(".??..??...?##");
        let segments = vec![1u8, 1, 3];
        let mut cache = HashMap::new();
        let state = State::default();
        let result = recursive_solver(state, &masks, &segments, &mut cache);

        assert_eq!(result, 4);
    }

    #[test]
    fn test_next_bad_or_unknown() {
        let masks = SpringMasks::parse("?.#");
        assert_eq!(
            masks.next_known_bad_or_unknown(0),
            Some((Kind::Unknown, 0))
        );
        assert_eq!(masks.next_known_bad_or_unknown(1), Some((Kind::Bad, 2)));
        assert_eq!(masks.next_known_bad_or_unknown(2), Some((Kind::Bad, 2)));
        assert_eq!(masks.next_known_bad_or_unknown(3), None);
    }

    #[test]
    fn test_check_segment_placement() {
        let masks = SpringMasks::parse("???.###");

        assert!(!masks.check_segment_placement(4, 2));
        assert!(!masks.check_segment_placement(5, 2));
        assert!(masks.check_segment_placement(4, 3));
        assert!(masks.check_segment_placement(2, 1));
        assert!(masks.check_segment_placement(1, 2));
        assert!(!masks.check_segment_placement(1, 3));
        assert!(masks.check_segment_placement(0, 1));
        assert!(masks.check_segment_placement(0, 3));
        assert!(!masks.check_segment_placement(0, 4));
    }

    #[test]
    fn test_part_one() {
        let result =
            part_one(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(21));
    }

    #[test]
    fn test_part_two() {
        let result =
            part_two(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(525152));
    }
}

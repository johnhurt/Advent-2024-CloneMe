use std::collections::{BTreeMap, HashSet};

use advent_of_code::{ws, Compass};
use itertools::Itertools;
use nom::{
    bytes::complete::tag,
    character::complete::anychar,
    character::complete::hex_digit1,
    character::complete::u32,
    combinator::map,
    sequence::{delimited, tuple},
    IResult,
};
use tinyvec::TinyVec;

advent_of_code::solution!(18);

// This solution is horribly complicated. I don't know how you were supposed to
// do this one, but this probably wasn't it. I derived a way to accumulate area
// based on a stack of steps by taking into account whether a set of turns is
// going clockwise or counter clockwise, whether an area contains other segments
// of the trench. It's like finding the area of a shape defined in vectors.
// Luckily there are no overlaps in the trench. There are comments below, but
// good luck understanding any of this :(

type TV3<T> = TinyVec<[T; 3]>;

struct Dig {
    dir: Compass,
    steps: u32,
    rgb: u32,
}

impl Dig {
    fn parse(line: &str) -> Self {
        let result: IResult<_, _> = tuple((
            map(anychar, Compass::from_relative),
            ws(u32),
            delimited(tag("(#"), hex_digit1, tag(")")),
        ))(line);

        let (dir, steps, color_digits) = result.unwrap().1;

        let rgb = u32::from_str_radix(color_digits, 16).unwrap();

        Self {
            dir: dir.unwrap(),
            steps,
            rgb,
        }
    }

    /// This is for part 2. It turns the parsed color into a new instructions
    /// which is much bigger X(
    fn into_rgb(self) -> Self {
        use Compass as D;

        let dir = match self.rgb % 4 {
            0 => D::E,
            1 => D::S,
            2 => D::W,
            3 => D::N,
            _ => unreachable!(),
        };

        Self {
            dir,
            steps: self.rgb >> 4,
            rgb: 0,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum ClockDir {
    #[default]
    Clockwise,
    Counterclockwise,
}

type Point = (i64, i64);

// Represents a captured starting point from executing all the digging
// instructions
#[derive(Debug, Default, Clone, Copy)]
struct Step {
    start: Point,
    dir: Compass,
    count: i64,
}

impl Step {
    // Get the point where this instruction will end
    fn end(&self) -> Point {
        use Compass as D;
        let mut curr = self.start;
        match self.dir {
            D::E => curr.0 += self.count,
            D::S => curr.1 += self.count,
            D::W => curr.0 -= self.count,
            D::N => curr.1 -= self.count,
        };

        curr
    }

    // Collapse this point with another point. If they point in the same,
    // the result is a longer combined step. If they point in opposite
    // directions, the result is shorter with some consumed area returned
    fn collapse(self, after: Self) -> (i64, Self) {
        let Step {
            start,
            dir: d1,
            count: n1,
        } = self;

        let Step {
            dir: d2, count: n2, ..
        } = after;

        let (area, dir, count) = if d1 == d2 {
            (0, d1, n1 + n2)
        } else if n1 > n2 {
            (n2, d1, n1 - n2)
        } else {
            (n1, d2, n2 - n1)
        };

        (area, Self { start, dir, count })
    }
}

/// This whole algorithm works by examining the top 3 steps on the stack and
/// determining if they can capture area or be collapsed
#[derive(Debug, Clone, Copy)]
struct ThreeSteps(Step, Step, Step);

impl ThreeSteps {
    // Determine if this set of points defines a captured rectangle that
    // contains any points in the given map
    fn contains_points_in(
        &self,
        y_by_x: &BTreeMap<i64, HashSet<Point>>,
    ) -> bool {
        let (x0, y0) = self.0.start;
        let (x1, y1) = self.0.end();
        let (x2, y2) = self.1.end();
        let (x3, y3) = self.2.end();

        let (min_x, max_x, min_y, max_y) = if x1 == x2 {
            // Partial rect open on the side
            if x1 > x0 {
                // opens left
                (x0.max(x3), x1, y1.min(y2), y1.max(y2))
            } else {
                // opens right
                (x1, x0.min(x3), y1.min(y2), y1.max(y2))
            }
        } else if y1 == y2 {
            // partial rect open on top or bottom
            if y1 > y0 {
                // opens up
                (x1.min(x2), x1.max(x2), y0.max(y3), y1)
            } else {
                // opens down
                (x1.min(x2), x1.max(x2), y1, y0.min(y3))
            }
        } else {
            unreachable!()
        };

        let x_range = (min_x)..=max_x;
        let y_range = (min_y)..=max_y;
        y_by_x
            .range(x_range)
            .flat_map(|(_, p)| p.iter())
            .any(|(_, y)| y_range.contains(y))
    }

    // Determine the area captured by this triple and resulting simplification
    // of digging steps that results from removing the area
    fn area(self) -> (i64, TV3<Step>) {
        let ThreeSteps(
            Step {
                dir: d1,
                count: n1,
                start,
            },
            Step {
                dir: d2, count: n2, ..
            },
            Step {
                dir: d3, count: n3, ..
            },
        ) = self;

        let diff = n1.min(n3);
        let area = (n2 + 1) * diff;
        let mut curr = start;

        let next = [(d1, n1 - n3), (d2, n2), (d3, n3 - n1)]
            .into_iter()
            .filter(|(_, n)| *n > 0)
            .map(|(dir, count)| {
                let result = Step {
                    dir,
                    count,
                    start: curr,
                };
                curr = result.end();
                result
            })
            .collect::<TV3<_>>();

        (area, next)
    }

    fn first_2_collapsable(&self) -> bool {
        self.0.dir == self.1.dir || self.0.dir == self.1.dir.opposite()
    }

    fn second_2_collapsable(&self) -> bool {
        self.2.dir == self.1.dir || self.2.dir == self.1.dir.opposite()
    }

    fn collapse_first_2(self) -> (i64, TV3<Step>) {
        let (area, collapsed) = self.0.collapse(self.1);
        let mut remain = self.2;
        remain.start = collapsed.end();

        (
            area,
            [collapsed, remain]
                .into_iter()
                .filter(|s| s.count > 0)
                .collect::<TV3<_>>(),
        )
    }

    fn collapse_second_2(self) -> (i64, TV3<Step>) {
        let (area, collapsed) = self.1.collapse(self.2);
        let remain = self.0;

        (
            area,
            [remain, collapsed]
                .into_iter()
                .filter(|s| s.count > 0)
                .collect::<TV3<_>>(),
        )
    }

    // Determine if this triple wraps around some area in a clockwise or
    // counterclockwise direction
    fn clock_dir(&self) -> Option<ClockDir> {
        use Compass as D;

        match (self.0.dir, self.1.dir, self.2.dir) {
            (D::N, D::E, D::S)
            | (D::E, D::S, D::W)
            | (D::S, D::W, D::N)
            | (D::W, D::N, D::E) => Some(ClockDir::Clockwise),
            (D::N, D::W, D::S)
            | (D::E, D::N, D::W)
            | (D::S, D::E, D::N)
            | (D::W, D::S, D::E) => Some(ClockDir::Counterclockwise),
            _ => None,
        }
    }

    fn collect(self) -> TV3<Step> {
        [self.0, self.1, self.2].into_iter().collect::<TV3<_>>()
    }

    /// Get the points associated with the triple. This includes the point
    /// after the last instruction is applied
    fn points(self) -> Vec<Point> {
        vec![self.0.start, self.1.start, self.2.start, self.2.end()]
    }
}

/// This is the tool for capturing area step by step in the instructions
#[derive(Debug, Default)]
struct AreaAccumulator {
    stack: Vec<Step>,
    points_by_x: BTreeMap<i64, HashSet<Point>>,
    total: i64,
    clock_dir: ClockDir,
}

impl AreaAccumulator {
    /// After each new step, we take the top 3 steps and determine if they
    /// cut off area without interference or if they can be simplified. If
    /// Neither are true, we keep consuming
    fn fold(mut self, step: Step) -> Self {
        // Consume until we have at least 3 steps
        if self.stack.len() < 2 {
            self.stack.push(step);
            return self;
        }

        let last = self.stack.pop().unwrap();
        let almost_last = self.stack.pop().unwrap();

        let top_3 = ThreeSteps(almost_last, last, step);
        let mut changed = true;

        // Remove the points we are looking at from the map of points. We don't
        // want them to interfere with themselves during the interference check
        top_3.points().iter().for_each(|(x, y)| {
            self.points_by_x.entry(*x).and_modify(|v| {
                v.remove(&(*x, *y));
            });
        });

        let (area, replacements) = if top_3.clock_dir() == Some(self.clock_dir)
            && !top_3.contains_points_in(&self.points_by_x)
        {
            // Count the area only if we are capturing in the right direction
            // and there are no points interfering with the rectangle defined
            // by our 3 points
            top_3.area()
        } else if top_3.first_2_collapsable() {
            top_3.collapse_first_2()
        } else if top_3.second_2_collapsable() {
            top_3.collapse_second_2()
        } else {
            changed = false;
            (0, top_3.collect())
        };

        self.total += area;
        self.stack.extend(replacements.into_iter());

        // Make a recursive call to the fold function until nothing changes
        if changed && !self.stack.is_empty() {
            let next = self.stack.pop().unwrap();
            self.fold(next)
        } else {
            self
        }
    }

    /// The folding process will end before the stack is empty. There should
    /// only be 2 entries left on the stack that point in opposite directions.
    /// They need to be collapsed and their area counted
    fn finish(&mut self) -> bool {
        if self.stack.len() != 2 {
            return false;
        }

        let (area, _) = self
            .stack
            .pop()
            .unwrap()
            .collapse(self.stack.pop().unwrap());
        self.total += area;

        true
    }
}

/// Calculate the area defined in terms of vector operations
fn find_vector_area<I>(digs: I) -> i64
where
    I: Iterator<Item = Dig>,
{
    let mut accumulator = AreaAccumulator::default();

    let mut curr = (0, 0);

    // Play out all the digging instructions starting at 0, 0. Record the
    // position at the beginning of each step
    let steps = digs
        .map(|d| (d.dir, d.steps as i64))
        .map(|(dir, count)| {
            let result = Step {
                dir,
                count,
                start: curr,
            };

            accumulator
                .points_by_x
                .entry(curr.0)
                .and_modify(|v| {
                    v.insert(curr);
                })
                .or_insert_with(|| [curr].into_iter().collect::<HashSet<_>>());

            curr = result.end();

            result
        })
        .collect_vec();

    // We don't know ahead of time if the loop we are making is clockwise or
    // counterclockwise, but we can check by counting all the turns in the path
    let mut cw_count = 0;
    let mut ccw_count = 0;

    steps
        .iter()
        .copied()
        .tuple_windows()
        .map(|(s1, s2, s3)| ThreeSteps(s1, s2, s3))
        .filter_map(|ts| ts.clock_dir())
        .for_each(|cd| match cd {
            ClockDir::Clockwise => cw_count += 1,
            ClockDir::Counterclockwise => ccw_count += 1,
        });

    // if there's more counterclockwise turns, then we need to accumulate only
    // counterclockwise area ... duh :-/
    if ccw_count > cw_count {
        accumulator.clock_dir = ClockDir::Counterclockwise;
    }

    let mut accumulator =
        steps.into_iter().fold(accumulator, AreaAccumulator::fold);

    debug_assert!(accumulator.finish());

    // Plus 1 here because the starting point at 0, 0 can't be captured by
    // any other dig step
    accumulator.total + 1
}

pub fn part_one(input: &str) -> Option<i64> {
    let result = find_vector_area(input.lines().map(Dig::parse));

    Some(result)
}

pub fn part_two(input: &str) -> Option<i64> {
    let result =
        find_vector_area(input.lines().map(Dig::parse).map(Dig::into_rgb));

    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let result =
            part_one(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(62));
    }

    #[test]
    fn test_part_two() {
        let result =
            part_two(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(952408144115));
    }
}

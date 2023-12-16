use std::collections::HashSet;

use advent_of_code::{Compass, Grid};
use itertools::Itertools;
use strum_macros::FromRepr;

advent_of_code::solution!(16);

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromRepr)]
enum Mirror {
    None = b'.',
    Forward = b'/',
    Back = b'\\',
    Horizontal = b'-',
    Vertical = b'|',
}

impl From<char> for Mirror {
    fn from(value: char) -> Self {
        Mirror::from_repr(value as u8).unwrap()
    }
}

impl From<Mirror> for char {
    fn from(value: Mirror) -> Self {
        value as u8 as char
    }
}

fn trace_paths(start: (usize, Compass), grid: &Grid<Mirror>) -> HashSet<usize> {
    let mut search_heads = vec![start];
    let mut seen_starts = HashSet::new();

    let mut energized = HashSet::new();

    while let Some((mut curr, mut dir)) = search_heads.pop() {
        let mut seen = HashSet::new();
        if !seen_starts.insert((curr, dir)) {
            continue;
        }

        energized.insert(curr);
        loop {
            use Compass as D;
            use Mirror as M;

            if !seen.insert((curr, dir)) {
                break;
            }
            energized.insert(curr);

            let mirror = grid.at_index(curr).unwrap();

            dir = match (dir, mirror) {
                (D::E, M::Forward) => D::N,
                (D::E, M::Back) => D::S,
                (D::E, M::Vertical) => {
                    search_heads.push((curr, D::N));
                    D::S
                }
                (D::W, M::Forward) => D::S,
                (D::W, M::Back) => D::N,
                (D::W, M::Vertical) => {
                    search_heads.push((curr, D::N));
                    D::S
                }
                (D::N, M::Forward) => D::E,
                (D::N, M::Back) => D::W,
                (D::N, M::Horizontal) => {
                    search_heads.push((curr, D::E));
                    D::W
                }
                (D::S, M::Forward) => D::W,
                (D::S, M::Back) => D::E,
                (D::S, M::Horizontal) => {
                    search_heads.push((curr, D::E));
                    D::W
                }
                _ => dir,
            };

            match grid.step_from_index(curr, dir) {
                Some(next) => curr = next,
                _ => break,
            }
        }
    }

    energized
}

pub fn part_one(input: &str) -> Option<usize> {
    let grid = Grid::<Mirror>::parse_lines(input);
    Some(trace_paths((0, Compass::E), &grid).len())
}

pub fn part_two(input: &str) -> Option<usize> {
    let grid = Grid::<Mirror>::parse_lines(input);
    let starts =
        Some(Compass::S)
            .iter()
            .cartesian_product(0..grid.width)
            .chain(Some(Compass::N).iter().cartesian_product(
                (grid.data.len() - grid.width)..grid.data.len(),
            ))
            .chain(
                Some(Compass::E).iter().cartesian_product(
                    (0..grid.height).map(|x| x * grid.width),
                ),
            )
            .chain(Some(Compass::W).iter().cartesian_product(
                (0..grid.height).map(|x| x * grid.width + (grid.width - 1)),
            ))
            .collect_vec();

    let result = starts
        .into_iter()
        .map(|(d, s)| trace_paths((s, *d), &grid).len())
        .max()
        .unwrap();

    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let result =
            part_one(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(46));
    }

    #[test]
    fn test_part_two() {
        let result =
            part_two(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(51));
    }
}

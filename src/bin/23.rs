use std::collections::{hash_map::Entry, BinaryHeap, HashMap, HashSet};

use advent_of_code::{Compass, Grid};

advent_of_code::solution!(23);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Hike {
    None,
    Wall,
    Walked,
    Slope(Compass),
}

impl From<char> for Hike {
    fn from(value: char) -> Self {
        match value {
            '#' => Hike::Wall,
            '.' => Hike::None,
            '>' => Hike::Slope(Compass::E),
            '<' => Hike::Slope(Compass::W),
            '^' => Hike::Slope(Compass::N),
            'v' => Hike::Slope(Compass::S),
            _ => unreachable!(),
        }
    }
}

impl From<Hike> for char {
    fn from(value: Hike) -> Self {
        match value {
            Hike::None => '.',
            Hike::Wall => '#',
            Hike::Walked => 'O',
            Hike::Slope(Compass::E) => '>',
            Hike::Slope(Compass::N) => '^',
            Hike::Slope(Compass::W) => '<',
            Hike::Slope(Compass::S) => 'v',
        }
    }
}

// ub 2028
// lb 2006
// g  2010

fn print(grid: &Grid<Hike>, walked: &HashSet<usize>) {
    let mut g = grid.clone();

    walked
        .iter()
        .copied()
        //.filter(|c| !matches!(grid.data[*c], Hike::Slope(_)))
        .for_each(|c| g.data[c] = Hike::Walked);

    g.print();
}

fn max_path(start: usize, end: usize, grid: &Grid<Hike>) -> usize {
    let mut stack = vec![];
    stack.push((0, start, HashSet::new()));

    let mut max_dist = 0;

    while let Some((mut dist, mut curr, mut hist)) = stack.pop() {
        if curr == end {
            if dist > max_dist {
                max_dist = max_dist.max(dist);
                // print(grid, &hist);
                // println!("{dist} ^");
            }
            continue;
        }

        if !hist.insert(curr) {
            continue;
        }

        match grid.data[curr] {
            Hike::Slope(Compass::N) => {
                curr -= grid.width;
                dist += 1;
                hist.insert(curr);
            }
            Hike::Slope(Compass::S) => {
                curr += grid.width;
                dist += 1;
                hist.insert(curr);
            }
            Hike::Slope(Compass::E) => {
                curr += 1;
                dist += 1;
                hist.insert(curr);
            }
            Hike::Slope(Compass::W) => {
                curr -= 1;
                dist += 1;
                hist.insert(curr);
            }
            _ => {}
        }

        stack.extend(
            grid.neighbors(curr)
                .filter(|(_, n)| grid.data[*n] != Hike::Wall)
                .filter(|(_, n)| !hist.contains(n))
                .filter(|(d, n)| Hike::Slope(d.opposite()) != grid.data[*n])
                .map(|(_, n)| (dist + 1, n, hist.clone())),
        );
    }

    max_dist
}

pub fn part_one(input: &str) -> Option<usize> {
    let grid = Grid::parse_lines(input);

    let result = max_path(1, grid.data.len() - 2, &grid);

    Some(result)
}

pub fn part_two(input: &str) -> Option<usize> {
    let mut grid = Grid::parse_lines(input);

    grid.data.iter_mut().for_each(|h| {
        if matches!(h, Hike::Slope(_)) {
            *h = Hike::None;
        }
    });

    let result = max_path(1, grid.data.len() - 2, &grid);

    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let result =
            part_one(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(94));
    }

    #[test]
    fn test_part_two() {
        let result =
            part_two(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(154));
    }
}

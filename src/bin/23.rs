use std::collections::{HashMap, HashSet, VecDeque};

use advent_of_code::{Compass, Grid};

advent_of_code::solution!(23);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Hike {
    None,
    Wall,
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
            Hike::Slope(Compass::E) => '>',
            Hike::Slope(Compass::N) => '^',
            Hike::Slope(Compass::W) => '<',
            Hike::Slope(Compass::S) => 'v',
        }
    }
}

fn max_path(start: usize, end: usize, grid: &mut Grid<Hike>) -> usize {
    let mut stack = vec![];
    stack.push((0, start, HashSet::new()));

    let mut max_dist = 0;

    while let Some((mut dist, mut curr, mut hist)) = stack.pop() {
        if curr == end {
            if dist > max_dist {
                max_dist = max_dist.max(dist);
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
    let mut grid = Grid::parse_lines(input);

    let result = max_path(1, grid.data.len() - 2, &mut grid);

    Some(result)
}

fn graph_neighbors(
    from: usize,
    grid: &Grid<Hike>,
    nodes: &HashSet<usize>,
) -> HashMap<usize, usize> {
    let mut result = HashMap::new();
    let mut queue = VecDeque::new();

    queue.push_back((from, 0, from));

    while let Some((curr, dist, prev)) = queue.pop_front() {
        if curr != from && nodes.contains(&curr) {
            result.insert(curr, dist);
        } else {
            queue.extend(
                grid.neighbors(curr)
                    .map(|(_, n)| n)
                    .filter(|n| grid.data[*n] != Hike::Wall)
                    .filter(|n| *n != prev)
                    .map(|n| (n, dist + 1, curr)),
            )
        }
    }

    result
}

fn extract_graph(
    grid: &Grid<Hike>,
    start: usize,
    end: usize,
) -> HashMap<usize, HashMap<usize, usize>> {
    let nodes = [start, end]
        .into_iter()
        .chain(
            grid.data
                .iter()
                .enumerate()
                .filter(|(_, h)| **h != Hike::Wall)
                .map(|(i, _)| i)
                .filter(|i| {
                    grid.neighbors(*i)
                        .map(|(_, n)| n)
                        .filter(|n| grid.data[*n] != Hike::Wall)
                        .count()
                        > 2
                }),
        )
        .collect::<HashSet<_>>();

    nodes
        .iter()
        .map(|n| (*n, graph_neighbors(*n, grid, &nodes)))
        .collect::<HashMap<_, _>>()
}

fn search_simplified(start: usize, end: usize, grid: &mut Grid<Hike>) -> usize {
    let simplified = extract_graph(grid, start, end);

    let mut stack = vec![];
    stack.push((0, start, HashSet::new()));

    let mut max_dist = 0;

    while let Some((dist, curr, mut hist)) = stack.pop() {
        if curr == end {
            if dist > max_dist {
                max_dist = max_dist.max(dist);
            }
            continue;
        }

        if !hist.insert(curr) {
            continue;
        }

        stack.extend(
            simplified
                .get(&curr)
                .unwrap()
                .iter()
                .map(|(too, add_dist)| (dist + add_dist, *too, hist.clone())),
        );
    }

    max_dist
}

pub fn part_two(input: &str) -> Option<usize> {
    let mut grid = Grid::parse_lines(input);

    grid.data.iter_mut().for_each(|h| {
        if matches!(h, Hike::Slope(_)) {
            *h = Hike::None;
        }
    });

    let result = search_simplified(1, grid.data.len() - 2, &mut grid);

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

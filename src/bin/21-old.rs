use std::collections::{HashMap, HashSet, VecDeque};

use advent_of_code::{Compass, Grid};
use itertools::Itertools;

advent_of_code::solution!(21);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum Plot {
    #[default]
    None,
    Rock,
    Start,
}

impl From<char> for Plot {
    fn from(value: char) -> Self {
        match value {
            '.' => Plot::None,
            '#' => Plot::Rock,
            'S' => Plot::Start,
            _ => unreachable!(),
        }
    }
}

fn possible_endpoints_impl(
    start: usize,
    grid: &Grid<Plot>,
    steps: usize,
) -> (usize, HashMap<usize, usize>) {
    let mut queue = VecDeque::new();
    queue.push_back((start, 0));

    let mut searched = HashSet::new();
    let mut result = HashSet::new();
    let mut escaped = HashMap::new();

    while let Some((curr, dist)) = queue.pop_front() {
        if !searched.insert((curr, dist)) {
            continue;
        }

        if dist == steps {
            result.insert(curr);
            continue;
        }

        grid.escaping(curr)
            .map(|dir| {
                let (dx, dy) = match dir {
                    Compass::E => (1, 0),
                    Compass::S => (0, 1),
                    Compass::W => (-1, 0),
                    Compass::N => (0, -1),
                };
                let (x, y) = (curr % grid.width, curr / grid.width);
                let xp = (x as i64 + dx).rem_euclid(grid.width as i64);
                let yp = (y as i64 + dy).rem_euclid(grid.height as i64);
                let ep = xp as usize + yp as usize * grid.width;
                (ep, dist + 1)
            })
            .for_each(|(p, ep)| {
                escaped.entry(p).or_insert(ep);
            });

        queue.extend(
            grid.neighbors(curr)
                .filter(|(_, n)| grid.data[*n] != Plot::Rock)
                .map(|(_, n)| (n, dist + 1)),
        );
    }

    (result.len(), escaped)
}

fn possible_endpoints(input: &str, steps: usize) -> usize {
    let mut grid = Grid::<Plot>::parse_lines(input);
    let start = grid
        .data
        .iter()
        .enumerate()
        .find(|(_, p)| **p == Plot::Start)
        .unwrap()
        .0;

    grid.data[start] = Plot::None;

    possible_endpoints_impl(start, &grid, steps).0
}

pub fn part_one(input: &str) -> Option<usize> {
    let result = possible_endpoints(input, 64);
    Some(result)
}

type Point = (i64, i64);
struct RepeatingGrid(Grid<Plot>);

impl RepeatingGrid {
    fn valid_neighbors(
        &self,
        (x, y): Point,
    ) -> impl Iterator<Item = Point> + '_ {
        [(-1, 0), (1, 0), (0, 1), (0, -1)]
            .into_iter()
            .map(move |(dx, dy)| (x + dx, y + dy))
            .filter(|(xx, yy)| {
                let xp = xx.rem_euclid(self.0.width as i64);
                let yp = yy.rem_euclid(self.0.height as i64);
                let c = xp as usize + yp as usize * self.0.width;
                self.0.data[c] != Plot::Rock
            })
    }
}

fn run_to_saturation(
    start: usize,
    grid: &Grid<Plot>,
) -> HashMap<usize, (usize, HashMap<usize, usize>)> {
    let mut seen_escapees: HashSet<usize> = HashSet::new();
    let mut steps = 0;
    let mut result = HashMap::new();
    let mut prev_escapees_size = 0;

    while prev_escapees_size == 0 || seen_escapees.len() != prev_escapees_size {
        if steps > 10 {
            println!("Hi -> {steps} {}", seen_escapees.len());
        }

        let ep_result = possible_endpoints_impl(start, grid, steps);

        seen_escapees.extend(ep_result.1.keys());

        prev_escapees_size = seen_escapees.len();

        result.insert(steps, ep_result);
        steps += 2;
    }

    result
}

fn possible_endpoints_inf(input: &str, steps: usize) -> usize {
    let mut grid = Grid::<Plot>::parse_lines(input);
    let start = grid
        .data
        .iter()
        .enumerate()
        .find(|(_, p)| **p == Plot::Start)
        .unwrap()
        .0;

    grid.data[start] = Plot::None;

    let start_points = Some(start)
        .into_iter()
        .chain(0..grid.width)
        .chain((grid.data.len() - grid.width)..grid.data.len())
        .chain((0..grid.height).map(|i| i * grid.width))
        .chain((0..grid.height).map(|i| (i + 1) * grid.width - 1))
        .collect_vec();

    let mut saturation = HashMap::new();

    let cache = start_points
        .iter()
        .copied()
        .flat_map(|start| {
            let sat_result = run_to_saturation(start, &grid);

            let saturation_point = sat_result.keys().max().unwrap();

            saturation.insert(start, *saturation_point);

            sat_result.into_iter().map(move |(steps, cache_result)| {
                ((start, steps), cache_result)
            })
        })
        .collect::<HashMap<_, _>>();

    println!("{}", cache.len());

    0
}

fn possible_endpoints_inf_brute(input: &str, steps: usize) -> usize {
    let mut grid = Grid::<Plot>::parse_lines(input);
    let start = grid
        .data
        .iter()
        .enumerate()
        .find(|(_, p)| **p == Plot::Start)
        .unwrap()
        .0;

    grid.data[start] = Plot::None;

    let start = ((start / grid.width) as i64, (start % grid.width) as i64);
    let grid = RepeatingGrid(grid);

    let mut queue = VecDeque::new();
    queue.push_back((start, 0));

    let mut searched = HashSet::new();
    let mut result = HashSet::new();

    while let Some((curr, dist)) = queue.pop_front() {
        if !searched.insert((curr, dist)) {
            continue;
        }

        if dist == steps {
            result.insert(curr);
            continue;
        }

        queue.extend(grid.valid_neighbors(curr).map(|n| (n, dist + 1)));
    }

    result.len()
}

pub fn part_two(input: &str) -> Option<usize> {
    Some(possible_endpoints_inf(input, 26501365))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one_1() {
        let result = possible_endpoints(
            &advent_of_code::template::read_file("examples", DAY),
            1,
        );
        assert_eq!(result, 2);
    }

    #[test]
    fn test_part_one_2() {
        let result = possible_endpoints(
            &advent_of_code::template::read_file("examples", DAY),
            2,
        );
        assert_eq!(result, 4);
    }

    #[test]
    fn test_part_one_6() {
        let result = possible_endpoints(
            &advent_of_code::template::read_file("examples", DAY),
            6,
        );
        assert_eq!(result, 16);
    }

    #[test]
    fn test_part_two_6() {
        let result = possible_endpoints_inf_brute(
            &advent_of_code::template::read_file("examples", DAY),
            6,
        );
        assert_eq!(result, 16);
    }

    #[test]
    fn test_part_two_50() {
        let result = possible_endpoints_inf(
            &advent_of_code::template::read_file("examples", DAY),
            50,
        );
        assert_eq!(result, 1594);
    }
}

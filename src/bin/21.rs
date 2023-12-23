use std::collections::{
    hash_map::Entry, BTreeMap, BinaryHeap, HashMap, VecDeque,
};

use advent_of_code::Grid;
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

fn all_paths_from(
    start: usize,
    grid: &Grid<Plot>,
    steps: usize,
) -> HashMap<usize, usize> {
    let mut queue = VecDeque::new();
    queue.push_back((start, 0));

    let mut result = HashMap::new();

    while let Some((curr, dist)) = queue.pop_front() {
        if let Entry::Vacant(e) = result.entry(curr) {
            e.insert(dist);
        } else {
            continue;
        }

        if dist == steps {
            continue;
        }

        queue.extend(
            grid.neighbors(curr)
                .filter(|(_, n)| grid.data[*n] != Plot::Rock)
                .filter(|(_, n)| !result.contains_key(n))
                .map(|(_, n)| (n, dist + 1)),
        );
    }

    result
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

    let mod_2 = steps % 2;

    all_paths_from(start, &grid, steps)
        .iter()
        .filter(|(_, d)| *d % 2 == mod_2)
        .count()
}

pub fn part_one(input: &str) -> Option<usize> {
    let result = possible_endpoints(input, 64);
    Some(result)
}

type Point = (i32, i32);
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
                // if (*xx, *yy) == (8, 14) {
                //     println!("asdf");
                // }

                let xp = xx.rem_euclid(self.0.width as i32);
                let yp = yy.rem_euclid(self.0.height as i32);
                let c = xp as usize + yp as usize * self.0.width;
                self.0.data[c] != Plot::Rock
            })
    }
}

fn all_shortest_paths_from(
    grid: &RepeatingGrid,
    max_steps: usize,
    mut queue: BinaryHeap<(i32, Point, usize)>,
    mut result: HashMap<Point, usize>,
) -> HashMap<usize, usize> {
    let mut round = 0;

    let mut count_at_dist = HashMap::new();
    let mut prev_dist = 0;
    let removal_buffer_size = 1;

    while let Some((_, curr, dist)) = queue.pop() {
        round += 1;

        if dist > prev_dist && prev_dist != 0 {
            let min_dist = prev_dist + 1 - removal_buffer_size;
            let mut removed = 0;
            let dist_removed = prev_dist - removal_buffer_size;

            result.retain(|(x, y), dist| {
                if *dist > 0 && *dist < min_dist {
                    removed += 1;
                }

                *dist >= min_dist || *dist == 0
            });

            count_at_dist.insert(dist_removed, removed);
        }

        if dist < prev_dist {
            panic!();
        }

        prev_dist = prev_dist.max(dist);

        // if curr == (6, 14) {
        //     println!("asdf");
        // }

        if dist > max_steps {
            continue;
        }

        if round % 10000000 == 0 {
            let remaining_dist = max_steps - dist;
            println!(
                "{remaining_dist} | {round} -> result = {} | queue = {}",
                result.len(),
                queue.len()
            );
        }

        if let Entry::Vacant(e) = result.entry(curr) {
            e.insert(dist);
        } else {
            continue;
        }

        if dist == max_steps {
            continue;
        }

        // let i = curr.0 + curr.1 * grid.0.width as i32;
        // if curr.0 >= 0
        //     && curr.1 >= 0
        //     && curr.1 < 11
        //     && curr.0 < 11
        //     && i < grid.0.data.len() as i32
        // {
        //     println!("WAT");
        // }

        // if curr == (0, 0) {
        //     println!("Ugh");
        // }

        queue.extend(grid.valid_neighbors(curr).filter_map(|p| {
            // if p == (8, 14) {
            //     println!("asdf");
            // }

            if result.contains_key(&p) {
                None
            } else {
                Some((-(dist as i32 + 1), p, dist + 1))
            }
        }));
    }

    for i in (max_steps - removal_buffer_size)..=max_steps {
        let count_at_i = result.values().filter(|d| **d == i).count();
        count_at_dist.insert(i, count_at_i);
    }

    count_at_dist
}

fn boundary_of_diamond(
    center: Point,
    radius_tiles: i32,
    tile_width: i32,
    interior: bool,
) -> Vec<Point> {
    let mut result = vec![];

    let mut curr = (
        -tile_width / 2 - 1,
        -radius_tiles * tile_width - tile_width / 2,
    );
    let horiz_step = tile_width;
    let vert_step = tile_width;

    if !interior {
        curr.1 -= 1;
    }

    for _ in 0..horiz_step {
        curr.0 += 1;
        result.push(curr);
    }

    if !interior {
        curr.0 += 1;
        result.push(curr);
    }

    for _ in 0..radius_tiles {
        for _ in 0..vert_step {
            curr.1 += 1;
            result.push(curr);
        }

        for _ in 0..horiz_step {
            curr.0 += 1;
            result.push(curr);
        }
    }

    result.extend(
        result
            .clone()
            .into_iter()
            .flat_map(|(x, y)| [(-x, -y), (y, -x), (-y, x)].into_iter()),
    );

    result.iter_mut().for_each(|(x, y)| {
        *x += center.0;
        *y += center.1
    });

    result
}

fn diamond_tiles(radius: i32) -> (usize, usize) {
    let (mut like, mut unlike) = (1, 0);

    for r in 1..(radius.clamp(0, i32::MAX) as usize + 1) {
        let added = (2 * r + 1) * 2 - 2;
        if r % 2 == 0 {
            like += added;
        } else {
            unlike += added;
        }
    }

    (like, unlike)
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

    let all_paths_tile = all_paths_from(start, &grid, steps);

    let mod_2 = steps % 2;

    let center_fill = all_paths_tile
        .iter()
        .filter(|(_, d)| *d % 2 == mod_2)
        .count();

    let adj_fill = all_paths_tile
        .iter()
        .filter(|(_, d)| *d % 2 != mod_2)
        .count();

    let width = grid.width as i32;
    let steps_to_fill = all_paths_tile.values().copied().max().unwrap() as i32;
    let radius_steps = steps as i32 - width / 2;
    let mut radius_tiles = radius_steps / width;

    while radius_steps - radius_tiles * width < steps_to_fill {
        radius_tiles -= 1;
    }

    // radius_tiles -= 1;
    radius_tiles = radius_tiles.clamp(0, i32::MAX);

    let start_point = grid.to_col_row(start);
    let interior_boundary =
        boundary_of_diamond(start_point, radius_tiles, width, true);
    let exterior_boundary =
        boundary_of_diamond(start_point, radius_tiles, width, false);

    // interior_boundary.iter().for_each(|(x, y)| {
    //     println!("{x}, {y}");
    // });

    // println!();

    // exterior_boundary.iter().for_each(|(x, y)| {
    //     println!("{x}, {y}");
    // });

    let result = interior_boundary
        .into_iter()
        .map(|p| (p, 0))
        .collect::<HashMap<_, _>>();

    let queue = exterior_boundary
        .into_iter()
        .map(|(px, py)| {
            let dist = (start_point.0 - px).abs() + (start_point.1 - py).abs();
            (-dist, (px, py), dist as usize)
        })
        .collect::<BinaryHeap<_>>();

    let res =
        all_shortest_paths_from(&RepeatingGrid(grid), steps, queue, result);

    //res.keys().for_each(|(x, y)| println!("{x}, {y}"));

    let mod_2 = steps % 2;

    let border_res = res
        .iter()
        .filter_map(|(d, c)| (*d != 0 && *d % 2 == mod_2).then_some(c))
        //.map(|((x, y), _)| println!("{x}, {y}"))
        .sum::<usize>();

    // res.iter()
    //     .collect::<BTreeMap<_, _>>()
    //     .into_iter()
    //     .for_each(|(p, d)| println!("{:?} -> {d}", p));

    let (like_center_count, unlike_count) = diamond_tiles(radius_tiles);

    let interior_res =
        like_center_count * center_fill + unlike_count * adj_fill;

    interior_res + border_res
}

pub fn part_two(input: &str) -> Option<usize> {
    Some(possible_endpoints_inf(input, 26501365))
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, HashSet};

    use super::*;

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

        let start = ((start / grid.width) as i32, (start % grid.width) as i32);
        let grid = RepeatingGrid(grid);

        let mut queue = VecDeque::new();
        queue.push_back((start, 0));

        let mut result = HashMap::new();

        while let Some((curr, dist)) = queue.pop_front() {
            if let Entry::Vacant(e) = result.entry(curr) {
                e.insert(dist);
            } else {
                continue;
            }

            if dist == steps {
                continue;
            }

            queue.extend(grid.valid_neighbors(curr).map(|n| (n, dist + 1)));
        }

        let mod_2 = steps % 2;

        // result
        //     .iter()
        //     .copied()
        //     .filter(|(x, y)| !(*x >= 0 && *x < 11 && *y >= 0 && *y < 11))
        //     .for_each(|(x, y)| println!("{x}, {y}"));

        //println!("ins {ins} outs {}", result.len() - ins);

        // result
        //     .iter()
        //     .collect::<BTreeMap<_, _>>()
        //     .into_iter()
        //     .for_each(|(p, d)| println!("{:?} -> {d}", p));

        // result
        //     .iter()
        //     .filter(|((x, y), _)| !(*x >= 0 && *x < 11 && *y >= 0 && *y < 11))
        //     .map(|(k, v)| (v, k))
        //     .into_group_map()
        //     .into_iter()
        //     .collect::<BTreeMap<_, _>>()
        //     .into_iter()
        //     .for_each(|(d, v)| println!("{:?} -> {:?}", d, v.len()));

        result.iter().filter(|(_, d)| *d % 2 == mod_2).count()
    }

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
    fn test_part_one_10_alt() {
        let input = &advent_of_code::template::read_extra_example_file(DAY, 1);
        let result = possible_endpoints(&input, 10);
        assert_eq!(result, 47);
    }

    #[test]
    fn test_part_two_50() {
        let input = &advent_of_code::template::read_extra_example_file(DAY, 1);

        let steps = 5001;
        let brute_result = possible_endpoints_inf_brute(input, steps);

        let result = possible_endpoints_inf(input, steps);
        assert_eq!(result, brute_result);
    }
}

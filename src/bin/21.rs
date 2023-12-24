use std::collections::{hash_map::Entry, HashMap, VecDeque};

use advent_of_code::Grid;

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

fn interior_diamond_tiles(radius: i32) -> (usize, usize) {
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

fn exterior_diamond_tiles(radius: i32, width: i32) -> Vec<Point> {
    let mut top_left = (0, -radius * width);
    let mut result = vec![];

    if radius == 0 {
        return vec![top_left];
    }

    let dxdy = [
        (width, width),
        (-width, width),
        (-width, -width),
        (width, -width),
    ];

    for (dx, dy) in dxdy {
        for _ in 0..radius {
            result.push(top_left);

            top_left.0 += dx;
            top_left.1 += dy;
        }
    }

    result
}

fn min_dist_to_tile(
    (sx, sy): Point,
    (tx, ty): Point,
    grid: &Grid<Plot>,
) -> (usize, i32) {
    let width = grid.width as i32;

    let ((min_x, min_y), min_dist) = [
        (tx, ty),                         // TL
        (tx + width / 2, ty),             // TM
        (tx + width - 1, ty),             // TR
        (tx, ty + width / 2),             // ML
        (tx + width - 1, ty + width / 2), // MR
        (tx, ty + width - 1),             // BL,
        (tx + width / 2, ty + width - 1), // BM
        (tx + width - 1, ty + width - 1), // BR
    ]
    .into_iter()
    .fold(((0, 0), i32::MAX), |((mx, my), min), (px, py)| {
        let d = (sx - px).abs() + (sy - py).abs();
        if d < min {
            ((px, py), d)
        } else {
            ((mx, my), min)
        }
    });

    (
        grid.to_index(min_x.rem_euclid(width), min_y.rem_euclid(width)),
        min_dist,
    )
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
    let max_radius_tiles = radius_steps / width + 2;
    let mut radius_tiles = max_radius_tiles;

    while radius_steps - radius_tiles * width < steps_to_fill {
        radius_tiles -= 1;
    }

    radius_tiles = radius_tiles.clamp(0, i32::MAX);

    let start_point = grid.to_col_row(start);

    let south = grid.data.len() - grid.width + grid.width / 2;
    let north = grid.width / 2;
    let west = grid.height / 2 * grid.width;
    let east = west + grid.width - 1;

    let all_paths = [north, south, west, east]
        .into_iter()
        .chain(grid.corners())
        .map(|p| (p, all_paths_from(p, &grid, usize::MAX)))
        .collect::<HashMap<_, _>>();

    let mut max_paths = HashMap::new();

    let cached_results = [north, south, west, east]
        .into_iter()
        .chain(grid.corners())
        .map(|p| (p, all_paths.get(&p).unwrap()))
        .flat_map(|(p, all_paths_from_p)| {
            let max_d = all_paths_from_p.iter().map(|(_, d)| *d).max().unwrap();

            max_paths.insert(p, max_d);

            (1..=max_d).map(move |d| (p, all_paths_from_p, d))
        })
        .map(|(p, all_paths_from_p, d_max)| {
            let max_d_mod_2 = d_max % 2;

            let result = all_paths_from_p
                .iter()
                .filter(|(_, d)| **d <= d_max && **d % 2 == max_d_mod_2)
                .count();

            ((p, d_max), result)
        })
        .collect::<HashMap<_, _>>();

    let (like_center_count, unlike_count) =
        interior_diamond_tiles(radius_tiles);

    let interior_res =
        like_center_count * center_fill + unlike_count * adj_fill;

    let mut border_res = 0;

    for rad in (radius_tiles + 1)..=max_radius_tiles {
        border_res += exterior_diamond_tiles(rad, width)
            .into_iter()
            .map(|p| {
                let (t, min_d) = min_dist_to_tile(start_point, p, &grid);
                let steps_remain_opt = steps.checked_sub(min_d as usize);

                match steps_remain_opt {
                    None => 0,
                    Some(0) => 1,
                    Some(steps_remain) => {
                        let mut capped_steps_remain =
                            steps_remain.min(*max_paths.get(&t).unwrap());

                        if capped_steps_remain % 2 != steps_remain % 2 {
                            capped_steps_remain -= 1;
                        }

                        *cached_results.get(&(t, capped_steps_remain)).unwrap()
                    }
                }
            })
            .sum::<usize>();
    }

    interior_res + border_res
}

pub fn part_two(input: &str) -> Option<usize> {
    Some(possible_endpoints_inf(input, 26501365))
}

#[cfg(test)]
mod tests {

    use super::*;
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
                    let xp = xx.rem_euclid(self.0.width as i32);
                    let yp = yy.rem_euclid(self.0.height as i32);
                    let c = xp as usize + yp as usize * self.0.width;
                    self.0.data[c] != Plot::Rock
                })
        }
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
        let result = possible_endpoints(input, 10);
        assert_eq!(result, 47);
    }

    #[test]
    fn test_part_two_50() {
        let input = &advent_of_code::template::read_extra_example_file(DAY, 1);

        for i in 12..=100 {
            let steps = i;
            let brute_result = possible_endpoints_inf_brute(input, steps);

            let result = possible_endpoints_inf(input, steps);
            assert_eq!(result, brute_result);
        }
    }
}

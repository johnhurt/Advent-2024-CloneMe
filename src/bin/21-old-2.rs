use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};

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
    let mut borders = HashMap::new();

    while let Some((curr, dist)) = queue.pop_front() {
        if !searched.insert((curr, dist)) {
            continue;
        }

        if dist == steps {
            result.insert(curr);
            continue;
        }

        if grid.is_border(curr) {
            borders.entry(curr).or_insert(dist);
        }

        queue.extend(
            grid.neighbors(curr)
                .filter(|(_, n)| grid.data[*n] != Plot::Rock)
                .map(|(_, n)| (n, dist + 1)),
        );
    }

    (result.len(), borders)
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

fn all_shortest_paths_from(
    grid: &Grid<Plot>,
    from: usize,
) -> HashMap<usize, usize> {
    let mut queue = VecDeque::new();
    let mut result = HashMap::new();

    queue.push_back((from, 0));

    while let Some((curr, dist)) = queue.pop_front() {
        if let std::collections::hash_map::Entry::Vacant(e) = result.entry(curr)
        {
            e.insert(dist);
        } else {
            continue;
        }

        queue.extend(
            grid.neighbors(curr)
                .filter(|(_, n)| grid.data[*n] != Plot::Rock)
                .map(|(_, n)| (n, dist + 1)),
        );
    }

    result
}

// fn all_shortest_paths_from(
//     grid: &Grid<Plot>,
//     start: usize,
// ) -> HashMap<usize, usize> {
//     grid.data
//         .iter()
//         .enumerate()
//         .filter(|(_, p)| **p != Plot::Rock)
//         .filter(|(i, _)| *i != start)
//         .map(|(i, _)| (i, shortest_path(start, i, grid)))
//         .filter_map(|(i, op)| op.map(|o| (i, o)))
//         .collect::<HashMap<_, _>>()
// }

type AllPaths = HashMap<usize, BTreeMap<usize, Vec<usize>>>;

fn steps_to_fill_tile_from(start: usize, all_paths: &AllPaths) -> usize {
    let max_path_len =
        all_paths.get(&start).unwrap().last_key_value().unwrap().0;

    *max_path_len
}

fn get_max_endpoints_from(
    start: usize,
    odd: bool,
    all_paths: &AllPaths,
) -> usize {
    all_paths
        .get(&start)
        .unwrap()
        .iter()
        .filter(|(d, _)| (*d % 2 == 1) == odd)
        .map(|(_, vs)| vs.len())
        .sum()
}

fn get_fill_from(start: usize, all_paths: &AllPaths, steps: usize) -> usize {
    all_paths
        .get(&start)
        .unwrap()
        .range(0..=steps)
        .map(|(_, v)| v.len())
        .sum()
}

fn extend_east(
    west: usize,
    steps_to_fill_from_left: usize,
    mut steps: usize,
    start_odd: bool,
    odd_fill: usize,
    even_fill: usize,
    grid: &Grid<Plot>,
    all_paths: &AllPaths,
) -> (usize, usize) {
    let tiles_to_right = steps / grid.width + 1;

    let mut filled_tiles = tiles_to_right;

    while steps
        .checked_sub(grid.width * filled_tiles)
        .filter(|ex| *ex >= steps_to_fill_from_left)
        .is_none()
    {
        filled_tiles -= 1;
    }

    let mut result = (odd_fill + even_fill) * (filled_tiles / 2);

    if filled_tiles % 2 == 1 {
        result += if start_odd { odd_fill } else { even_fill };
    }

    let mut unfilling_steps = steps - filled_tiles * grid.width;

    while unfilling_steps > 0 {
        result += get_fill_from(west, all_paths, steps);
        unfilling_steps.saturating_sub(grid.width);
    }

    (filled_tiles, result)
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

    grid.data.iter_mut().for_each(|p| {
        if *p != Plot::Rock {
            *p = Plot::None
        }
    });

    let center = start;
    let south = grid.data.len() - grid.width + grid.width / 2;
    let north = grid.width / 2;
    let west = grid.height / 2 * grid.width;
    let east = west + grid.width - 1;

    let starts = [start, north, south, east, west];
    let all_paths = starts
        .iter()
        .copied()
        .map(|i| {
            (
                i,
                all_shortest_paths_from(&grid, i)
                    .into_iter()
                    .map(|(to, dist)| (dist, to))
                    .into_group_map()
                    .into_iter()
                    .collect::<BTreeMap<_, _>>(),
            )
        })
        .collect::<HashMap<_, _>>();

    let steps_from_middle_to_right = grid.width / 2 + 1;
    let right_steps = steps - steps_from_middle_to_right;
    let tiles_to_right = right_steps / grid.width + 1;

    let fill_steps_from = starts
        .into_iter()
        .map(|i| (i, steps_to_fill_tile_from(i, &all_paths)))
        .collect::<HashMap<_, _>>();
    let steps_to_fill_from_left = fill_steps_from.get(&west).unwrap();

    let mut filled_tiles_to_right = tiles_to_right;

    while right_steps
        .checked_sub(grid.width * filled_tiles_to_right)
        .filter(|ex| ex >= steps_to_fill_from_left)
        .is_none()
    {
        filled_tiles_to_right -= 1;
    }

    let steps_odd = steps % 2 == 1;
    let odd_fill = get_max_endpoints_from(center, true, &all_paths);
    let even_fill = get_max_endpoints_from(center, false, &all_paths);

    let left_filled_tile = get_max_endpoints_from(west, !steps_odd, &all_paths);

    let mut total = 0;

    // center
    total += if steps_odd { odd_fill } else { even_fill };

    // right
    total += (odd_fill + even_fill) * (filled_tiles_to_right / 2);

    if filled_tiles_to_right % 2 == 1 {
        total += if steps_odd { even_fill } else { odd_fill }
    }

    println!("{:#?}", total);

    // let starts = Some(start).into_iter().chain(grid.border()).collect_vec();

    // let mut cache = HashMap::new();

    // cache.extend(starts.iter().flat_map(|s| {
    //     saturate(0, *s, &grid)
    //         .into_iter()
    //         .map(move |(steps, res)| ((s, steps), res))
    // }));

    // cache.extend(starts.iter().flat_map(|s| {
    //     saturate(1, *s, &grid)
    //         .into_iter()
    //         .map(move |(steps, res)| ((s, steps), res))
    // }));

    // println!("{}", all_pairs.len());

    0
}

pub fn part_two(input: &str) -> Option<usize> {
    Some(possible_endpoints_inf(input, 26501365))
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_part_two_50() {
        let input = &advent_of_code::template::read_extra_example_file(DAY, 1);

        let steps = 50;
        let brute_result = possible_endpoints_inf_brute(input, steps);

        let result = possible_endpoints_inf(
            &advent_of_code::template::read_file("examples", DAY),
            steps,
        );
        assert_eq!(result, brute_result);
    }
}

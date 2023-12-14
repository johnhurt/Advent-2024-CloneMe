use std::mem::swap;

use advent_of_code::Grid;
use itertools::Itertools;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

advent_of_code::solution!(14);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Rock {
    None,
    Square,
    Round,
}

impl From<Rock> for char {
    fn from(val: Rock) -> Self {
        use Rock as R;
        match val {
            R::None => '.',
            R::Square => '#',
            R::Round => 'O',
        }
    }
}

impl From<char> for Rock {
    fn from(value: char) -> Self {
        use Rock as R;

        match value {
            '.' => R::None,
            '#' => R::Square,
            'O' => R::Round,
            _ => unreachable!("Not part of this puzzle"),
        }
    }
}

#[derive(Debug, Clone, Copy, EnumIter)]
enum Direction {
    N,
    W,
    S,
    E,
}

/// Simulate tipping the platform in the given direction
fn tip(grid: &mut Grid<Rock>, dir: Direction) {
    use Direction as D;
    use Rock as R;
    let width = grid.width;

    match dir {
        D::N => loop {
            let mut changes = 0;

            grid.for_row_pairs_mut(|above, below| {
                changes += above
                    .iter_mut()
                    .zip(below.iter_mut())
                    .filter(|(top, bot)| **top == R::None && **bot == R::Round)
                    .map(|(top, bottom)| swap(top, bottom))
                    .count();
            });

            if changes == 0 {
                break;
            }
        },
        D::S => loop {
            let mut changes = 0;

            grid.for_row_pairs_mut(|above, below| {
                changes += above
                    .iter_mut()
                    .zip(below.iter_mut())
                    .filter(|(top, bot)| **top == R::Round && **bot == R::None)
                    .map(|(top, bottom)| swap(top, bottom))
                    .count();
            });

            if changes == 0 {
                break;
            }
        },
        D::W => {
            while grid
                .rows_mut()
                .map(|row| {
                    (0..width)
                        .tuple_windows()
                        .filter_map(|(left, right)| {
                            (row[left] == R::None && row[right] == R::Round)
                                .then(|| row.swap(left, right))
                        })
                        .count()
                })
                .sum::<usize>()
                > 0
            {}
        }
        D::E => {
            while grid
                .rows_mut()
                .map(|row| {
                    (0..width)
                        .tuple_windows()
                        .filter_map(|(left, right)| {
                            (row[left] == R::Round && row[right] == R::None)
                                .then(|| row.swap(left, right))
                        })
                        .count()
                })
                .sum::<usize>()
                > 0
            {}
        }
    }
}

/// Run the "spin" cycle the given number of times
fn cycle(grid: &mut Grid<Rock>, times: usize) {
    Direction::iter().cycle().take(4 * times).for_each(|d| {
        tip(grid, d);
    });
}

fn calculate_load(grid: &Grid<Rock>) -> usize {
    grid.rows()
        .enumerate()
        .map(|(i, r)| (grid.height - i, r))
        .flat_map(|(d, r)| r.iter().map(move |c| (d, c)))
        .filter(|(_, c)| **c == Rock::Round)
        .map(|(d, _)| d)
        .sum()
}

pub fn part_one(input: &str) -> Option<usize> {
    let mut grid = Grid::<Rock>::parse_lines(input);

    tip(&mut grid, Direction::N);

    Some(calculate_load(&grid))
}

pub fn part_two(input: &str) -> Option<usize> {
    let mut grid = Grid::<Rock>::parse_lines(input);

    // Run for long enough that a repeating pattern should emerge. This is
    // empirical. Not robust :(
    let prefix = 200;
    cycle(&mut grid, prefix);

    let first = calculate_load(&grid);

    let mut cycle_len = 0;
    let mut cycle_vals = vec![first];

    // Again not robust. Just look for a single repeated value and assume that's
    // the start of the cycle repeating :-/ 
    loop {
        cycle_len += 1;
        cycle(&mut grid, 1);
        let load = calculate_load(&grid);

        if first == load {
            break;
        }

        cycle_vals.push(load);
    }

    let result = cycle_vals[(1_000_000_000 - prefix) % cycle_len];

    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cycle() {
        let expected = Grid::<Rock>::parse_lines(
            &advent_of_code::template::read_extra_example_file(DAY, 1),
        );
        let mut grid = Grid::<Rock>::parse_lines(
            &advent_of_code::template::read_file("examples", DAY),
        );

        cycle(&mut grid, 1);

        assert_eq!(grid, expected);
    }

    #[test]
    fn test_part_one() {
        let result =
            part_one(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(136));
    }

    #[test]
    fn test_part_two() {
        let result =
            part_two(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(64));
    }
}

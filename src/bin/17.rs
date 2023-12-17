use std::collections::{BinaryHeap, HashSet};

use advent_of_code::{Compass, Grid, TV4};

advent_of_code::solution!(17);

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct HeatLoss(i64);

impl From<char> for HeatLoss {
    fn from(value: char) -> Self {
        Self((value as u8 - b'0') as i64)
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
struct Path {
    heat_loss: i64,
    curr: usize,
    dir: Option<Compass>,
    linear_steps: usize,
}

impl Path {
    /// Progress the path in the current direction until the minimum number of
    /// steps have been taken. If a boundary is encountered before the required
    /// number of steps is reached, then `None` is returned
    fn take_min_steps(
        self,
        grid: &Grid<HeatLoss>,
        min_steps: usize,
    ) -> Option<Self> {
        (self.linear_steps..min_steps).fold(Some(self), |prev_path, _| {
            prev_path.and_then(|prev| {
                grid.step_from_index(prev.curr, prev.dir.unwrap()).map(
                    |next_p| Path {
                        curr: next_p,
                        heat_loss: prev.heat_loss + grid.data[next_p].0,
                        linear_steps: prev.linear_steps + 1,
                        ..prev
                    },
                )
            })
        })
    }
}

/// Paths are ordered only by heat loss
impl PartialOrd for Path {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        (other.heat_loss).partial_cmp(&(self.heat_loss))
    }
}

impl Ord for Path {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

/// Run dijkstra on the grid of heat-loss values starting from the top-left
/// and running to the bottom right. The top-left heat loss is not used for
/// some reason. The minimum-possible total heat loss is returned
fn crucible_dijkstra(
    grid: &Grid<HeatLoss>,
    min_steps: usize,
    max_steps: usize,
) -> i64 {
    let mut queue = BinaryHeap::new();
    let mut seen = HashSet::new();
    let target = grid.data.len() - 1;

    // This queue will always return the path with least heat loss
    queue.push(Default::default());

    while let Some(Path {
        curr,
        dir,
        heat_loss,
        linear_steps,
        ..
    }) = queue.pop()
    {
        // This is important. It's not just the position we need to worry about
        // seeing before, it's the position and how we got there. This took a
        // long time to figure out X(
        if !seen.insert((linear_steps, dir, curr)) {
            continue;
        }

        if curr == target {
            return heat_loss;
        }

        // Allowed neighbors are determined based where in the grid we are
        // plus how far in the current direction we have already gone
        let neighbors = match (dir, linear_steps) {
            (None, _) => grid.neighbors(curr).collect::<TV4<_>>(),
            (Some(dir), steps) if steps == max_steps => grid
                .neighbors(curr)
                .filter(|(d, _)| *d != dir)
                .filter(|(d, _)| *d != dir.opposite())
                .collect::<TV4<_>>(),
            (Some(dir), _) => grid
                .neighbors(curr)
                .filter(|(d, _)| *d != dir.opposite())
                .collect::<TV4<_>>(),
        };

        queue.extend(
            neighbors
                .into_iter()
                .map(|(next_dir, next)| Path {
                    heat_loss: heat_loss + grid.data[next].0,
                    curr: next,
                    dir: Some(next_dir),
                    linear_steps: if Some(next_dir) == dir {
                        linear_steps + 1
                    } else {
                        1
                    },
                })
                .filter_map(|path| path.take_min_steps(grid, min_steps))
                .filter(|path| {
                    !seen.contains(&(path.linear_steps, path.dir, path.curr))
                }),
        );
    }

    unreachable!()
}

pub fn part_one(input: &str) -> Option<i64> {
    let grid = Grid::<HeatLoss>::parse_lines(input);

    let result = crucible_dijkstra(&grid, 0, 3);

    Some(result)
}

pub fn part_two(input: &str) -> Option<i64> {
    let grid = Grid::<HeatLoss>::parse_lines(input);

    let result = crucible_dijkstra(&grid, 4, 10);

    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let result =
            part_one(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(102));
    }

    #[test]
    fn test_part_two() {
        let result =
            part_two(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(94));
    }
}

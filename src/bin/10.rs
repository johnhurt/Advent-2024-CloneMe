use std::collections::HashSet;

use itertools::Itertools;
use strum::IntoEnumIterator;
use strum_macros::{EnumIter, FromRepr};
use tinyvec::tiny_vec;

advent_of_code::solution!(10);

#[repr(isize)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, EnumIter)]
enum Direction {
    #[default]
    N,
    S,
    E,
    W,
}

impl Direction {
    fn opposite(&self) -> Self {
        use Direction as D;

        match self {
            D::E => D::W,
            D::W => D::E,
            D::S => D::N,
            D::N => D::S,
        }
    }

    /// Get the neighbor to the current node in this direction based on a grid
    /// width given
    fn neighbor(&self, curr: usize, width: isize) -> Option<usize> {
        use Direction as D;

        let delta = match self {
            D::N => -width,
            D::S => width,
            D::W => -1,
            D::E => 1,
        };

        curr.checked_add_signed(delta)
    }
}

/// Enumeration of all the pipe types given. Not all are actual pipes, but this
/// enum deriving `FromRepr` was a convenient way to parse the grid
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromRepr)]
#[repr(u8)]
enum Pipe {
    Start = b'S',
    None = b'.',
    NS = b'|',
    EW = b'-',
    NE = b'L',
    NW = b'J',
    SE = b'F',
    SW = b'7',
}

impl Pipe {
    /// Get the pipe piece that would satisfy the given two (assumed different)
    /// directions. This is used to determine which pipe piece goes at `S`
    fn from_dirs(d1: Direction, d2: Direction) -> Pipe {
        use Direction as D;

        match (d1, d2) {
            (D::N, D::S) | (D::S, D::N) => Pipe::NS,
            (D::E, D::W) | (D::W, D::E) => Pipe::EW,
            (D::N, D::E) | (D::E, D::N) => Pipe::NE,
            (D::N, D::W) | (D::W, D::N) => Pipe::NW,
            (D::S, D::W) | (D::W, D::S) => Pipe::SW,
            (D::S, D::E) | (D::E, D::S) => Pipe::SE,
            _ => unimplemented!("Shouldn't appear in this problem"),
        }
    }

    /// Get the resulting direction from traversing this piece of pipe from the
    /// given direction
    fn traverse_from(&self, dir: Direction) -> Option<Direction> {
        use Direction as D;
        use Pipe as P;

        match (dir, self) {
            (D::E, P::EW) | (D::S, P::SW) | (D::N, P::NW) => Some(D::W),
            (D::W, P::EW) | (D::S, P::SE) | (D::N, P::NE) => Some(D::E),
            (D::S, P::NS) | (D::E, P::NE) | (D::W, P::NW) => Some(D::N),
            (D::N, P::NS) | (D::E, P::SE) | (D::W, P::SW) => Some(D::S),
            _ => None,
        }
    }

    /// Get the directions that this piece of pipe has openings. This is used
    /// to pick a starting direction in traversals
    fn outputs(&self) -> (Direction, Direction) {
        use Direction as D;
        use Pipe as P;

        match self {
            P::NS => (D::N, D::S),
            P::EW => (D::E, D::W),
            P::NE => (D::N, D::E),
            P::NW => (D::N, D::W),
            P::SE => (D::S, D::E),
            P::SW => (D::S, D::W),
            _ => unimplemented!("Shouldn't happen in this puzzle"),
        }
    }

    /// This is used for part 2 to get the nodes that appear on the left side
    /// of this piece of pipe when traversing from the given direction.
    fn left_side_neighbors(
        &self,
        from_dir: Direction,
        curr: usize,
        width: isize,
    ) -> impl Iterator<Item = usize> {
        use Direction as D;
        use Pipe as P;

        let mut result = tiny_vec!([Direction; 2]);

        match (from_dir, self) {
            (D::E, P::EW) => result.push(D::S),
            (D::E, P::NE) => result.extend([D::S, D::W]),
            (D::W, P::EW) => result.push(D::N),
            (D::W, P::SW) => result.extend([D::N, D::E]),
            (D::N, P::NS) => result.push(D::E),
            (D::N, P::NW) => result.extend([D::E, D::S]),
            (D::S, P::NS) => result.push(D::W),
            (D::S, P::SE) => result.extend([D::W, D::N]),
            _ => {}
        }

        result
            .into_iter()
            .filter_map(move |dir| dir.neighbor(curr, width))
    }
}

/// Check to see if traversal is allowed from the current position in the
/// direction given
fn check_traversal(
    pipes: &[Pipe],
    curr: usize,
    dir: Direction,
    width: isize,
) -> Option<usize> {
    dir.neighbor(curr, width)
        .filter(|n| *n < pipes.len())
        .map(|n| (n, pipes[n]))
        .filter(|(_, p)| p.traverse_from(dir.opposite()).is_some())
        .map(|(n, _)| n)
}

fn parse(input: &str) -> (usize, isize, Vec<Pipe>) {
    let width = input.find('\n').unwrap() as isize + 1;

    let mut start_opt = None;

    let mut pipes = input
        .chars()
        .map(|c| if c == '\n' { '.' } else { c })
        .map(|c| Pipe::from_repr(c as u8).unwrap())
        .enumerate()
        .map(|(i, p)| {
            if p == Pipe::Start {
                start_opt = Some(i);
            }
            p
        })
        .collect::<Vec<_>>();

    let start = start_opt.unwrap();

    let (start_d1, start_d2) = Direction::iter()
        .filter(|dir| check_traversal(&pipes, start, *dir, width).is_some())
        .tuples()
        .next()
        .unwrap();

    pipes[start] = Pipe::from_dirs(start_d1, start_d2);

    (start, width, pipes)
}

/// Struct used to make it possible to iterate through all steps in the
/// traversal of a cycle of pipe. `std::Iterator` is implemented for this struct
/// below and will yield a position in the grid and direction taken to arrive
/// there. It will end after the traversal returns to the starting point
struct PipeTraversal<'a> {
    done: bool,
    start: usize,
    curr: usize,
    width: isize,
    dir: Direction,
    pipes: &'a [Pipe],
}

impl<'a> PipeTraversal<'a> {
    fn new(pipes: &'a [Pipe], start: usize, width: isize) -> Self {
        Self {
            done: false,
            start,
            dir: pipes[start].outputs().0,
            pipes,
            curr: start,
            width,
        }
    }
}

impl<'a> Iterator for PipeTraversal<'a> {
    type Item = (Direction, usize);

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        let next_p = self.dir.neighbor(self.curr, self.width).unwrap();

        let from_dir = self.dir.opposite();
        let next_dir = self.pipes[next_p].traverse_from(from_dir).unwrap();

        self.curr = next_p;
        self.dir = next_dir;

        if self.curr == self.start {
            self.done = true;
        }

        Some((from_dir, next_p))
    }
}

pub fn part_one(input: &str) -> Option<u32> {
    let (start, width, pipes) = parse(input);

    // All we need to know here is how far it is around the cycle
    let cycle_len = PipeTraversal::new(&pipes, start, width).count();

    // The maximum distance will be the length over 2
    Some(cycle_len as u32 / 2)
}

/// Perform a search from the given starting point and mark all nodes that
/// can be reached and aren't covered. Passing in the queue here is just a way
/// to save allocations by reusing working space
fn mark_connecting_uncovered_nodes(
    covered: &HashSet<usize>,
    marked: &'_ mut HashSet<usize>,
    queue: &'_ mut Vec<usize>,
    start: usize,
    width: isize,
    max: usize,
) {
    queue.clear();
    queue.push(start);

    while let Some(curr) = queue.pop() {
        marked.insert(curr);
        queue.extend(
            Direction::iter()
                .filter_map(|d| d.neighbor(curr, width))
                .filter(|curr| *curr < max)
                .filter(|curr| !covered.contains(curr))
                .filter(|curr| !marked.contains(curr)),
        )
    }
}

/// For part 2 we need to know the nodes enclosed by the pipe, but when we
/// traverse the cycle, we don't know at any point which side is the inside
/// and which side is the outside. This function iterators of the nodes that
/// appear on one side of the cycle, and nodes that appear on the other. We
/// can decide later which is inside
fn boundary_nodes(
    pipes: &[Pipe],
    start: usize,
    width: isize,
) -> (
    impl Iterator<Item = usize> + '_,
    impl Iterator<Item = usize> + '_,
) {
    let left = PipeTraversal::new(pipes, start, width);
    let mut right = PipeTraversal::new(pipes, start, width);

    right.dir = pipes[start].outputs().1;

    (
        left.flat_map(move |(from_dir, curr)| {
            pipes[curr].left_side_neighbors(from_dir, curr, width)
        }),
        right.flat_map(move |(from_dir, curr)| {
            pipes[curr].left_side_neighbors(from_dir, curr, width)
        }),
    )
}

/// Search for any uncovered nodes that are reachable from any of the nodes in
/// the set of starting points
fn uncovered_nodes_reachable_from<I>(
    covered: &HashSet<usize>,
    starting_points: I,
    width: isize,
    max: usize,
) -> HashSet<usize>
where
    I: Iterator<Item = usize>,
{
    let mut result = HashSet::new();
    let mut working_queue = vec![];

    starting_points.for_each(|search_start| {
        if covered.contains(&search_start) || result.contains(&search_start) {
            return;
        }

        mark_connecting_uncovered_nodes(
            covered,
            &mut result,
            &mut working_queue,
            search_start,
            width,
            max,
        )
    });

    result
}

pub fn part_two(input: &str) -> Option<u32> {
    let (start, width, pipes) = parse(input);

    // Create a set of all nodes covered with pipe
    let covered = PipeTraversal::new(&pipes, start, width)
        .map(|(_, curr)| curr)
        .collect::<HashSet<_>>();

    // left and right are iterators of all the points that are adjacent to the
    // traversed cycle on the left side or right side. One will contain only
    // outside nodes and one will contain only inside nodes, but we don't know
    // which is which
    let (left, right) = boundary_nodes(&pipes, start, width);

    // Find all the nodes connected to the left-side nodes and right-side nodes
    let left_nodes =
        uncovered_nodes_reachable_from(&covered, left, width, input.len());
    let right_nodes =
        uncovered_nodes_reachable_from(&covered, right, width, input.len());

    // We just assume the nodes inside will be fewer than the ones outside
    let result = left_nodes.len().min(right_nodes.len());

    Some(result as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let result =
            part_one(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(8));
    }

    #[test]
    fn test_part_two() {
        let result = part_two(
            &advent_of_code::template::read_extra_example_file(DAY, 1),
        );
        assert_eq!(result, Some(4));
    }

    #[test]
    fn test_part_two_again() {
        let result = part_two(
            &advent_of_code::template::read_extra_example_file(DAY, 2),
        );
        assert_eq!(result, Some(10));
    }

    #[test]
    fn test_part_two_a_third_time() {
        let result = part_two(
            &advent_of_code::template::read_extra_example_file(DAY, 3),
        );
        assert_eq!(result, Some(8));
    }
}

use std::collections::{BTreeSet, HashSet};

advent_of_code::solution!(11);

/// The two parts of this puzzle are the same except for the amount of distance
/// that empty rows/cols contribute to total distances
fn solve_puzzle(input: &str, empty_line_dist: usize) -> usize {
    let width = input.find('\n').unwrap() + 1;
    let height = input.len() / width + 1;

    let mut unused_rows = (0..height).collect::<HashSet<_>>();
    let mut unused_cols = (0..width).collect::<HashSet<_>>();

    let galaxies = input
        .char_indices()
        .filter(|(_, c)| *c == '#')
        .map(|(i, _)| (i / width, i % width))
        .collect::<Vec<_>>();

    galaxies.iter().copied().for_each(|(r, c)| {
        unused_rows.remove(&r);
        unused_cols.remove(&c);
    });

    // convert the unused row and columns to sorted sets
    let unused_rows = unused_rows.into_iter().collect::<BTreeSet<_>>();
    let unused_cols = unused_cols.into_iter().collect::<BTreeSet<_>>();

    let mut result = 0;

    for i in 0..galaxies.len() {
        for j in i..galaxies.len() {
            let (r1, c1) = galaxies[i];
            let (r2, c2) = galaxies[j];

            let row_range = (r1.min(r2))..(r1.max(r2));
            let col_range = (c1.min(c2))..(c1.max(c2));

            let empty_rows = unused_rows.range(row_range).count();
            let empty_cols = unused_cols.range(col_range).count();

            // Raw distance
            let mut dist = r2.abs_diff(r1) + c1.abs_diff(c2);

            // Remove contributions from empty lines
            dist -= empty_cols + empty_rows;

            // Add scaled distance contribution from empty lines
            dist += (empty_cols + empty_rows) * empty_line_dist;

            result += dist;
        }
    }

    result
}

pub fn part_one(input: &str) -> Option<usize> {
    Some(solve_puzzle(input, 2))
}

pub fn part_two(input: &str) -> Option<usize> {
    Some(solve_puzzle(input, 1_000_000))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let result =
            part_one(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(374));
    }

    #[test]
    fn test_part_two() {
        let result = solve_puzzle(
            &advent_of_code::template::read_file("examples", DAY),
            100,
        );
        assert_eq!(result, 8410);
    }
}

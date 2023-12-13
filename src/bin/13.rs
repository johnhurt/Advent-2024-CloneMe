use itertools::Itertools;

advent_of_code::solution!(13);

/// Turn the lines in this string into a list of u64s turning `#`s into ones
/// and `.`s into zeros in the binary representation
fn horizontal_values(input: &str) -> Vec<u64> {
    input
        .lines()
        .map(|line| {
            line.chars()
                .fold(0, |accum, c| (c == '#') as u64 + (accum << 1))
        })
        .collect_vec()
}

/// Turn the columns in this string into a list of u64s turning `#`s into ones
/// and `.`s into zeros in the binary representation
fn vertical_values(input: &str) -> Vec<u64> {
    let width = input.find('\n').unwrap();
    let input = input.replace('\n', "");
    let height = input.len() / width;
    let mut result = vec![];

    for col in 0..width {
        let mut val = 0;
        for row in 0..height {
            val = (val << 1)
                + (input.as_bytes()[row * width + col] == b'#') as u64;
        }
        result.push(val);
    }

    result
}

/// Count the number of values on the list that pass before the numbers start
/// repeating backwards (if any)
fn count_before_reflection(vals: &[u64]) -> Option<u64> {
    for i in 0..(vals.len() - 1) {
        let mut j = i as i64;
        let mut k = i + 1;
        let mut found = true;

        while j >= 0 && k < vals.len() {
            let left = vals[j as usize];
            let right = vals[k];

            let equal = left == right;

            if !equal {
                found = false;
                break;
            }

            j -= 1;
            k += 1;
        }

        if found {
            return Some(i as u64 + 1);
        }
    }

    None
}

/// This is the same as the function above, but used for part 2. Instead of
/// strict equality, we allow one near equal where left xor right has one bit
/// set to 1 (making it a power of two)
fn count_before_smudged_reflection(vals: &[u64]) -> Option<u64> {
    for i in 0..(vals.len() - 1) {
        let mut j = i as i64;
        let mut k = i + 1;
        let mut found = true;
        let mut smudge_found = false;

        while j >= 0 && k < vals.len() {
            let left = vals[j as usize];
            let right = vals[k];

            let equal = left == right;
            let smudge = (left ^ right).is_power_of_two() && !smudge_found;

            if smudge {
                smudge_found = true;
            }

            if !equal && !smudge {
                found = false;
                break;
            }

            j -= 1;
            k += 1;
        }

        if found && smudge_found {
            return Some(i as u64 + 1);
        }
    }

    None
}

pub fn part_one(input: &str) -> Option<u64> {
    let blocks = input.split("\n\n").collect_vec();
    let num_above = blocks
        .iter()
        .map(|block| horizontal_values(block))
        .filter_map(|vals| count_before_reflection(&vals))
        .sum::<u64>();

    let num_left = blocks
        .iter()
        .map(|block| vertical_values(block))
        .filter_map(|vals| count_before_reflection(&vals))
        .sum::<u64>();

    Some(num_left + 100 * num_above)
}

pub fn part_two(input: &str) -> Option<u64> {
    let blocks = input.split("\n\n").collect_vec();
    let num_above = blocks
        .iter()
        .map(|block| horizontal_values(block))
        .filter_map(|vals| count_before_smudged_reflection(&vals))
        .sum::<u64>();

    let num_left = blocks
        .iter()
        .map(|block| vertical_values(block))
        .filter_map(|vals| count_before_smudged_reflection(&vals))
        .sum::<u64>();

    Some(num_left + 100 * num_above)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let result =
            part_one(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(405));
    }

    #[test]
    fn test_part_two() {
        let result =
            part_two(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(400));
    }
}

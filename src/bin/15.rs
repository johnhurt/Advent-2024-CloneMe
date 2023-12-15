use hashlink::LinkedHashMap;
use nom::{
    branch::alt, bytes::complete::tag, character::complete::alpha1,
    character::complete::u32, combinator::opt, sequence::tuple, IResult,
};

advent_of_code::solution!(15);

#[derive(Debug)]
enum Instruction<'a> {
    Add { label: &'a str, focal_len: u32 },
    Del(&'a str),
}

fn parse_instruction(input: &str) -> Instruction<'_> {
    let result: IResult<_, _> =
        tuple((alpha1, alt((tag("-"), tag("="))), opt(u32)))(input);
    let (label, op, focal_len) = result.unwrap().1;

    match op {
        "=" => Instruction::Add {
            label,
            focal_len: focal_len.unwrap(),
        },
        "-" => Instruction::Del(label),
        _ => unreachable!("Not here, buddy"),
    }
}

#[allow(non_snake_case)]
fn HASH(label: &str) -> u32 {
    label
        .as_bytes()
        .iter()
        .fold(0, |res, c| ((res + *c as u32) * 17) % 256)
}

#[allow(non_snake_case)]
fn HASHMAP<'a>(input: &'a str, boxes: &mut [LinkedHashMap<&'a str, u32>]) {
    use Instruction as I;
    let inst = parse_instruction(input);

    match inst {
        I::Add { label, focal_len } => {
            let hash = HASH(label);
            if let Some(existing) = boxes[hash as usize].get_mut(label) {
                *existing = focal_len;
            } else {
                boxes[hash as usize].insert(label, focal_len);
            }
        }
        I::Del(label) => {
            let hash = HASH(label);
            boxes[hash as usize].remove(label);
        }
    }
}

pub fn part_one(input: &str) -> Option<u32> {
    let result = input.trim().split(',').map(HASH).sum::<u32>();
    Some(result)
}

pub fn part_two(input: &str) -> Option<usize> {
    let mut boxes = vec![LinkedHashMap::new(); 256];

    input
        .trim()
        .split(',')
        .for_each(|inst| HASHMAP(inst, &mut boxes));

    let result = boxes
        .into_iter()
        .enumerate()
        .map(|(i, bx)| {
            bx.values()
                .enumerate()
                .map(move |(j, f)| (i + 1) * (j + 1) * *f as usize)
                .sum::<usize>()
        })
        .sum::<usize>();

    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let result =
            part_one(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(1320));
    }

    #[test]
    fn test_part_two() {
        let result =
            part_two(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(145));
    }
}

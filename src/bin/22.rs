use itertools::Itertools;
use nom::{
    bytes::complete::tag,
    character::complete::i64,
    sequence::{terminated, tuple},
    IResult,
};
use num_traits::Bounded;
use rstar::{DefaultParams, Envelope, Point, RTree, RTreeObject, AABB};

advent_of_code::solution!(22);

type Corner = (i64, i64, i64);

struct Brick(Corner, Corner);

fn parse(line: &str) -> Brick {
    let result: IResult<_, _> = (tuple((
        tuple((
            terminated(i64, tag(",")),
            terminated(i64, tag(",")),
            terminated(i64, tag("~")),
        )),
        tuple((terminated(i64, tag(",")), terminated(i64, tag(",")), i64)),
    )))(line);

    let ((x1, y1, z1), (x2, y2, z2)) = result.unwrap().1;

    Brick((x1, y1, z1), (x2, y2, z2))
}

// impl Envelope for Brick {
//     type Point = (i64, i64, i64);
// }

fn p() -> impl Point {
    (0_i64, 0_i64, 0_i64)
}

pub fn part_one(input: &str) -> Option<u32> {
    let bricks = input.lines().map(parse).collect_vec();

    let i = (0_i64, 0_i64, 0_i64);
    let p = p();

    let mut r_tree: RTree<Corner, DefaultParams> = RTree::default();

    let a = AABB::from_corners(bricks[0].0, bricks[0].1);

    r_tree.ins(a);

    println!("{}", bricks.len());

    None
}

pub fn part_two(input: &str) -> Option<u32> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let result =
            part_one(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(5));
    }

    #[test]
    fn test_part_two() {
        let result =
            part_two(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, None);
    }
}

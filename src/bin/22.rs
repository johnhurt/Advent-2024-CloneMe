use core::panic;
use std::collections::{BTreeMap, HashSet};

use itertools::Itertools;
use nom::{
    bytes::complete::tag,
    character::complete::i64,
    sequence::{terminated, tuple},
    IResult,
};
use rstar::{primitives::Rectangle, DefaultParams, Point, RTree, AABB};

advent_of_code::solution!(22);

type Corner = (i64, i64, i64);

type Brick = (Corner, Corner);
type RT = RTree<Rectangle<Corner>, DefaultParams>;

fn parse(line: &str) -> Brick {
    let result: IResult<_, _> = (tuple((
        tuple((
            terminated(i64, tag(",")),
            terminated(i64, tag(",")),
            terminated(i64, tag("~")),
        )),
        tuple((terminated(i64, tag(",")), terminated(i64, tag(",")), i64)),
    )))(line);

    result.unwrap().1
}

fn parse_and_fall(input: &str) -> RT {
    let bricks = input.lines().map(parse).collect_vec();

    let bricks_by_bottom = bricks
        .iter()
        .copied()
        .map(|b| (b.0 .2.min(b.1 .2), b))
        .into_group_map()
        .into_iter()
        .collect::<BTreeMap<_, _>>();

    let mut r_tree = RTree::bulk_load(
        bricks
            .iter()
            .copied()
            .map(|b| Rectangle::from_corners(b.0, b.1))
            .collect_vec(),
    );

    bricks_by_bottom
        .values()
        .flatten()
        .copied()
        .for_each(|mut b| {
            let min_z = b.0 .2.min(b.1 .2);

            let shadow = AABB::from_corners(
                (b.0 .0.min(b.1 .0), b.0 .1.min(b.1 .1), 0),
                (b.0 .0.max(b.1 .0), b.0 .1.max(b.1 .1), min_z - 1),
            );

            let below = r_tree.locate_in_envelope_intersecting(&shadow);

            let max_z = below.map(|r| r.upper().2).max().unwrap_or_default();

            let z_shift = min_z - max_z - 1;

            if z_shift < 0 {
                panic!();
            }

            let old_rect = Rectangle::from_corners(b.0, b.1);

            b.0 .2 -= z_shift;
            b.1 .2 -= z_shift;

            let new_rect = Rectangle::from_corners(b.0, b.1);

            assert!(r_tree.remove(&old_rect).is_some());
            r_tree.insert(new_rect);

            let down_shadow = AABB::from_corners(
                (
                    new_rect.lower().0,
                    new_rect.lower().1,
                    new_rect.lower().2 - 1,
                ),
                (
                    new_rect.upper().0,
                    new_rect.upper().1,
                    new_rect.lower().2 - 1,
                ),
            );

            let below =
                r_tree.locate_in_envelope_intersecting(&down_shadow).count();

            if below == 0 && new_rect.lower().2 > 1 {
                panic!();
            }
        });

    r_tree
}

pub fn part_one(input: &str) -> Option<usize> {
    let r_tree = parse_and_fall(input);

    let result = r_tree
        .iter()
        .copied()
        .filter(|rect| {
            let up_shadow = AABB::from_corners(
                (rect.lower().0, rect.lower().1, rect.upper().2 + 1),
                (rect.upper().0, rect.upper().1, rect.upper().2 + 1),
            );

            let res = r_tree.locate_in_envelope_intersecting(&up_shadow).all(
                |upper_rect| {
                    let c1 = upper_rect.lower();
                    let c2 = upper_rect.upper();
                    let min_z = c1.2;

                    let down_shadow = AABB::from_corners(
                        (c1.0, c1.1, min_z - 1),
                        (c2.0, c2.1, min_z - 1),
                    );

                    let below = r_tree
                        .locate_in_envelope_intersecting(&down_shadow)
                        .count();

                    below > 1
                },
            );

            res
        })
        .count();

    Some(result)
}

fn get_falling_bricks(
    r_tree: &RT,
    rect: &Rectangle<Corner>,
    mut gone: HashSet<Brick>,
) -> HashSet<Brick> {
    let up_shadow = AABB::from_corners(
        (rect.lower().0, rect.lower().1, rect.upper().2 + 1),
        (rect.upper().0, rect.upper().1, rect.upper().2 + 1),
    );

    //gone.insert((rect.lower(), rect.upper()));

    let falling = r_tree
        .locate_in_envelope_intersecting(&up_shadow)
        .filter(|up_rect| {
            let c1 = up_rect.lower();
            let c2 = up_rect.upper();
            let min_z = c1.2;

            let down_shadow = AABB::from_corners(
                (c1.0, c1.1, min_z - 1),
                (c2.0, c2.1, min_z - 1),
            );

            let below = r_tree
                .locate_in_envelope_intersecting(&down_shadow)
                .filter(|r| !gone.contains(&(r.lower(), r.upper())))
                .count();

            below <= 1
        })
        .collect_vec();

    gone.extend(falling.iter().map(|f| (f.lower(), f.upper())));

    for fall_rect in falling.into_iter() {
        let g_clone = gone.clone();
        gone.extend(get_falling_bricks(r_tree, fall_rect, g_clone));
    }

    gone
}

fn get_falling_bricks_until_done(
    r_tree: &RT,
    rect: &Rectangle<Corner>,
) -> HashSet<Brick> {
    let mut gone = HashSet::new();
    let mut gone_size = 0;

    loop {
        gone = get_falling_bricks(r_tree, rect, gone);
        if gone_size == gone.len() {
            break;
        }
        gone_size = gone.len();
    }

    gone
}

fn simulate_fall(mut r_tree: RT, rect: &Rectangle<Corner>) -> usize {
    r_tree.remove(rect);

    let mut result = 0;

    loop {
        let to_remove = r_tree
            .iter()
            .copied()
            .filter(|r| {
                let c1 = r.lower();
                let c2 = r.upper();
                let min_z = c1.2;

                let down_shadow = AABB::from_corners(
                    (c1.0, c1.1, min_z - 1),
                    (c2.0, c2.1, min_z - 1),
                );

                let below = r_tree
                    .locate_in_envelope_intersecting(&down_shadow)
                    .count();

                below == 0 && c1.2 > 1
            })
            .collect_vec();

        if to_remove.is_empty() {
            break;
        }

        result += to_remove.len();

        to_remove
            .into_iter()
            .for_each(|r| assert!(r_tree.remove(&r).is_some()));
    }

    result
}

pub fn part_two(input: &str) -> Option<usize> {
    let r_tree = parse_and_fall(input);

    let result = r_tree
        .iter()
        .map(|r| simulate_fall(r_tree.clone(), r))
        //.map(|rect| get_falling_bricks_until_done(&r_tree, rect))
        //.map(|f| dbg!(f))
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
        assert_eq!(result, Some(5));
    }

    #[test]
    fn test_part_two() {
        let result =
            part_two(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(7));
    }
}

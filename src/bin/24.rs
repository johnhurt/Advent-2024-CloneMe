use std::ops::RangeInclusive;

use advent_of_code::ws;
use itertools::Itertools;
use nom::{
    bytes::complete::tag,
    character::complete::i64,
    sequence::{terminated, tuple},
    IResult,
};

advent_of_code::solution!(24);

const EPSILON: f64 = 1e-12;

type F3 = (f64, f64, f64);

#[derive(Debug, Clone, Copy, PartialEq)]
struct Hail {
    p: F3,
    v: F3,
}

fn parse(line: &str) -> Hail {
    let result: IResult<_, _> = (tuple((
        tuple((
            terminated(ws(i64), tag(",")),
            terminated(ws(i64), tag(",")),
            terminated(ws(i64), ws(tag("@"))),
        )),
        tuple((
            terminated(ws(i64), tag(",")),
            terminated(ws(i64), tag(",")),
            ws(i64),
        )),
    )))(line);

    let ((px, py, pz), (vx, vy, vz)) = result.unwrap().1;

    Hail {
        p: (px as f64, py as f64, pz as f64),
        v: (vx as f64, vy as f64, vz as f64),
    }
}

fn intersect_xy(h1: &Hail, h2: &Hail) -> Option<(f64, f64)> {
    let t_opt = match (h1.v.0.abs() > EPSILON, h2.v.0.abs() > EPSILON) {
        (false, false) => {
            // both particles are only moving in y means they are either
            // constantly colliding or parallel
            None
        }
        (true, false) => Some((h1.p.0 - h2.p.0) / h2.v.0),
        (false, true) => Some((h2.p.0 - h1.p.0) / h1.v.0),
        (true, true) => {
            let denom = h1.v.1 / h1.v.0 - h2.v.1 / h2.v.0;

            if denom.abs() < EPSILON {
                None
            } else {
                let num = h2.p.1 - h1.p.1 + h1.p.0 * h1.v.1 / h1.v.0
                    - h2.p.0 * h2.v.1 / h2.v.0;

                let x = num / denom;

                // The collision point can't be in the past of either stone
                let t1 = x / h1.v.0 - h1.p.0 / h1.v.0;
                let t2 = x / h2.v.0 - h2.p.0 / h2.v.0;

                (t1 >= 0. && t2 >= 0.).then_some(t1)
            }
        }
    };

    t_opt.map(|t| (h1.p.0 + h1.v.0 * t, h1.p.1 + h1.v.1 * t))
}

fn count_intersections(input: &str, test_box: RangeInclusive<f64>) -> usize {
    let stones = input.lines().map(parse).collect_vec();

    let wx = test_box.clone();
    let wy = wx.clone();

    let result = stones
        .iter()
        .copied()
        .cartesian_product(stones.iter().copied())
        .filter(|(s1, s2)| s1 != s2)
        .filter_map(|(s1, s2)| intersect_xy(&s1, &s2))
        .filter(|(px, py)| wx.contains(px) && wy.contains(py))
        .count()
        / 2;
    result
}

pub fn part_one(input: &str) -> Option<usize> {
    let result =
        count_intersections(input, (200000000000000.)..=400000000000000.);
    Some(result)
}

pub fn part_two(_: &str) -> Option<u32> {
    // I have no idea how to do this algorithmically, so I used sage and put in
    // the equations for the first 3 collision points. It looks like this
    //
    // var = [ a_0,
    //         b_0,
    //         c_0,
    //         d_0,
    //         e_0,
    //         f_0,
    //         a_1,
    //         b_1,
    //         c_1,
    //         d_1,
    //         e_1,
    //         f_1,
    //         t_1,
    //         x_1,
    //         y_1,
    //         z_1,
    //         a_2,
    //         b_2,
    //         c_2,
    //         d_2,
    //         e_2,
    //         f_2,
    //         t_2,
    //         x_2,
    //         y_2,
    //         z_2,
    //         a_3,
    //         b_3,
    //         c_3,
    //         d_3,
    //         e_3,
    //         f_3,
    //         t_3,
    //         x_3,
    //         y_3,
    //         z_3 ];
    //
    //   solve([
    //     x_1= a_1 + d_1 * t_1,
    //     y_1=b_1 + e_1 * t_1,
    //     z_1=c_1 + f_1 * t_1,
    //     x_1= a_0 + d_0 * t_1,
    //     y_1=b_0 + e_0 * t_1,
    //     z_1=c_0 + f_0 * t_1,
    //     x_2= a_2 + d_2 * t_2,
    //     y_2=b_2 + e_2 * t_2,
    //     z_2=c_2 + f_2 * t_2,
    //     x_2= a_0 + d_0 * t_2,
    //     y_2=b_0 + e_0 * t_2,
    //     z_2=c_0 + f_0 * t_2,
    //     x_3= a_3 + d_3 * t_3,
    //     y_3=b_3 + e_3 * t_3,
    //     z_3=c_3 + f_3 * t_3,
    //     x_3= a_0 + d_0 * t_3,
    //     y_3=b_0 + e_0 * t_3,
    //     z_3=c_0 + f_0 * t_3,
    //     a_1=246721424318191,
    //     b_1=306735195971895,
    //     c_1=195640804079938,
    //     d_1=46,
    //     e_1=-42,
    //     f_1=141,
    //     a_2=286716952521568,
    //     b_2=348951612232772,
    //     c_2=274203424013154,
    //     d_2=121,
    //     e_2=421,
    //     f_2=-683,
    //     a_3=231402843137765,
    //     b_3=83297412652001,
    //     c_3=273065723902291,
    //     d_3=30,
    //     e_3=154,
    //     f_3=66 ]);
    //
    //  the result is a_0 + b_0 + c_0
    //
    // This url might work too https://quickmath.com/webMathematica3/quickmath/equations/solve/advanced.jsp#c=solve_solveequationsadvanced&v1=x_1%253D%2520a_1%2520%2B%2520d_1%2520*%2520t_1%250Ay_1%253Db_1%2520%2B%2520e_1%2520*%2520t_1%250Az_1%253Dc_1%2520%2B%2520f_1%2520*%2520t_1%250Ax_1%253D%2520a_0%2520%2B%2520d_0%2520*%2520t_1%250Ay_1%253Db_0%2520%2B%2520e_0%2520*%2520t_1%250Az_1%253Dc_0%2520%2B%2520f_0%2520*%2520t_1%250Ax_2%253D%2520a_2%2520%2B%2520d_2%2520*%2520t_2%250Ay_2%253Db_2%2520%2B%2520e_2%2520*%2520t_2%250Az_2%253Dc_2%2520%2B%2520f_2%2520*%2520t_2%250Ax_2%253D%2520a_0%2520%2B%2520d_0%2520*%2520t_2%250Ay_2%253Db_0%2520%2B%2520e_0%2520*%2520t_2%250Az_2%253Dc_0%2520%2B%2520f_0%2520*%2520t_2%250Ax_3%253D%2520a_3%2520%2B%2520d_3%2520*%2520t_3%250Ay_3%253Db_3%2520%2B%2520e_3%2520*%2520t_3%250Az_3%253Dc_3%2520%2B%2520f_3%2520*%2520t_3%250Ax_3%253D%2520a_0%2520%2B%2520d_0%2520*%2520t_3%250Ay_3%253Db_0%2520%2B%2520e_0%2520*%2520t_3%250Az_3%253Dc_0%2520%2B%2520f_0%2520*%2520t_3%250Aa_1%253D246721424318191%250Ab_1%253D306735195971895%250Ac_1%253D195640804079938%250Ad_1%253D46%250Ae_1%253D-42%250Af_1%253D141%250Aa_2%253D286716952521568%250Ab_2%253D348951612232772%250Ac_2%253D274203424013154%250Ad_2%253D121%250Ae_2%253D421%250Af_2%253D-683%250Aa_3%253D231402843137765%250Ab_3%253D83297412652001%250Ac_3%253D273065723902291%250Ad_3%253D30%250Ae_3%253D154%250Af_3%253D66&v2=a_0%250Ab_0%250Ac_0%250Ad_0%250Ae_0%250Af_0%250Aa_1%250Ab_1%250Ac_1%250Ad_1%250Ae_1%250Af_1%250At_1%250Ax_1%250Ay_1%250Az_1%250Aa_2%250Ab_2%250Ac_2%250Ad_2%250Ae_2%250Af_2%250At_2%250Ax_2%250Ay_2%250Az_2%250Aa_3%250Ab_3%250Ac_3%250Ad_3%250Ae_3%250Af_3%250At_3%250Ax_3%250Ay_3%250Az_3&v6=x_1%250Ax_2%250Ax_3%250Ax_5
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one_1() {
        let input = "2, 0, 0 @ -1,  1, -2\n\
                     0, 0, 0 @ 1, 1, -2";
        let result = count_intersections(input, (0.)..=3.);
        assert_eq!(result, 1);
    }

    #[test]
    fn test_part_one() {
        let input = &advent_of_code::template::read_file("examples", DAY);
        let result = count_intersections(input, (7.)..=27.);
        assert_eq!(result, 2);
    }

    #[test]
    fn test_part_two() {
        let result =
            part_two(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, None);
    }
}

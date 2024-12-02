mod day;
pub mod template;

use std::ops::Range;

pub use day::*;

use itertools::Itertools;
use nom::{
    bytes::complete::{tag, take_until},
    character::complete::multispace0,
    error::ParseError,
    sequence::{delimited, preceded},
    Parser,
};
use strum::{EnumIter, IntoEnumIterator};
use tinyvec::TinyVec;

pub type TV4<K> = TinyVec<[K; 4]>;

/// A combinator that takes a parser `inner` and produces a parser that also consumes both leading and
/// trailing whitespace, returning the output of `inner`.
pub fn ws<'a, F, O, E: ParseError<&'a str>>(
    inner: F,
) -> impl Parser<&'a str, O, E>
where
    F: Parser<&'a str, O, E>,
{
    delimited(multispace0, inner, multispace0)
}

pub fn take_until_inclusive<'a, E: ParseError<&'a str>>(
    tag_str: &'static str,
) -> impl Parser<&'a str, &'a str, E> {
    preceded(take_until(tag_str), tag(tag_str))
}

// Get the intersection between the two ranges if there are any
pub fn intersection<T>(r1: &Range<T>, r2: &Range<T>) -> Option<Range<T>>
where
    T: PartialOrd + Copy,
{
    let max_start = if r1.start > r2.start {
        r1.start
    } else {
        r2.start
    };

    let min_end = if r1.end < r2.end { r1.end } else { r2.end };

    (max_start < min_end).then(|| max_start..min_end)
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, EnumIter)]
pub enum Compass {
    #[default]
    N,
    E,
    S,
    W,
}

impl Compass {
    pub fn from_relative(relative: char) -> Option<Self> {
        match relative {
            'U' => Some(Compass::N),
            'D' => Some(Compass::S),
            'L' => Some(Compass::W),
            'R' => Some(Compass::E),
            _ => None,
        }
    }

    pub fn opposite(&self) -> Self {
        use Compass as D;

        match self {
            D::E => D::W,
            D::N => D::S,
            D::W => D::E,
            D::S => D::N,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Grid<T> {
    pub width: usize,
    pub height: usize,
    pub data: Vec<T>,
}

impl<T> Grid<T>
where
    T: From<char>,
{
    pub fn parse_lines(input: &str) -> Self {
        let width = input.find('\n').unwrap_or(input.len());

        // Double check all the lines match the expected width
        debug_assert!(!input.lines().any(|line| line.len() != width));

        let data = input
            .chars()
            .filter(|c| *c != '\n')
            .map(T::from)
            .collect_vec();

        Self::new(data, width)
    }
}

impl<T> Grid<T>
where
    char: From<T>,
    T: Copy,
{
    pub fn print(&self) {
        self.rows().for_each(|row| {
            println!(
                "{}",
                row.iter().map(|t| char::from(*t)).collect::<String>()
            )
        });
    }
}

impl<T> Grid<T> {
    pub fn new(data: Vec<T>, width: usize) -> Self {
        let height = data.len() / width;
        Self {
            data,
            width,
            height,
        }
    }

    pub fn rows(&self) -> impl DoubleEndedIterator<Item = &'_ [T]> + '_ {
        self.data.chunks(self.width)
    }

    pub fn rows_mut(
        &mut self,
    ) -> impl DoubleEndedIterator<Item = &'_ mut [T]> + '_ {
        self.data.chunks_mut(self.width)
    }

    pub fn for_row_pairs_mut<F>(&mut self, mut action: F)
    where
        F: FnMut(&'_ mut [T], &'_ mut [T]),
    {
        let mut iter = self.rows_mut();
        let mut prev = iter.next().unwrap();

        for next in iter {
            action(prev, next);
            prev = next;
        }
    }

    pub fn at_index(&self, i: usize) -> Option<&'_ T> {
        self.data.get(i)
    }

    #[allow(clippy::unnecessary_lazy_evaluations)]
    pub fn step_from_index(&self, i: usize, dir: Compass) -> Option<usize> {
        use Compass as D;

        match dir {
            D::E => (i % self.width < (self.width - 1)).then_some(i + 1),
            D::W => (i % self.width > 0).then(|| i - 1),
            D::N => i.checked_sub(self.width),
            D::S => Some(i + self.width).filter(|j| *j < self.data.len()),
        }
    }

    pub fn neighbors(
        &self,
        i: usize,
    ) -> impl Iterator<Item = (Compass, usize)> + '_ {
        Compass::iter().filter_map(move |dir| {
            self.step_from_index(i, dir).map(move |j| (dir, j))
        })
    }

    pub fn escaping(&self, i: usize) -> impl Iterator<Item = Compass> + '_ {
        Compass::iter()
            .filter(move |dir| self.step_from_index(i, *dir).is_some())
    }

    pub fn min_dist(&self, from: usize, to: usize) -> usize {
        let col_dist = (from % self.width).abs_diff(to % self.width);
        let row_dist = (from / self.width).abs_diff(to / self.width);

        col_dist + row_dist
    }

    pub fn is_border(&self, i: usize) -> bool {
        let col = i % self.width;
        let row = i / self.width;

        row == 0 || col == 0 || row == self.height - 1 || col == self.width - 1
    }

    pub fn border(&self) -> impl Iterator<Item = usize> + '_ {
        (0..self.width)
            .chain((self.data.len() - self.width)..self.data.len())
            .chain((0..self.height).map(|i| i * self.width))
            .chain((0..self.height).map(|i| (i + 1) * self.width - 1))
    }

    pub fn are_neighbors(&self, i1: usize, i2: usize) -> bool {
        let (r1, c1) = (i1 / self.width, i1 % self.width);
        let (r2, c2) = (i2 / self.width, i2 % self.width);

        r1.abs_diff(r2) + c1.abs_diff(c2) == 1
    }

    pub fn corners(&self) -> impl Iterator<Item = usize> {
        [
            0,
            self.width - 1,
            self.data.len() - 1,
            self.data.len() - self.width,
        ]
        .into_iter()
    }

    pub fn to_col_row(&self, i: usize) -> (i32, i32) {
        ((i % self.width) as i32, (i / self.width) as i32)
    }

    pub fn to_index(&self, col: i32, row: i32) -> usize {
        col as usize + row as usize * self.width
    }
}

impl<T> Grid<T>
where
    T: Copy,
{
    pub fn quadruple(self) -> Self {
        let mut data = Vec::with_capacity(self.data.len() * 4);
        let width = self.width * 2;
        let height = self.height * 2;

        for _ in 0..2 {
            for row in 0..self.height {
                for _ in 0..2 {
                    data.extend(
                        self.data[(row * self.width)..((row + 1) * self.width)]
                            .iter()
                            .copied(),
                    );
                }
            }
        }

        Self {
            data,
            height,
            width,
        }
    }
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Compass {
    N,
    E,
    S,
    W,
}

#[derive(Debug, PartialEq, Eq)]
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

    pub fn step_from_index(&self, i: usize, dir: Compass) -> Option<usize> {
        use Compass as D;

        match dir {
            D::E => (i % self.width < (self.width - 1)).then_some(i + 1),
            D::W => (i % self.width > 0).then_some(i - 1),
            D::N => i.checked_sub(self.width),
            D::S => Some(i + self.width).filter(|j| *j < self.data.len()),
        }
    }
}

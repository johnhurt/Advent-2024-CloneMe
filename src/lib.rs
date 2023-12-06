mod day;
pub mod template;

use std::ops::Range;

pub use day::*;

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

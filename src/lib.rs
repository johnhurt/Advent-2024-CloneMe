mod day;
pub mod template;

pub use day::*;

use nom::{
    character::complete::multispace0, error::ParseError, sequence::delimited,
    IResult, Parser,
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

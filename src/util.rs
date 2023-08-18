use crate::parse::{Parse, ParseResult};

pub fn match_literal<'a>(expected: &'static str) -> impl Parse<'a, ()> {
    move |input: &'a str| match input.get(0..expected.len()) {
        Some(prefix) if prefix == expected => Ok((&input[expected.len()..], ())),
        _ => Err(input),
    }
}

struct PrefixParser<'a>(pub &'a str);

impl<'a> Parse<'a, ()> for PrefixParser<'a> {
    fn parse(&self, data: &'a str) -> ParseResult<'a, ()> {
        data.strip_prefix(self.0).map(|rest| (rest, ())).ok_or(data)
    }
}

struct NumberParser;

impl<'a> Parse<'a, usize> for NumberParser {
    fn parse(&self, data: &'a str) -> ParseResult<'a, usize> {
        let non_num = data.find(|c: char| !c.is_numeric());
        let (num, rest) = data.split_at(non_num.unwrap_or(data.len()));
        num.parse().map(|x| (rest, x)).map_err(|_| data)
    }
}

#[derive(Debug)]
pub(crate) struct RangeQuantifier {
    lower: usize,
    upper: usize,
}

impl RangeQuantifier {
    pub fn new(lower: usize, upper: usize) -> Self {
        Self { lower, upper }
    }
}

pub(crate) fn parse_range_quantifier(data: &str) -> ParseResult<'_, RangeQuantifier> {
    PrefixParser("{")
        .parse(data)
        .and_then(|(rest, _)| NumberParser {}.parse(rest))
        .map(|(rest, lower)| (rest, RangeQuantifier::new(lower, 0)))
}
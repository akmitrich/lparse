use crate::{
    combinators::{left, map, one_or_more, pred, right, zero_or_more},
    parse::{Parse, ParseResult},
};

pub fn match_literal<'a>(expected: &'static str) -> impl Parse<'a, ()> {
    move |input: &'a str| match input.get(0..expected.len()) {
        Some(prefix) if prefix == expected => Ok((&input[expected.len()..], ())),
        _ => Err(input),
    }
}

pub fn any_char(input: &str) -> ParseResult<char> {
    match input.chars().next() {
        Some(ch) => Ok((&input[ch.len_utf8()..], ch)),
        _ => Err(input),
    }
}

pub fn whitespace_char<'a>() -> impl Parse<'a, char> {
    pred(any_char, |c| c.is_whitespace())
}

pub fn whitespaces_any<'a>() -> impl Parse<'a, Vec<char>> {
    zero_or_more(whitespace_char())
}

pub fn whitespaces_exact<'a>() -> impl Parse<'a, Vec<char>> {
    one_or_more(whitespace_char())
}

struct PrefixParser<'a>(pub &'a str);

impl<'a> Parse<'a, ()> for PrefixParser<'a> {
    fn parse(&self, data: &'a str) -> ParseResult<'a, ()> {
        data.strip_prefix(self.0).map(|rest| (rest, ())).ok_or(data)
    }
}

pub fn quoted_string<'a>() -> impl Parse<'a, String> {
    map(
        right(
            match_literal("\""),
            left(
                zero_or_more(pred(any_char, |c| *c != '"')),
                match_literal("\""),
            ),
        ),
        |chars| chars.into_iter().collect(),
    )
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

pub fn identifier(input: &str) -> ParseResult<String> {
    let mut matched = String::new();
    let mut chars = input.chars();

    match chars.next() {
        Some(next) if next.is_alphabetic() => matched.push(next),
        _ => return Err(input),
    }

    for ch in chars {
        if ch.is_alphanumeric() || ch == '-' {
            matched.push(ch);
        } else {
            break;
        }
    }

    let next_index = matched.len();
    Ok((&input[next_index..], matched))
}

#[cfg(test)]
mod test_xml {
    use super::*;
    #[test]
    fn right_combinator() {
        let tag_opener = right(match_literal("<"), identifier);
        assert_eq!(
            Ok(("/>", "my-first-element".to_string())),
            tag_opener.parse("<my-first-element/>")
        );
        assert_eq!(Err("oops"), tag_opener.parse("oops"));
        assert_eq!(Err("!oops"), tag_opener.parse("<!oops"));
    }

    #[test]
    fn one_or_more_combinator() {
        let parser = one_or_more(match_literal("ha"));
        assert_eq!(Ok(("", vec![(), (), ()])), parser.parse("hahaha"));
        assert_eq!(Err("ahah"), parser.parse("ahah"));
        assert_eq!(Err(""), parser.parse(""));
    }

    #[test]
    fn zero_or_more_combinator() {
        let parser = zero_or_more(match_literal("ha"));
        assert_eq!(Ok(("", vec![(), (), ()])), parser.parse("hahaha"));
        assert_eq!(Ok(("ahah", vec![])), parser.parse("ahah"));
        assert_eq!(Ok(("", vec![])), parser.parse(""));
    }

    #[test]
    fn predicate_combinator() {
        let parser = pred(any_char, |ch| *ch == 'o');
        assert_eq!(Ok(("mg", 'o')), parser.parse("omg"));
        assert_eq!(Err("lol"), parser.parse("lol"));
    }

    #[test]
    fn quoted_string_parser() {
        assert_eq!(
            Ok(("", "Hello Joe!".to_string())),
            quoted_string().parse("\"Hello Joe!\"")
        );
    }
}

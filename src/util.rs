use crate::{
    combinators::{filter, left, map, one_or_more, right, zero_or_more},
    parse::{Parse, ParseResult},
};

pub fn match_literal<'a>(expected: &'static str) -> impl Parse<'a, ()> {
    move |input: &'a str| match input.get(0..expected.len()) {
        Some(prefix) if prefix == expected => Ok((&input[expected.len()..], ())),
        _ => Err(input),
    }
}

pub fn match_string<'a>(expected: String) -> impl Parse<'a, ()> {
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

pub fn digit<'a>() -> impl Parse<'a, char> {
    filter(any_char, |ch| ch.is_numeric())
}

pub fn whitespace_char<'a>() -> impl Parse<'a, char> {
    filter(any_char, |c| c.is_whitespace())
}

pub fn space_some<'a>() -> impl Parse<'a, Vec<char>> {
    one_or_more(whitespace_char())
}

pub fn spaces_any<'a>() -> impl Parse<'a, Vec<char>> {
    zero_or_more(whitespace_char())
}

pub fn whitespace_wrap<'a, P, A>(parser: P) -> impl Parse<'a, A>
where
    P: Parse<'a, A>,
{
    right(spaces_any(), left(parser, spaces_any()))
}

pub fn quoted_string<'a>() -> impl Parse<'a, String> {
    map(
        right(
            match_literal("\""),
            left(
                zero_or_more(filter(any_char, |c| *c != '"')),
                match_literal("\""),
            ),
        ),
        |chars| chars.into_iter().collect(),
    )
}

pub fn number<'a>() -> impl Parse<'a, u64> {
    map(one_or_more(digit()), |x| {
        x.into_iter().collect::<String>().parse().unwrap()
    })
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
        let parser = filter(any_char, |ch| *ch == 'o');
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

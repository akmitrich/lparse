use std::ops::{Range, RangeBounds};

use crate::parse::Parse;

pub fn map<'a, P, F, A, B>(parser: P, map_fn: F) -> impl Parse<'a, B>
where
    P: Parse<'a, A>,
    F: Fn(A) -> B,
{
    move |input| {
        parser
            .parse(input)
            .map(|(parsed, result)| (parsed, map_fn(result)))
    }
}

pub fn pair<'a, P1, P2, R1, R2>(parser1: P1, parser2: P2) -> impl Parse<'a, (R1, R2)>
where
    P1: Parse<'a, R1>,
    P2: Parse<'a, R2>,
{
    move |input| {
        parser1.parse(input).and_then(|(intermediate, result1)| {
            parser2
                .parse(intermediate)
                .map(|(eventual, result2)| (eventual, (result1, result2)))
        })
    }
}

pub fn left<'a, P1, P2, R1, R2>(parser1: P1, parser2: P2) -> impl Parse<'a, R1>
where
    P1: Parse<'a, R1>,
    P2: Parse<'a, R2>,
{
    map(pair(parser1, parser2), |(left, _right)| left)
}

pub fn right<'a, P1, P2, R1, R2>(parser1: P1, parser2: P2) -> impl Parse<'a, R2>
where
    P1: Parse<'a, R1>,
    P2: Parse<'a, R2>,
{
    map(pair(parser1, parser2), |(_left, right)| right)
}

pub fn repeat<'a, T>(
    parser: impl Parse<'a, T>,
    range: impl RangeBounds<usize>,
) -> impl Parse<'a, Vec<T>> {
    fn helper<'h, T>(result: &mut Vec<T>, rest: &'h str, item: T) -> &'h str {
        result.push(item);
        rest
    }

    fn parse_in_vec<'h, P, T>(
        parser: &P,
        input: &'h str,
        result: &mut Vec<T>,
        bounded_range: Range<usize>,
    ) -> Result<&'h str, &'h str>
    where
        P: Parse<'h, T>,
    {
        let mut rest = input;
        for _ in bounded_range {
            if let Ok((next_input, item)) = parser.parse(rest) {
                rest = helper(result, next_input, item);
            } else {
                return Err(rest);
            }
        }
        Ok(rest)
    }

    move |input| {
        let mut result = Vec::new();
        let start_bound = match range.start_bound() {
            std::ops::Bound::Included(s) => *s,
            std::ops::Bound::Excluded(s) => s + 1,
            std::ops::Bound::Unbounded => 0,
        };
        let mut rest =
            parse_in_vec(&parser, input, &mut result, 0..start_bound).map_err(|_| input)?;
        if let std::ops::Bound::Unbounded = range.end_bound() {
            while let Ok((next_input, item)) = parser.parse(rest) {
                rest = helper(&mut result, next_input, item);
            }
        } else {
            let end_bound = match range.end_bound() {
                std::ops::Bound::Included(e) => e + 1, //I don't believe in end_bound=usize::MAX
                std::ops::Bound::Excluded(e) => *e,
                std::ops::Bound::Unbounded => unreachable!(),
            };
            rest = parse_in_vec(&parser, rest, &mut result, start_bound + 1..end_bound)
                .unwrap_or(rest);
        }
        Ok((rest, result))
    }
}

pub fn one_or_more<'a, P, A>(parser: P) -> impl Parse<'a, Vec<A>>
where
    P: Parse<'a, A>,
{
    repeat(parser, 1..)
}

pub fn zero_or_more<'a, P, A>(parser: P) -> impl Parse<'a, Vec<A>>
where
    P: Parse<'a, A>,
{
    repeat(parser, 0..)
}

pub fn filter<'a, P, A, F>(parser: P, predicate: F) -> impl Parse<'a, A>
where
    P: Parse<'a, A>,
    F: Fn(&A) -> bool,
{
    move |input| {
        if let Ok((next_input, value)) = parser.parse(input) {
            if predicate(&value) {
                return Ok((next_input, value));
            }
        }
        Err(input)
    }
}

pub fn either<'a, P1, P2, A>(parser1: P1, parser2: P2) -> impl Parse<'a, A>
where
    P1: Parse<'a, A>,
    P2: Parse<'a, A>,
{
    move |input| match parser1.parse(input) {
        ok @ Ok(_) => ok,
        Err(_) => parser2.parse(input),
    }
}

pub fn and_then<'a, P, F, A, B, NextP>(parser: P, f: F) -> impl Parse<'a, B>
where
    P: Parse<'a, A>,
    NextP: Parse<'a, B>,
    F: Fn(A) -> NextP,
{
    move |input| match parser.parse(input) {
        Ok((next_input, result)) => f(result).parse(next_input),
        Err(err) => Err(err),
    }
}

#[cfg(test)]
mod test_xml {
    use super::*;

    fn identifier(input: &str) -> Result<(&str, String), &str> {
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

    #[test]
    fn right_combinator() {
        let tag_opener = right(crate::util::match_literal("<"), identifier);
        assert_eq!(
            Ok(("/>", "my-first-element".to_string())),
            tag_opener.parse("<my-first-element/>")
        );
        assert_eq!(Err("oops"), tag_opener.parse("oops"));
        assert_eq!(Err("!oops"), tag_opener.parse("<!oops"));
    }

    #[test]
    fn one_or_more_combinator() {
        let parser = one_or_more(crate::util::match_literal("ha"));
        assert_eq!(Ok(("", vec![(), (), ()])), parser.parse("hahaha"));
        assert_eq!(Err("ahah"), parser.parse("ahah"));
        assert_eq!(Err(""), parser.parse(""));
    }

    #[test]
    fn zero_or_more_combinator() {
        let parser = zero_or_more(crate::util::match_literal("ha"));
        assert_eq!(Ok(("", vec![(), (), ()])), parser.parse("hahaha"));
        assert_eq!(Ok(("ahah", vec![])), parser.parse("ahah"));
        assert_eq!(Ok(("", vec![])), parser.parse(""));
    }

    #[test]
    fn repeat_combinator_5_6() {
        let p = repeat(crate::util::match_literal("a"), 5..=6);
        assert_eq!(Ok(("", vec![(); 5])), p.parse("aaaaa"));
        assert_eq!(Ok(("", vec![(); 6])), p.parse("aaaaaa"));
        assert_eq!(Ok(("Trail", vec![(); 5])), p.parse("aaaaaTrail"));
        assert_eq!(Ok(("Trail", vec![(); 6])), p.parse("aaaaaaTrail"));
        assert_eq!(Err("aaaaTrail"), p.parse("aaaaTrail"));
        assert_eq!(Err("Trail"), p.parse("Trail"));
        assert_eq!(Err(""), p.parse(""));
    }

    #[test]
    fn predicate_combinator() {
        let parser = filter(crate::util::any_char, |c| *c == 'o');
        assert_eq!(Ok(("mg", 'o')), parser.parse("omg"));
        assert_eq!(Err("lol"), parser.parse("lol"));
    }
}

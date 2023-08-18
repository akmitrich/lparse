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

pub fn one_or_more<'a, P, A>(parser: P) -> impl Parse<'a, Vec<A>>
where
    P: Parse<'a, A>,
{
    move |input| {
        let mut result = Vec::new();
        let mut rest = input;

        if let Ok((next_input, first_item)) = parser.parse(rest) {
            rest = next_input;
            result.push(first_item);
        } else {
            return Err(rest);
        }

        while let Ok((next_input, next_item)) = parser.parse(rest) {
            rest = next_input;
            result.push(next_item);
        }
        Ok((rest, result))
    }
}

pub fn zero_or_more<'a, P, A>(parser: P) -> impl Parse<'a, Vec<A>>
where
    P: Parse<'a, A>,
{
    move |input| {
        let mut result = Vec::new();
        let mut rest = input;
        while let Ok((next_input, next_item)) = parser.parse(rest) {
            rest = next_input;
            result.push(next_item);
        }
        Ok((rest, result))
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
}

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

pub fn pred<'a, P, A, F>(parser: P, predicate: F) -> impl Parse<'a, A>
where
    P: Parse<'a, A>,
    F: Fn(&A) -> bool,
{
    move |input| {
        if let Ok((rest, value)) = parser.parse(input) {
            if predicate(&value) {
                return Ok((rest, value));
            }
        }
        Err(input)
    }
}

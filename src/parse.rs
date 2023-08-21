use crate::combinators;

pub type ParseResult<'a, Output> = Result<(&'a str, Output), &'a str>;

pub trait Parse<'a, T> {
    fn parse(&self, data: &'a str) -> ParseResult<'a, T>;

    fn map<F, NewOutput>(self, map_fn: F) -> BoxedParser<'a, NewOutput>
    where
        Self: Sized + 'a,
        T: 'a,
        NewOutput: 'a,
        F: Fn(T) -> NewOutput + 'a,
    {
        BoxedParser::new(combinators::map(self, map_fn))
    }

    fn filter<F>(self, pred_fn: F) -> BoxedParser<'a, T>
    where
        Self: Sized + 'a,
        T: 'a,
        F: Fn(&T) -> bool + 'a,
    {
        BoxedParser::new(combinators::filter(self, pred_fn))
    }

    fn and_then<F, NextParser, NewOutput>(self, f: F) -> BoxedParser<'a, NewOutput>
    where
        Self: Sized + 'a,
        T: 'a,
        NewOutput: 'a,
        NextParser: Parse<'a, NewOutput> + 'a,
        F: Fn(T) -> NextParser + 'a,
    {
        BoxedParser::new(combinators::and_then(self, f))
    }
}

impl<'a, F, Output> Parse<'a, Output> for F
where
    F: Fn(&'a str) -> ParseResult<Output>,
{
    fn parse(&self, input: &'a str) -> ParseResult<'a, Output> {
        self(input)
    }
}

pub struct BoxedParser<'a, Output> {
    parser: Box<dyn Parse<'a, Output> + 'a>,
}

impl<'a, Output> BoxedParser<'a, Output> {
    pub fn new<P>(parser: P) -> Self
    where
        P: Parse<'a, Output> + 'a,
    {
        BoxedParser {
            parser: Box::new(parser),
        }
    }
}

impl<'a, Output> Parse<'a, Output> for BoxedParser<'a, Output> {
    fn parse(&self, input: &'a str) -> ParseResult<'a, Output> {
        self.parser.parse(input)
    }
}

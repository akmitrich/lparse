pub type ParseResult<'a, Output> = Result<(&'a str, Output), &'a str>;

pub trait Parse<'a, T> {
    fn parse(&self, data: &'a str) -> ParseResult<'a, T>;
}

impl<'a, F, Output> Parse<'a, Output> for F
where
    F: Fn(&'a str) -> ParseResult<Output>,
{
    fn parse(&self, input: &'a str) -> ParseResult<'a, Output> {
        self(input)
    }
}

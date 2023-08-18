fn main() {
    println!("Hello, world! {:?}", parse_range_quantifier("{444,567}"));
}

trait Parse<T> {
    fn parse<'a>(&self, data: &'a str) -> Option<(T, &'a str)>;
}

struct PrefixParser<'a>(pub &'a str);

impl<'a> Parse<()> for PrefixParser<'a> {
    fn parse<'b>(&self, data: &'b str) -> Option<((), &'b str)> {
        data.strip_prefix(self.0).map(|rest| ((), rest))
    }
}

struct NumberParser;

impl Parse<usize> for NumberParser {
    fn parse<'a>(&self, data: &'a str) -> Option<(usize, &'a str)> {
        let non_num = data.find(|c: char| !c.is_numeric());
        let (num, rest) = data.split_at(non_num.unwrap_or(data.len()));
        Some((num.parse().ok()?, rest))
    }
}

#[derive(Debug)]
struct RangeQuantifier {
    lower: usize,
    upper: usize,
}

impl RangeQuantifier {
    pub fn new(lower: usize, upper: usize) -> Self {
        Self { lower, upper }
    }
}

fn parse_range_quantifier(data: &str) -> Option<(RangeQuantifier, &str)> {
    PrefixParser("{")
        .parse(data)
        .and_then(|(_, rest)| NumberParser {}.parse(rest))
        .map(|(lower, rest)| (RangeQuantifier::new(lower, 0), rest))
}

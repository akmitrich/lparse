pub mod combinators;
pub mod parse;
pub mod util;

fn main() {
    println!(
        "Hello, world! {:?}",
        util::parse_range_quantifier("{444,567}")
    );
}

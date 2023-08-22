use crate::parse::Parse;

pub mod combinators;
pub mod parse;
pub mod re;
pub mod util;
pub mod xml;

fn main() {
    println!("Parsed: {:?}", re::range_quantifier().parse("{75}"));
}

fn _xml_example() {
    let now = std::time::Instant::now();
    let elem = xml::element().parse(
        r#"
            <top label="Top">
                <semi-bottom label="Bottom"/>
                <middle>
                    <bottom label="Another bottom"/>
                </middle>
            </top>"#,
    );
    let elapsed = std::time::Instant::now().duration_since(now);
    let element = elem.unwrap();
    println!("Hello, world! {:?}\n{:#?}", element.0, element.1);
    println!("Time: {:?}", elapsed);
}

use crate::{combinators, parse::Parse, util};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Element {
    pub name: String,
    pub attributes: Vec<(String, String)>,
    pub children: Vec<Element>,
}

fn parent_element<'a>() -> impl Parse<'a, Element> {
    open_element().and_then(|elem| {
        combinators::map(
            combinators::left(
                combinators::zero_or_more(element()),
                close_element(elem.name.clone()),
            ),
            move |children| {
                let mut elem = elem.clone();
                elem.children = children;
                elem
            },
        )
    })
}

fn close_element<'a>(expected_name: String) -> impl Parse<'a, String> {
    combinators::filter(
        combinators::right(
            util::match_literal("</"),
            combinators::left(identifier, util::match_literal(">")),
        ),
        move |name| name == &expected_name,
    )
}

pub fn element<'a>() -> impl Parse<'a, Element> {
    util::whitespace_wrap(combinators::either(single_element(), parent_element()))
}

fn open_element<'a>() -> impl Parse<'a, Element> {
    combinators::map(
        combinators::left(element_start(), util::match_literal(">")),
        |(name, attributes)| Element {
            name,
            attributes,
            children: vec![],
        },
    )
}

fn single_element<'a>() -> impl Parse<'a, Element> {
    combinators::map(
        combinators::left(element_start(), util::match_literal("/>")),
        |(name, attributes)| Element {
            name,
            attributes,
            children: vec![],
        },
    )
}

fn element_start<'a>() -> impl Parse<'a, (String, Vec<(String, String)>)> {
    combinators::right(
        util::match_literal("<"),
        combinators::pair(identifier, attributes()),
    )
}

fn attributes<'a>() -> impl Parse<'a, Vec<(String, String)>> {
    combinators::zero_or_more(combinators::right(util::space_some(), attribute_pair()))
}

fn attribute_pair<'a>() -> impl Parse<'a, (String, String)> {
    combinators::pair(
        identifier,
        combinators::right(util::match_literal("="), quoted_string()),
    )
}

fn quoted_string<'a>() -> impl Parse<'a, String> {
    combinators::map(
        combinators::right(
            util::match_literal("\""),
            combinators::left(
                combinators::zero_or_more(combinators::filter(util::any_char, |c| *c != '"')),
                util::match_literal("\""),
            ),
        ),
        |chars| chars.into_iter().collect(),
    )
}

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

use crate::{
    combinators::{left, map, optional, pair, right},
    parse::Parse,
    util::{match_literal, number},
};

enum Bound {
    None,
    Upper(usize),
    Unbounded,
}

#[derive(Debug, PartialEq)]
pub struct RangeQuantifier {
    lower: usize,
    upper: Option<usize>,
}

impl RangeQuantifier {
    pub fn new(lower: usize, upper: Option<usize>) -> Self {
        Self { lower, upper }
    }
}

pub fn range_quantifier<'a>() -> impl Parse<'a, RangeQuantifier> {
    map(
        left(
            pair(right(match_literal(r"{"), number()), upper_bound()),
            match_literal(r"}"),
        ),
        |(l, u)| {
            RangeQuantifier::new(
                l as _,
                match u {
                    Bound::None => Some(l as _),
                    Bound::Upper(u) => Some(u),
                    Bound::Unbounded => None,
                },
            )
        },
    )
}

fn upper_bound<'a>() -> impl Parse<'a, Bound> {
    map(
        pair(optional(match_literal(",")), optional(number())),
        |x| match x {
            (None, _) => Bound::None,
            (Some(_), None) => Bound::Unbounded,
            (Some(_), Some(number)) => Bound::Upper(number as _),
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trivial() {
        let p = range_quantifier();
        let illegal = "mhgsdkjafg";
        assert_eq!(Err(illegal), p.parse(illegal));
    }

    #[test]
    fn dangling_curly_brace() {
        let p = range_quantifier();
        let dangling = r"{54,88";
        assert_eq!(Err(""), p.parse(dangling));
    }

    #[test]
    fn space_after_comma() {
        let p = range_quantifier();
        assert_eq!(Err(" 88}"), p.parse(r"{54, 88}"));
    }

    #[test]
    fn exact_number() {
        let p = range_quantifier();
        assert_eq!(
            Ok(("", RangeQuantifier::new(75, Some(75)))),
            p.parse(r"{75}")
        )
    }

    #[test]
    fn unbounded_range() {
        let p = range_quantifier();
        assert_eq!(Ok(("", RangeQuantifier::new(34, None))), p.parse(r"{34,}"))
    }

    #[test]
    fn exact_range() {
        let p = range_quantifier();
        assert_eq!(Ok(("", RangeQuantifier::new(1, Some(3)))), p.parse(r"{1,3}"))
    }
}

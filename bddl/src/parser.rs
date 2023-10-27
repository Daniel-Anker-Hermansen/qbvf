use nom::{IResult, branch::alt, character::complete::{one_of, char, space0}, combinator::map_res, bytes::complete::{take_while1, tag}, sequence::tuple, multi::many0};

use crate::bddl::{E, Pred, SubCondition, Condition};

fn e_offset(offset: char, tag: impl Fn(i64) -> E) -> impl Fn(&str) -> IResult<&str, E> {
    move |input| tuple((
        char('?'),
        one_of("xy"),
        space0,
        char(offset),
        space0,
        map_res(take_while1(|c: char| c.is_digit(10)), |v| i64::from_str_radix(v, 10)),
    ))(input).map(|(input, (.., v))| (input, tag(v)))
}

fn e_identity(input: &str) -> IResult<&str, E> {
    tuple((
        char('?'),
        one_of("xy"),
    ))(input).map(|(input, _)| (input, E::Identity))
}

fn e_max(input: &str) -> IResult<&str, E> {
    tuple((
        one_of("xy"),
        tag("max"),
    ))(input).map(|(input, _)| (input, E::Max))
}

fn e_min(input: &str) -> IResult<&str, E> {
    tuple((
        one_of("xy"),
        tag("min"),
    ))(input).map(|(input, _)| (input, E::Min))
}

fn e(input: &str) -> IResult<&str, E> {
    alt((
        e_offset('+', E::Add), 
        e_offset('-', E::Sub),
        e_identity,
        e_max,
        e_min,
    ))(input)
}

fn pred(input: &str) -> IResult<&str, Pred> {
    alt((
        |input| tag("open")(input).map(|(input, _)| (input, Pred::Open)),
        |input| tag("black")(input).map(|(input, _)| (input, Pred::Black)),
        |input| tag("white")(input).map(|(input, _)| (input, Pred::White)),
    ))(input)
}

fn sub_cond_pred(input: &str) -> IResult<&str, (Pred, E, E)> {
    tuple((
        pred,
        char('('),
        e,
        char(','),
        space0,
        e,
        char(')'),
    ))(input).map(|(input, (p, _, e1, _, _, e2, ..))| (input, (p, e1, e2)))
}

fn not_sub_cond_pred(input: &str) -> IResult<&str, SubCondition> {
    tuple((
        tag("NOT("),
        sub_cond_pred,
        char(')')
    ))(input).map(|(input, (_, p, _))| (input, SubCondition::Not { pred: p.0, x_e: p.1, y_e: p.2 }))
}

fn id_sub_cond_pred(input: &str) -> IResult<&str, SubCondition> {
    sub_cond_pred(input).map(|(input, p)| (input, SubCondition::Id { pred: p.0, x_e: p.1, y_e: p.2 }))
}

fn sub_cond(input: &str) -> IResult<&str, SubCondition> {
    tuple((
        space0,
        alt((
            id_sub_cond_pred,
            not_sub_cond_pred,
        )),
        space0,
    ))(input).map(|(input, (_, v, _))| (input, v))
}

pub fn condition(input: &str) -> IResult<&str, Condition> {
    many0(sub_cond)(input).map(|(input, sub_cond)| (input, Condition { sub_cond }))
}

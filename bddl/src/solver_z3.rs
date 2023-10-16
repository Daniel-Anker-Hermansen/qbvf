use std::iter::repeat;

use z3::{Context, ast::{BV, Bool, Ast}};

use crate::bddl::{InitPred, Pred, SubCondition, Size, Condition};

type Pieces<'ctx> = [(BV<'ctx>, BV<'ctx>)];

fn gen_contains<'ctx>(ctx: &'ctx Context, pieces: &Pieces<'ctx>, x: &BV<'ctx>, y: &BV<'ctx>) -> Bool<'ctx> {
    let terms: Vec<_> = pieces.iter().map(|(x_, y_)| Bool::and(ctx, &[&x_._eq(x), &y_._eq(y)])).collect();
    let terms: Vec<_> = terms.iter().collect();
    Bool::or(ctx, &terms)
}

fn gen_pred<'ctx>(ctx: &'ctx Context, whites: &Pieces<'ctx>, blacks: &Pieces<'ctx>, pred: Pred, x: &BV<'ctx>, y: &BV<'ctx>) -> Bool<'ctx> {
    match pred {
        Pred::Open => Bool::or(ctx, &[&gen_contains(ctx, whites, x, y), &gen_contains(ctx, blacks, x, y)]).not(),
        Pred::White => gen_contains(ctx, whites, x, y),
        Pred::Black => gen_contains(ctx, blacks, x, y),
    }
}

// Returns none if it causes out of bounds
fn gen_subcondition<'ctx>(ctx: &'ctx Context, whites: &Pieces<'ctx>, blacks: &Pieces<'ctx>, sub_condition: &SubCondition, size: Size, x: i64, y: i64) -> Option<Bool<'ctx>> {
    match *sub_condition {
        SubCondition::Id { pred, x_e, y_e } => {
            let x = x_e.noramlize(x, size.x);
            let y = y_e.noramlize(y, size.y);
            if x < 0 || x >= size.x || y < 0 || y >= size.y {
                None
            }
            else {
                let x = BV::from_i64(ctx, x, 32);
                let y = BV::from_i64(ctx, y, 32);
                Some(gen_pred(ctx, whites, blacks, pred, &x, &y))
            }
        },
        SubCondition::Not { pred, x_e, y_e } => gen_subcondition(ctx, whites, blacks, &SubCondition::Id { pred, x_e, y_e }, size,x , y).map(|b| b.not()),
    }
}

fn gen_condition<'ctx>(ctx: &'ctx Context, whites: &Pieces<'ctx>, blacks: &Pieces<'ctx>, condition: &Condition, size: Size, x: i64, y: i64) -> Bool<'ctx> {
    let sub_conditions: Vec<_> = condition.sub_cond.iter()
        .filter_map(|sub_condition| gen_subcondition(ctx, whites, blacks, sub_condition, size, x, y))
        .collect();
    Bool::and(ctx, &sub_conditions.iter().collect::<Vec<_>>())
}

fn gen_goals<'ctx>(ctx: &'ctx Context, whites: &Pieces<'ctx>, blacks: &Pieces<'ctx>, conditions: &[Condition], size: Size) -> Bool<'ctx> {
    let ors: Vec<_> = (0..size.x).flat_map(|x| repeat(x).zip(0..size.y))
        .flat_map(|(x, y)| {
            conditions.iter()
                .map(move |condition| gen_condition(ctx, whites, blacks, condition, size, x, y))
        })
        .collect();
        Bool::or(ctx, &ors.iter().collect::<Vec<_>>())
}

fn effect_action<'ctx>(ctx: &'ctx Context, whites: &Pieces<'ctx>, blacks: &Pieces<'ctx>) {
    // We can get into a situation where we need to set an existing value to something else, this
    // requires that we remove it from the list but we do not know which it is because it is
    // symbolic so we cannot use lists for games which allow this. I have to think about how to fix
    // this. Maybe use function shadowing although i am worried about performace regarding that.
    todo!()
}

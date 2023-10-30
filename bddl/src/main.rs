//use std::iter::{repeat, once};

//use z3::{Context, ast::{Bool, BV, Ast, exists_const, forall_const}, Tactic};

//use crate::bddl::Problem;

use z3::{Context, Tactic};

mod bddl;
mod tictac;
mod solver;
mod solver_z3;
mod parser;
/*
fn gen_contains<'ctx>(ctx: &'ctx Context, pieces: &[(BV<'ctx>, BV<'ctx>)], x: &BV<'ctx>, y: &BV<'ctx>) -> Bool<'ctx> {
    let terms: Vec<_> = pieces.iter().map(|(x_, y_)| Bool::and(ctx, &[&x_._eq(x), &y_._eq(y)])).collect();
    let terms: Vec<_> = terms.iter().collect();
    Bool::or(ctx, &terms)
}

fn gen_contains_all<'a, 'ctx>(ctx: &'ctx Context, pieces: &'a [(BV<'ctx>, BV<'ctx>)], positions: impl Iterator<Item = (u64, u64)> + 'a) -> Bool<'ctx> {
    let values: Vec<_> = positions.map(|(x, y)| {
        let x = BV::from_u64(ctx, x, 2);
        let y = BV::from_u64(ctx, y, 2);
        gen_contains(ctx, pieces, &x, &y)
    }).collect();
    let values: Vec<_> = values.iter().collect();
    Bool::and(ctx, &values)
}

fn gen_has_won<'ctx>(ctx: &'ctx Context, pieces: &[(BV<'ctx>, BV<'ctx>)]) -> Bool<'ctx> {
    let vertical = (0..3).map(|x| gen_contains_all(ctx, pieces, repeat(x).zip(0..3)));
    let horizontal = (0..3).map(|y| gen_contains_all(ctx, pieces, (0..3).zip(repeat(y))));
    let diagional_1 = gen_contains_all(ctx, pieces, (0..3).zip(0..3));
    let diagional_2 = gen_contains_all(ctx, pieces, (0..3).rev().zip(0..3));
    let values: Vec<_> = vertical.chain(horizontal).chain(once(diagional_1)).chain(once(diagional_2)).collect();
    let values: Vec<_> = values.iter().collect();
    Bool::or(ctx, &values)
}

fn gen_not_duplicate<'ctx>(ctx: &'ctx Context, whites: &[(BV<'ctx>, BV<'ctx>)], blacks: &[(BV<'ctx>, BV<'ctx>)], x: &BV<'ctx>, y: &BV<'ctx>) -> Bool<'ctx> {
    let w_contains = gen_contains(ctx, whites, x, y);
    let b_contains = gen_contains(ctx, blacks, x, y);
    Bool::or(ctx, &[&w_contains, &b_contains]).not()
}

fn gen_first_move<'ctx>(ctx: &'ctx Context, whites: &mut Vec<(BV<'ctx>, BV<'ctx>)>, blacks: &mut Vec<(BV<'ctx>, BV<'ctx>)>, depth: usize) -> Bool<'ctx> {
    let x = BV::new_const(ctx, format!("whitex{}", depth), 2);
    let y = BV::new_const(ctx, format!("whitey{}", depth), 2);
    let const_2 = BV::from_u64(ctx, 2, 2);
    let x_le = x.bvule(&const_2);
    let y_le = y.bvule(&const_2);
    let not_duplicate = gen_not_duplicate(ctx, whites, blacks, &x, &y);
    whites.push((x.clone(), y.clone()));
    let has_won_immediately = gen_has_won(ctx, whites);
    let has_won = if depth == 0 {
        has_won_immediately
    }
    else {
        let will_win = gen_black_move(ctx, whites, blacks, depth - 1);
        Bool::or(ctx, &[&has_won_immediately, &will_win])
    };
    Bool::and(ctx, &[&x_le, &y_le, &has_won, &not_duplicate])
}

fn gen_white_move<'ctx>(ctx: &'ctx Context, whites: &mut Vec<(BV<'ctx>, BV<'ctx>)>, blacks: &mut Vec<(BV<'ctx>, BV<'ctx>)>, depth: usize) -> Bool<'ctx> {
    let x = BV::new_const(ctx, format!("whitex{}", depth), 2);
    let y = BV::new_const(ctx, format!("whitey{}", depth), 2);
    let const_2 = BV::from_u64(ctx, 2, 2);
    let x_le = x.bvule(&const_2);
    let y_le = y.bvule(&const_2);
    let not_duplicate = gen_not_duplicate(ctx, whites, blacks, &x, &y);
    whites.push((x.clone(), y.clone()));
    let has_won_immediately = gen_has_won(ctx, whites);
    let has_won = if depth == 0 {
        has_won_immediately
    }
    else {
        let will_win = gen_black_move(ctx, whites, blacks, depth - 1);
        Bool::or(ctx, &[&has_won_immediately, &will_win])
    };
    let body = Bool::and(ctx, &[&x_le, &y_le, &has_won, &not_duplicate]);
    // Research more about patterns. Maybe there is vital performance here?
    exists_const(ctx, &[&x, &y], &[], &body)
}

fn gen_black_move<'ctx>(ctx: &'ctx Context, whites: &mut Vec<(BV<'ctx>, BV<'ctx>)>, blacks: &mut Vec<(BV<'ctx>, BV<'ctx>)>, depth: usize) -> Bool<'ctx> {
    let x = BV::new_const(ctx, format!("blackx{}", depth), 2);
    let y = BV::new_const(ctx, format!("blacky{}", depth), 2);
    let const_2 = BV::from_u64(ctx, 2, 2);
    let x_le = x.bvule(&const_2);
    let y_le = y.bvule(&const_2);
    let not_duplicate = gen_not_duplicate(ctx, whites, blacks, &x, &y);
    blacks.push((x.clone(), y.clone()));
    let has_not_won = gen_has_won(ctx, blacks).not();
    let white_will_win = gen_white_move(ctx, whites, blacks, depth);
    let body = Bool::or(ctx, &[&x_le, &y_le, &not_duplicate]).implies(&Bool::and(ctx, &[&has_not_won, &white_will_win]));
    forall_const(ctx, &[&x, &y], &[], &body)
}
*/
fn main() {
    /*let problem = Problem::parse("hi");
    let now = std::time::Instant::now();
    let ctx = Context::new(&Default::default());
    let mut whites = Vec::new();
    let mut blacks = Vec::new();
    //let c0 = BV::from_u64(&ctx, 0, 2);
    //let c1 = BV::from_u64(&ctx, 1, 2);
    //let c2 = BV::from_u64(&ctx, 2, 2);
    //whites.push((c2.clone(), c0.clone()));
    //whites.push((c1.clone(), c2.clone()));
    //whites.push((c0.clone(), c2.clone()));
    //blacks.push((c0.clone(), c0.clone()));
    //blacks.push((c1.clone(), c0.clone()));
    //blacks.push((c2.clone(), c0.clone()));
    let formula = gen_first_move(&ctx, &mut whites, &mut blacks, 4);
    let solver = Tactic::new(&ctx, "simplify").and_then(&Tactic::new(&ctx, "elim-small-bv")).and_then(&Tactic::new(&ctx, "smt")).solver();
    solver.assert(&formula);
    dbg!(solver.check());
    dbg!(solver.get_model());
    println!("{:?}", now.elapsed());*/
    let now = std::time::Instant::now();
    let ctx = Context::new(&Default::default());
    let problem = tictac::problem();
    let domain = tictac::domain();
    let f = solver_z3::solve(&problem, &domain);
    let solver = Tactic::new(&ctx, "simplify").and_then(&Tactic::new(&ctx, "elim-small-bv")).and_then(&Tactic::new(&ctx, "smt")).solver();
    solver.assert(&f(&ctx));
    dbg!(solver.check());
    dbg!(solver.get_model());
    println!("{:?}", now.elapsed());
    dbg!(&solver::solve(&problem, &domain));
}

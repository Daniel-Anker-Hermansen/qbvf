#![feature(slice_group_by)]

use bddl::{Domain, Problem};
use lalrpop_util::lalrpop_mod;
use logos::Logos;
use z3::{Context, Tactic};

use crate::qbf::{atom, display_tseitin, qdimacs, BitVector};

mod bddl;
mod solver;
mod solver_z3;
mod lexer;
mod qbf;

lalrpop_mod!(parser);

fn main() {
    let a = atom();
    let b = atom();
    let form = a.exists(b.forall((!!a).ite(!!b, !b)));
    let bv = BitVector::new(4);
    let form = bv.exists(bv.equal(5) & bv.le(5));
    eprintln!("{}", form);
    eprintln!("{}", form.clone().denegify());
    eprintln!("{}", form.clone().denegify().prenexify());
    let (a, b) = form.denegify().prenexify().prenex_to_prenex_cnf();
    eprintln!("{}", display_tseitin(&a, &b));
    println!("{}", qdimacs(&a, &b));
    return;

    let problem = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();
    let domain = std::fs::read_to_string(std::env::args().nth(2).unwrap()).unwrap();
    let problem = parse_problem(&problem);
    let domain = parse_domain(&domain);
    let now = std::time::Instant::now();
    let ctx = Context::new(&Default::default());
    let f = solver_z3::solve(&problem, &domain);
    let solver = Tactic::new(&ctx, "simplify").and_then(&Tactic::new(&ctx, "smt")).solver();
    let formula = f(&ctx);
    solver.assert(&formula);
    dbg!(solver.check());
    dbg!(solver.get_model());
    println!("{:?}", now.elapsed());
    dbg!(&solver::solve(&problem, &domain));
}

fn parse_domain(src: &str) -> Domain {
    let lexer = lexer::Token::lexer(src);
    let parser = parser::DomainParser::new();
    parser.parse(lexer).unwrap()
}

fn parse_problem(src: &str) -> Problem {
    let lexer = lexer::Token::lexer(src);
    let parser = parser::ProblemParser::new();
    parser.parse(lexer).unwrap()
}

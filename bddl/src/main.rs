use bddl::{Domain, Problem};
use lalrpop_util::lalrpop_mod;
use logos::Logos;
use z3::{Context, Tactic};

mod bddl;
mod solver;
mod solver_z3;
mod lexer;
//mod parser;

lalrpop_mod!(parser);

fn main() {
    let problem = parse_problem(include_str!("ttt.problem"));
    let domain = parse_domain(include_str!("ttt.domain"));
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

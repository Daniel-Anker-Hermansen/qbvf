#![feature(slice_group_by)]

use bddl::{Domain, Problem};
use lalrpop_util::lalrpop_mod;
use logos::Logos;

mod bddl;
mod solver;
mod solver_z3;
mod solver_qbf;
mod lexer;
mod qbf;

lalrpop_mod!(parser);

fn main() {
    let sproblem = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();
    let sdomain = std::fs::read_to_string(std::env::args().nth(2).unwrap()).unwrap();
    let problem = parse_problem(&sproblem);
    let domain = parse_domain(&sdomain);
    let now = std::time::Instant::now();
    let formula = solver_qbf::solve(problem, domain, false);
    println!("{}: {:?}", formula.check_with_preprocessing(), now.elapsed());
    let problem = parse_problem(&sproblem);
    let domain = parse_domain(&sdomain);
    let now = std::time::Instant::now();
    let formula = solver_qbf::solve(problem, domain, true);
    println!("{}: {:?}", formula.check_with_preprocessing(), now.elapsed());
    let problem = parse_problem(&sproblem);
    let domain = parse_domain(&sdomain);
    dbg!(solver::solve(&problem, &domain));
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

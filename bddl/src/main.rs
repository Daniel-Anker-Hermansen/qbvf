#![feature(slice_group_by)]

use bddl::{Domain, Problem};
use lalrpop_util::lalrpop_mod;
use logos::Logos;

use crate::qbf::qdimacs;

mod bddl;
mod solver;
mod solver_z3;
mod solver_qbf;
mod lexer;
mod qbf;

lalrpop_mod!(parser);

fn main() {
    let problem = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();
    let domain = std::fs::read_to_string(std::env::args().nth(2).unwrap()).unwrap();
    let problem = parse_problem(&problem);
    let domain = parse_domain(&domain);
    let formula = solver_qbf::solve(problem, domain);
    eprintln!("{}", formula);
    let tceicin = formula.denegify().prenexify().prenex_to_prenex_cnf();
    println!("{}", qdimacs(&tceicin.0, &tceicin.1));
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

use z3::{Context, Tactic};

#[allow(unused)]
mod bddl;
mod tictac;
mod solver;
mod solver_z3;
//mod parser;

fn main() {
    let now = std::time::Instant::now();
    let ctx = Context::new(&Default::default());
    let problem = tictac::problem();
    let domain = tictac::domain();
    let f = solver_z3::solve(&problem, &domain);
    let solver = Tactic::new(&ctx, "simplify").and_then(&Tactic::new(&ctx, "smt")).solver();
    let formula = f(&ctx);
    solver.assert(&formula);
    dbg!(solver.check());
    dbg!(solver.get_model());
    println!("{:?}", now.elapsed());
    dbg!(&solver::solve(&problem, &domain));
}

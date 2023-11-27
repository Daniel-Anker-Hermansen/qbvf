use std::{iter::repeat, convert::identity};

use crate::{qbf::{BitVector, Formula, Atom, atom}, bddl::{E, Size, Pred, InitPred, SubCondition, Condition, Action, Domain, Problem}};

struct Context {
    truth: Atom,
    size: Size,
    board: SymbolicBoard,
    domain: Domain,
    problem: Problem,
}

pub fn solve(problem: Problem, domain: Domain) -> Formula {
    let fake_board = SymbolicBoard { size: problem.size, symbols: Vec::new() };
    let mut context = Context {
        truth: atom(),
        size: problem.size,
        board: fake_board,
        domain,
        problem,
    };
    context.board = SymbolicBoard::init(&context, &context.problem.init, context.problem.size);
    !!context.truth & context.solve_black(context.problem.depth)
}

impl Context {
    fn gen_bounds_check(&self, e: &E, v: &BitVector, max: u64) -> Formula {
        v.ge(0) & v.le(max - 1) & match e {
            &E::Add(i) => v.le(max - i as u64 - 1),
            &E::Sub(i) => v.ge(i as u64),
            _ => !!self.truth,
        } 
    }

    fn gen_e_bv_eq(&self, e: &E, v: &BitVector, target: u64, max: u64) -> Formula {
        match e {
            &E::Add(i) => if i as u64 > target { !self.truth } else { v.equal(target - i as u64) },
            &E::Sub(i) => if i as u64 + target >= max { !self.truth } else { v.equal(target + i as u64) },
            &E::Int(i) => if i as u64 == target { !!self.truth } else { !self.truth },
            &E::Identity => v.equal(target),
            &E::Min => if 0 == target { !!self.truth } else { !self.truth },
            &E::Max => if max == target + 1 { !!self.truth } else { !self.truth },
        }
    }
    
    fn pred_to_atoms(&self, pred: Pred) -> (Atom, Atom) {
        match pred {
            Pred::Open => (self.truth, self.truth.invert()),
            Pred::White => (self.truth.invert(), self.truth.invert()),
            Pred::Black => (self.truth.invert(), self.truth),
        }
    }

    fn gen_subcondition(&self, sub_condition: SubCondition, x: &BitVector, y: &BitVector) -> Formula {
        match sub_condition {
            SubCondition::Id { pred, x_e, y_e } => {
                let x_bound = self.gen_bounds_check(&x_e, x, self.size.x as u64);
                let y_bound = self.gen_bounds_check(&y_e, y, self.size.y as u64);
                let pred_assert = self.board.gen_pred(self, x, &x_e, y, &y_e, pred);
                x_bound & y_bound & pred_assert
            },
            SubCondition::Not { pred, x_e, y_e } => {
                let x_bound = self.gen_bounds_check(&x_e, x, self.size.x as u64);
                let y_bound = self.gen_bounds_check(&y_e, y, self.size.y as u64);
                let pred_assert = self.board.gen_pred(self, x, &x_e, y, &y_e, pred);
                x_bound & y_bound & !pred_assert
            },

        }
    }

    fn gen_condition(&self, condition: &Condition, x: &BitVector, y: &BitVector) -> Formula {
        condition.sub_cond.iter().map(|sub_condition| self.gen_subcondition(*sub_condition, x, y))
            .reduce(|a, b| a & b)
            .unwrap_or(!!self.truth)
    }

    fn gen_goals(&self, goals: &[Condition]) -> Formula {
        let x_sz = (2 * self.size.x - 1).ilog2();
        let y_sz = (2 * self.size.y - 1).ilog2();
        let x = BitVector::new(x_sz as usize);
        let y = BitVector::new(y_sz as usize);
        let formula = goals.iter()
            .map(|condition| self.gen_condition(condition, &x, &y))
            .reduce(|a, b| a | b)
            .unwrap_or(!self.truth);
        x.exists(y.exists(formula))
    }

    fn effect_action(&self, actions: &[Action], x: &BitVector, y: &BitVector, tpe: &BitVector) -> (Formula, SymbolicBoard) {
        let effects: Vec<Effect> = actions.iter()
            .enumerate()
            .flat_map(|(idx, action)| action.effect.sub_cond.iter()
                .map(move |cond| match cond {
                    SubCondition::Id { pred, x_e, y_e } => Effect { x: *x_e, y: *y_e, pred: *pred, tpe: idx as _ },
                    _ => panic!("Cannot not as an effect"),
                })
            )
            .collect();
        self.board.effect(self, &effects, x, y, tpe)
    }

    fn solve_black(&mut self, depth: u64) -> Formula {
        if depth == 0 {
            return !self.truth
        }
        let black_actions = &self.domain.black_actions;
        let x_sz = (2 * self.size.x - 1).ilog2();
        let y_sz = (2 * self.size.y - 1).ilog2();
        let tpe_sz = (2 * black_actions.len() - 1).ilog2();
        let x = BitVector::new(x_sz as usize);
        let y = BitVector::new(y_sz as usize);
        let tpe = BitVector::new(tpe_sz.max(1) as usize);
        let (effect, new_board) = self.effect_action(black_actions, &x, &y, &tpe);
        let valid = black_actions.iter()
            .enumerate()
            .map(|(idx, action)| tpe.equal(idx as u64)
                .implies(self.gen_condition(&action.precondition, &x, &y)))
            .reduce(|a, b| a & b)
            .unwrap_or(!!self.truth);
        let previous = std::mem::replace(&mut self.board, new_board);
        let goal = self.gen_goals(&self.problem.black_goals);
        let wins = self.solve_white(depth - 1);
        let new_board = std::mem::replace(&mut self.board, previous);
        new_board.exists(x.exists(y.exists(tpe.exists(effect & valid & (wins | goal)))))
    }

    fn solve_white(&mut self, depth: u64) -> Formula {
        if depth == 0 {
            return !self.truth
        }
        let white_actions = &self.domain.white_actions;
        let x_sz = (2 * self.size.x - 1).ilog2();
        let y_sz = (2 * self.size.y - 1).ilog2();
        let tpe_sz = (2 * white_actions.len() - 1).ilog2();
        let x = BitVector::new(x_sz as usize);
        let y = BitVector::new(y_sz as usize);
        let tpe = BitVector::new(tpe_sz.max(1) as usize);
        let (effect, new_board) = self.effect_action(white_actions, &x, &y, &tpe);
        let valid = white_actions.iter()
            .enumerate()
            .map(|(idx, action)| tpe.equal(idx as u64)
                .implies(self.gen_condition(&action.precondition, &x, &y)))
            .reduce(|a, b| a & b)
            .unwrap_or(!!self.truth);
        let previous = std::mem::replace(&mut self.board, new_board);
        let goal = self.gen_goals(&self.problem.black_goals);
        let wins = self.solve_black(depth - 1);
        let new_board = std::mem::replace(&mut self.board, previous);
        new_board.forall(x.forall(y.forall(tpe.forall((effect & valid).implies(wins | goal)))))
    }
}


struct SymbolicBoard {
    size: Size,
    // First atom indicates open and second indicates black
    symbols: Vec<Vec<(Atom, Atom)>>,
}

struct Effect {
    x: E,
    y: E,
    pred: Pred,
    tpe: u64,
}

fn tuple_eq(a: (Atom, Atom), b: (Atom, Atom)) -> Formula {
    a.0.equal(b.0) & a.1.equal(b.1)
}

// Assumes bounds checks have been done. Causes garbage if not the case
fn e_eq(context: &Context, bv: &BitVector, e: &E, value: u64, max: i64) -> Formula {
    match e {
        E::Add(v) => bv.equal(value.saturating_sub(*v as u64)),
        E::Sub(v) => bv.equal((value + *v as u64).min(max as u64 - 1)),
        E::Int(v) => if value == *v as u64 { !!context.truth } else  { !context.truth },
        E::Identity => bv.equal(value),
        E::Min => if value == 0 { !!context.truth } else  { !context.truth },
        E::Max => if value == max as u64 - 1 { !!context.truth } else { !context.truth },
    }
}

impl SymbolicBoard {
    fn gen_pred(&self, context: &Context, x: &BitVector, x_e: &E, y: &BitVector, y_e: &E, pred: Pred) -> Formula {
        let (o, b) = context.pred_to_atoms(pred);
        (0..self.size.x as usize).flat_map(|x| repeat(x).zip(0..self.size.y as usize))
            .map(|(xi, yi)| 
                 (e_eq(context, x, x_e, xi as u64, self.size.x) & e_eq(context, y, y_e, yi as u64, self.size.y))
                    .implies(o.equal(self.symbols[xi][yi].0) & b.equal(self.symbols[xi][yi].1)))
            .reduce(|a, b| a & b)
            .expect("board size is not zero")
    }

    fn _gen_static_pred(&self, context: &Context, x: usize, y: usize, pred: Pred) -> Formula {
        let (o, b) = context.pred_to_atoms(pred);
        o.equal(self.symbols[x][y].0) & b.equal(self.symbols[x][y].1)
    }

    fn init(context: &Context, initpreds: &[InitPred], size: Size) -> SymbolicBoard {
        let mut symbols = vec![vec![context.pred_to_atoms(Pred::Open); size.y as usize]; size.x as usize]; 
        for initpred in initpreds {
            symbols[initpred.x as usize][initpred.y as usize] = context.pred_to_atoms(initpred.pred);
        }
        SymbolicBoard { size, symbols }
    }

    fn rec_effect(&self, context: &Context, effects: &[Effect], x: &BitVector, y: &BitVector, tpe: &BitVector, next: (Atom, Atom), xi: usize, yi: usize) -> Formula {
        match effects {
            [] => tuple_eq(self.symbols[xi][yi], next),
            [hd, ..] => {
                let tpe_eq = tpe.equal(hd.tpe);
                let x_bound = context.gen_bounds_check(&hd.x, x, self.size.x as u64);
                let y_bound = context.gen_bounds_check(&hd.y, y, self.size.y as u64);
                let x_eq = context.gen_e_bv_eq(&hd.x, x, xi as u64, self.size.x as u64);
                let y_eq = context.gen_e_bv_eq(&hd.y, y, yi as u64, self.size.y as u64);
                let then = tuple_eq(next, context.pred_to_atoms(hd.pred));
                let otherwise = self.rec_effect(context, &effects[1..], x, y, tpe, next, xi, yi);
                (tpe_eq & x_bound & y_bound & x_eq & y_eq).ite(then, otherwise)
            }
        }
    }

    fn effect(&self, context: &Context, effects: &[Effect], x: &BitVector, y: &BitVector, tpe: &BitVector) -> (Formula, SymbolicBoard) {
        let symbols = (0..self.size.x).map(|_| (0..self.size.y).map(|_| (atom(), atom())).collect()).collect();
        let board = SymbolicBoard { size: self.size, symbols };
        let formula = (0..self.size.x as usize).flat_map(|x| repeat(x).zip(0..self.size.y as usize))
            .map(|(xi, yi)| self.rec_effect(context, effects, x, y, tpe, board.symbols[xi][yi], xi, yi))
            .reduce(|a, b| a & b)
            .expect("board is not zero size");
        (formula, board)
    }

    fn exists(&self, formula: Formula) -> Formula {
        self.symbols.iter()
            .flat_map(identity)
            .fold(formula, |acc, (a, b)| a.exists(b.exists(acc)))
    }

    fn forall(&self, formula: Formula) -> Formula {
        self.symbols.iter()
            .flat_map(identity)
            .fold(formula, |acc, (a, b)| a.forall(b.forall(acc)))
    }
}

use std::iter::repeat;

use super::bddl::*;

fn e_noramlize(e: &E, v: i64, max: i64) -> i64 {
    match e {
        E::Add(u) => v + u,
        E::Sub(u) => v - u,
        E::Identity => v,
        E::Min => 0,
        E::Max => max,
    }
}

#[derive(Debug, Clone)]
struct Board {
    preds: Vec<Vec<Pred>>,
    size: Size,
}

impl Board {
    fn new(size: Size) -> Board {
        Board { preds: vec![vec![Pred::Open; size.y as _]; size.x as _], size }
    }

    fn assert_pred(&self, x: i64, y: i64, pred: Pred) -> bool {
        0 <= x && x < self.size.x && 0 <= y && y < self.size.y && self.preds[x as usize][y as usize] == pred
    }

    fn assert_not_pred(&self, x: i64, y: i64, pred: Pred) -> bool {
        0 <= x && x < self.size.x && 0 <= y && y < self.size.y && self.preds[x as usize][y as usize] != pred
    }

    fn assert_condition(&self, condition: &Condition, x: i64, y: i64) -> bool {
        condition.sub_cond.iter().all(|cond| {
            match cond {
                SubCondition::Id { pred, x_e, y_e } => self.assert_pred(e_noramlize(x_e, x, self.size.x), e_noramlize(y_e, y, self.size.y), *pred),
                SubCondition::Not { pred, x_e, y_e } => self.assert_not_pred(e_noramlize(x_e, x, self.size.x), e_noramlize(y_e, y, self.size.y), *pred),
            }
        })
    }

    fn effect_conditon(&mut self, condition: &Condition, x: i64, y: i64) {
        condition.sub_cond.iter().for_each(|cond| {
            match cond {
                SubCondition::Id { pred, x_e, y_e } => self.effect(e_noramlize(x_e, x, self.size.x), e_noramlize(y_e, y, self.size.y), *pred),
                SubCondition::Not { .. } => panic!("Cannot use not subcondition in effect"), 
            };
        })
        
    }

    fn actions(&self, precondition: &Condition) -> Vec<(i64, i64)> {
        (0..self.size.x).flat_map(|x| repeat(x).zip(0..self.size.y))
            .filter(|&(x, y)| self.assert_condition(precondition, x, y))
            .collect()
    }

    fn effect(&mut self, x: i64, y: i64, pred: Pred) {
        self.preds[x as usize][y as usize] = pred;
    }

    fn assert_goals(&self, goal: &[Condition]) -> bool {
        (0..self.size.x).flat_map(|x| repeat(x).zip(0..self.size.y))
            .any(|(x, y)| goal.iter().any(|c| self.assert_condition(c, x, y)))
    }
}

pub fn solve(problem: &Problem, domain: &Domain) -> Option<(String, i64, i64)> {
    let mut board = Board::new(problem.size);
    for init_pred in &problem.init {
        board.effect(init_pred.x, init_pred.y, init_pred.pred);
    }
    solve_black(problem, domain, board, problem.depth)
}

fn solve_black(problem: &Problem, domain: &Domain, board: Board, depth: u64) -> Option<(String, i64, i64)> {
    if depth == 0 {
        return None;
    }
    for action in &domain.black_actions {
        let valids = board.actions(&action.precondition);
        for (x, y) in valids {
            let mut board = board.clone();
            board.effect_conditon(&action.effect, x, y);
            if board.assert_goals(&problem.black_goals) {
                return Some((action.name.clone(), x, y));
            }
            if solve_white(problem, domain, board, depth - 1) {
                return Some((action.name.clone(), x, y));
            }
        }
    }
    None
}

fn solve_white(problem: &Problem, domain: &Domain, board: Board, depth: u64) -> bool {
    if depth == 0 {
        return false;
    }
    for action in &domain.white_actions {
        let valids = board.actions(&action.precondition);
        for (x, y) in valids {
            let mut board = board.clone();
            board.effect_conditon(&action.effect, x, y);
            if board.assert_goals(&problem.white_goals) {
                return false;
            }
            let f = solve_black(problem, domain, board.clone(), depth - 1);
            if f.is_none() {
                return false;
            }
        }
    }
    true
}

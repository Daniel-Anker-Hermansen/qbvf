use std::{iter::repeat, intrinsics::discriminant_value};

use z3::{Context, ast::{BV, Bool, Ast, Dynamic, exists_const}, FuncDecl, Sort, DatatypeBuilder, DatatypeSort};

use crate::bddl::{InitPred, Pred, SubCondition, Size, Condition, Problem, Domain};

struct Solver<'ctx> {
    ctx: &'ctx Context,
    pred_datatype: DatatypeSort<'ctx>,
    open: Dynamic<'ctx>,
    white: Dynamic<'ctx>,
    black: Dynamic<'ctx>,
    problem: &'ctx Problem,
    domain: &'ctx Domain,
    x_sz: u32,
    y_sz: u32,
    size: Size,
    board: Vec<(BV<'ctx>, BV<'ctx>, Dynamic<'ctx>)>,
}

impl<'ctx> Solver<'ctx> {
    fn new(ctx: &'ctx Context, problem: &'ctx Problem, domain: &'ctx Domain) -> Solver<'ctx> {
        let pred_datatype = DatatypeBuilder::new(ctx, "Pred")
            .variant("Open", Vec::new())
            .variant("White", Vec::new())
            .variant("Black", Vec::new())
            .finish();
        let open = pred_datatype.variants[0].constructor.apply(&[]);
        let white = pred_datatype.variants[1].constructor.apply(&[]);
        let black = pred_datatype.variants[2].constructor.apply(&[]);
        let x_sz = (2 * problem.size.x - 1).ilog2();
        let y_sz = (2 * problem.size.y - 1).ilog2();
        let board = Vec::new();
        let size = problem.size;
        Solver {
            ctx,
            pred_datatype,
            open,
            white,
            black,
            problem,
            domain,
            x_sz,
            y_sz,
            size,
            board,
        }
    }

    fn pred_to_z3(&self, pred: Pred) -> &Dynamic<'ctx> {
        match pred {
            Pred::Open => &self.open,
            Pred::White => &self.white,
            Pred::Black => &self.black,
        }
    }

    fn gen_pred_assert(&self, x: &BV<'ctx>, y: &BV<'ctx>, pred: Pred) -> Bool<'ctx> {
        fn helper<'ctx>(this: &Solver<'ctx>, x: &BV<'ctx>, y: &BV<'ctx>, pred: Pred, board: &[(BV<'ctx>, BV<'ctx>, Dynamic<'ctx>)]) -> Bool<'ctx> {
            if board.is_empty() {
                match pred {
                    Pred::Open => Bool::from_bool(&this.ctx, true),
                    _ => Bool::from_bool(&this.ctx, false),
                }
            }
            else {
                let (hx, hy, hp) = board.last().unwrap();
                let p = this.pred_to_z3(pred);
                let tail = helper(this, x, y, pred, &board[..board.len() - 1]);
                let xe = x._eq(hx);
                let ye = y._eq(hy);
                let pe = p._eq(hp);
                let matche = Bool::and(&this.ctx, &[&xe, &ye, &pe]);
                let nmatche = Bool::and(&this.ctx, &[&xe, &ye, &pe.not()]);
                let alt = Bool::and(&this.ctx, &[&nmatche.not(), &tail]);
                Bool::or(&this.ctx, &[&matche, &alt])
            }
        }
        helper(self, x, y, pred, &self.board)
    }

    fn gen_subcondition(&self, sub_condition: SubCondition, x: &BV<'ctx>, y: &BV<'ctx>) -> Option<Bool<'ctx>> {
        match sub_condition {
            SubCondition::Id { pred, x_e, y_e } => {
                let x = x_e.noramlize(x, self.size.x);
                let y = y_e.noramlize(y, self.size.y);
                if x < 0 || x >= self.size.x || y < 0 || y >= self.size.y {
                    None
                }
                else {
                    let x = BV::from_i64(self.ctx, x, self.x_sz);
                    let y = BV::from_i64(self.ctx, y, self.y_sz);
                    Some(self.gen_pred_assert(&x, &y, pred))
                }
            },
            SubCondition::Not { pred, x_e, y_e } => self.gen_subcondition(SubCondition::Id { pred, x_e, y_e }, x, y).map(|b| b.not()),
        }
    }

    fn gen_condition(&self, condition: &Condition, x: &BV<'ctx>, y: &BV<'ctx>) -> Option<Bool<'ctx>> {
        let all = condition.sub_cond.iter()
            .map(|sub_condition| self.gen_subcondition(*sub_condition, x, y))
            .collect::<Option<Vec<Bool<'ctx>>>>()?;
        Some(Bool::and(self.ctx, &all.iter().collect::<Vec<&Bool<'ctx>>>()))
    }

    fn gen_goals(&self, goals: &[Condition]) -> Bool<'ctx> {
        let ors: Vec<_> = (0..self.size.x).flat_map(|x| repeat(x).zip(0..self.size.y))
            .flat_map(|(x, y)| {
                goals.iter()
                    .filter_map(move |condition| self.gen_condition(condition, x, y))
            })
            .collect();
        Bool::or(self.ctx, &ors.iter().collect::<Vec<&Bool<'ctx>>>())
    }

    fn effect_action(&mut self, effect: &Condition, x: i64, y: i64) {
        for subcondition in &effect.sub_cond {
            match subcondition {
                SubCondition::Id { pred, x_e, y_e } => {
                    let x = x_e.noramlize(x, self.size.x);
                    let y = y_e.noramlize(y, self.size.y);
                    let p = self.pred_to_z3(*pred).clone();
                    let x = BV::from_i64(self.ctx, x, self.x_sz);
                    let y = BV::from_i64(self.ctx, y, self.y_sz);
                    self.board.push((x, y, p));
                },
                SubCondition::Not { .. } => panic!("Not cannot be used in effect"),
            }
        }
    }

    fn solve_black(&mut self, depth: u64) -> Bool<'ctx> {
        if depth == 0 {
            return Bool::from_bool(&self.ctx, false);
        }
        let black_actions = self.domain.black_actions.clone();
        let size = self.size;
        let ors: Vec<_> = black_actions.iter().map(|action| {
            let x = BV::new_const(&self.ctx, "h", self.x_sz);
            let y = BV::new_const(&self.ctx, "i", self.x_sz);
            let valid = self.gen_condition(&action.precondition, x, y);
            self.effect_action(&action.effect, x, y);
            let wins = self.solve_white(depth - 1);
            // remove the effect of the action
            self.board.truncate(self.board.len() - action.effect.sub_cond.len());
            exists_const(&self.ctx, &[&x, &y], &[], valid.and(wins))
        }).collect();
        Bool::or(&self.ctx, &ors.iter().collect::<Vec<&Bool>>())
    }
    
    fn solve_white(&mut self, depth: u64) -> Bool<'ctx> {
        if depth == 0 {
            return Bool::from_bool(&self.ctx, false);
        }

        unimplemented!()
    }
}

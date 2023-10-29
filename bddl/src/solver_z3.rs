use std::iter::repeat;

use z3::{Context, ast::{BV, Bool, Ast, Dynamic, forall_const, Datatype, exists_const}, FuncDecl, Sort, DatatypeBuilder, DatatypeSort};

use crate::bddl::{InitPred, Pred, SubCondition, Size, Condition, Problem, Domain, E, Action};

fn gen_bounds_check<'ctx>(e: &E, v: &BV<'ctx>, max: i64) -> Bool<'ctx> {
    match e {
        E::Add(o) => v.bvule(&BV::from_i64(v.get_ctx(), max - o - 1, v.get_size())),
        E::Sub(o) => v.bvuge(&BV::from_i64(v.get_ctx(), *o, v.get_size())),
        _ => Bool::from_bool(v.get_ctx(), true),
    }
}

fn e_to_bv<'ctx>(e: &E, x: &BV<'ctx>, sz: i64) -> BV<'ctx> {
    match e {
        E::Add(o) => x + BV::from_i64(x.get_ctx(), 0, x.get_size()),
        E::Sub(o) => x - BV::from_i64(x.get_ctx(), 0, x.get_size()),
        E::Identity => *x,
        E::Min => BV::from_i64(x.get_ctx(), 0, x.get_size()),
        E::Max => BV::from_i64(x.get_ctx(), sz - 1, x.get_size()),
    }
}

fn gen_coor_bounds<'ctx>(ex: &E, ey: &E, size: Size, x: &BV<'ctx>, y: &BV<'ctx>) -> Bool<'ctx> {
    Bool::and(x.get_ctx(), &[&gen_bounds_check(ex, x, size.x), &gen_bounds_check(ey, y, size.y)])
}

struct Effect<'ctx> {
    x: &'ctx E,
    y: &'ctx E,
    pred: &'ctx Dynamic<'ctx>,
    tp: i64,
}

struct SymbolicBoard<'ctx> {
    prefix: String,
    size: Size,
    symbols: Vec<Vec<Dynamic<'ctx>>>,
}

impl<'ctx> SymbolicBoard<'ctx> {
    fn pred(&self, x: &BV<'ctx>, y: &BV<'ctx>, pred: &Dynamic<'ctx>) -> Bool<'ctx> {
        let all = (0..self.symbols.len()).flat_map(|x| repeat(x).zip(0..self.symbols[0].len()))
            .map(|(xid, yid)| {
                let xi = BV::from_i64(x.get_ctx(), xid as _, x.get_size());
                let yi = BV::from_i64(y.get_ctx(), yid as _, y.get_size());
                Bool::and(x.get_ctx(), &[&xi._eq(x), &yi._eq(y)]).implies(&self.symbols[xid][yid]._eq(pred))
            })
            .collect::<Vec<_>>();
        Bool::and(x.get_ctx(), &all.iter().collect::<Vec<_>>())
    }

    fn static_pred(&self, x: i64, y: i64, pred: &Dynamic<'ctx>) -> Bool<'ctx> {
        self.symbols[x as usize][y as usize]._eq(pred)
    }

    fn init(initpreds: &[InitPred], size: Size, solver: &'ctx Solver) -> (Self, Bool<'ctx>) {
        let symbols: Vec<Vec<Dynamic<'ctx>>> = (0..size.x)
            .map(|x| (0..size.y).map(|y| Datatype::new_const(&solver.ctx, format!("x{}y{}", x, y), &solver.pred_datatype.sort).into()).collect())
            .collect();
        let all = (0..symbols.len()).flat_map(|x| repeat(x).zip(0..symbols[0].len()))
            .map(|(xid, yid)| {
                let pred = initpreds.iter().find(|i| i.x == xid as _ && i.y == yid as _);
                let z3_pred = match pred {
                    Some(v) => solver.pred_to_z3(v.pred),
                    None => &solver.open,
                };
                symbols[xid][yid]._eq(z3_pred)
            })
            .collect::<Vec<_>>();

        let this = Self {
            prefix: String::new(),
            symbols,
            size,
        };
        (this, Bool::and(&solver.ctx, &all.iter().collect::<Vec<_>>()))
    }

    fn rec_effect(&self, effects: &[Effect<'_>], x: &BV<'ctx>, y: &BV<'ctx>, tpe: &BV<'ctx>, symbol: &Dynamic<'ctx>, xid: usize, yid: usize) -> Bool<'ctx> {
        match effects {
            &[] => self.symbols[xid][yid]._eq(symbol),
            &[hd, ..] => {
                let tpe_ = tpe._eq(&BV::from_i64(tpe.get_ctx(), hd.tp, tpe.get_size()));
                let x_ = e_to_bv(&hd.x, x, self.size.x)._eq(&BV::from_i64(x.get_ctx(), xid as _, x.get_size()));
                let y_ = e_to_bv(&hd.y, y, self.size.x)._eq(&BV::from_i64(y.get_ctx(), yid as _, y.get_size()));
                Bool::and(symbol.get_ctx(), &[&tpe_, &x_, &y_]).ite(&symbol._eq(&hd.pred), &self.rec_effect(&effects[1..], x, y, tpe, symbol, xid, yid))
            },
        }
    }

    fn effect(&self, effects: &[Effect<'_>], x: &BV<'ctx>, y: &BV<'ctx>, tpe: &BV<'ctx>, solver: &Solver<'ctx>) -> (Bool<'ctx>, Self) {
        let prefix = format!("_{}", self.prefix);
        let symbols: Vec<Vec<Dynamic<'ctx>>> = (0..self.size.x)
            .map(|x| (0..self.size.y).map(|y| Datatype::new_const(&solver.ctx, format!("x{}y{}", x, y), &solver.pred_datatype.sort).into()).collect())
            .collect();
        let all = (0..symbols.len()).flat_map(|x| repeat(x).zip(0..symbols[0].len()))
            .map(|(xid, yid)| {
                self.rec_effect(effects, x, y, tpe, &symbols[xid][yid], xid, yid)
            })
            .collect::<Vec<_>>();
        let b = Bool::and(x.get_ctx(), &all.iter().collect::<Vec<_>>());
        let this = Self { 
            prefix, 
            size: self.size, 
            symbols 
        };
        (b, this)
    }
}

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
    board: SymbolicBoard<'ctx>,
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

    fn effect_action(&self, actions: &[Action], x: &BV<'ctx>, y: &BV<'ctx>, tpe: &BV<'ctx>, board: &SymbolicBoard<'ctx>) -> (Bool<'ctx>, SymbolicBoard<'ctx>) {
        let effects: Vec<Effect> = actions.iter()
            .enumerate().
            flat_map(|(idx, action)| action.effect.sub_cond.iter()
                .map(|cond| match cond {
                    SubCondition::Id { pred, x_e, y_e } => Effect { x: x_e, y: y_e, pred: self.pred_to_z3(*pred), tp: idx as _ },
                    _ => panic!("Cannot not as an effect"),
                }))
            .collect();
        board.effect(&effects, x, y, tpe, self)
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

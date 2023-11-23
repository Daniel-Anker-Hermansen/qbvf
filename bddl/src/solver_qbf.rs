use std::iter::repeat;

use crate::{qbf::{BitVector, Formula, Atom}, bddl::{E, Size, Pred}};

struct Context {
    truth: Atom,
}

impl Context {
    fn gen_bounds_check(&self, e: &E, v: &BitVector, max: u64) -> Formula {
        match e {
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
}


struct SymbolicBoard {
    size: Size,
    // First atom indicates open and second indicates black
    symbols: Vec<Vec<(Atom, Atom)>>,
}

impl SymbolicBoard {
    fn gen_pred(&self, context: &Context, x: &BitVector, y: &BitVector, pred: Pred) -> Formula {
        let (o, b) = context.pred_to_atoms(pred);
        (0..self.size.x as usize).flat_map(|x| repeat(x).zip(0..self.size.y as usize))
            .map(|(xi, yi)| 
                 (x.equal(xi as u64) & y.equal(yi as u64))
                    .implies(o.equal(self.symbols[xi][yi].0) & b.equal(self.symbols[xi][yi].1)))
            .reduce(|a, b| a & b)
            .expect("board size is not zero")
    }

    fn gen_static_pred(&self, context: &Context, x: usize, y: usize, pred: Pred) -> Formula {
        let (o, b) = context.pred_to_atoms(pred);
        o.equal(self.symbols[x][y].0) & b.equal(self.symbols[x][y].1)
    }
}

use std::{ops::{BitOr, BitAnd, Not}, cell::Cell, fmt::{Display, Write}, collections::HashMap, iter::repeat_with};

thread_local! {
    static COUNT: Cell<i64> = Cell::new(0);
}

pub fn atom() -> Atom {
    COUNT.set(COUNT.get() + 1);
    Atom(COUNT.get())
}

#[derive(Debug, Clone, Copy)]
pub struct Atom(i64);

impl Not for Atom {
    type Output = Formula;

    fn not(self) -> Self::Output {
        Formula::Atom(Self(-self.0))
    }
}

impl Atom {
    pub fn exists(self, formula: Formula) -> Formula {
        Formula::Exists(self, Box::new(formula))
    }
    
    pub fn forall(self, formula: Formula) -> Formula {
        Formula::Forall(self, Box::new(formula))
    }

    pub fn invert(self) -> Self {
        Atom(-self.0)
    }

    pub fn equal(self, other: Self) -> Formula {
        !!self & !!other | !self & !other
    }
}

impl Display for Atom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0 > 0 {
            f.write_str(&format!("{}", self.0))
        }
        else {
            f.write_str(&format!("\u{ac}{}", -self.0))
        }
    }
}

#[derive(Debug, Clone)]
pub enum Formula {
    Atom(Atom),
    Not(Box<Self>),
    Exists(Atom, Box<Self>),
    Forall(Atom, Box<Self>),
    And(Box<Self>, Box<Self>),
    Or(Box<Self>, Box<Self>),
}

impl Display for Formula {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Formula::Atom(a) => f.write_str(&format!("{}", a)),
            Formula::Not(g) => f.write_str(&format!("\u{ac}{}", g)),
            Formula::Exists(a, g) => f.write_str(&format!("\u{2203}{}({})", a, g)),
            Formula::Forall(a, g) => f.write_str(&format!("\u{2200}{}({})", a, g)),
            Formula::And(a, b) => f.write_str(&format!("{}\u{2227}{}", a, b)),
            Formula::Or(a, b) => f.write_str(&format!("({}\u{2228}{})", a, b)),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Quantifier {
    Forall,
    Exists,
}

impl Display for Quantifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Quantifier::Forall => f.write_str(&format!("\u{2200}")),
            Quantifier::Exists => f.write_str(&format!("\u{2203}")),
        }
    }
}

pub fn display_tseitin(quantifiers: &Vec<(Quantifier, Atom)>, matrix: &Vec<Vec<Atom>>) -> String {
    let mut acc = String::new();
    for (quantifier, atom) in quantifiers {
        write!(&mut acc, "{}{}", quantifier, atom).unwrap();
    }
    write!(&mut acc, "{}", 
        matrix.iter().map(|f| 
            f.iter().map(|a| 
                format!("{}", a)).collect::<Vec<_>>().join("\u{2228}")
            )
            .collect::<Vec<String>>()
            .join("\u{2227}")
        ).unwrap();
    acc
}

pub fn qdimacs(quantifiers: &Vec<(Quantifier, Atom)>, matrix: &Vec<Vec<Atom>>) -> String {
    let mut counter = 0;
    let mut atom_map = HashMap::new();
    let mut acc = String::new();
    for group in quantifiers.group_by(|a, b| a.0 == b.0) {
        let quant = match group[0].0 {
            Quantifier::Forall => "a",
            Quantifier::Exists => "e",
        };
        acc.push_str(quant);
        for (_, v) in group {
            let mut atom = *atom_map.entry(v.0.abs()).or_insert_with(|| { counter += 1; counter });
            if v.0 < 0 { atom = -atom; }
            acc.push_str(&format!(" {}", atom));
        }
        acc.push_str(" 0\n");
    }
    for clause in matrix {
        for v in clause {
            let mut atom = *atom_map.entry(v.0.abs()).or_insert_with(|| { counter += 1; counter });
            if v.0 < 0 { atom = -atom; }
            acc.push_str(&format!("{} ", atom));
        }
        acc.push_str("0\n");
    }
    format!("p cnf {} {}\n{}", counter, matrix.len(), acc)
}

impl Formula {
    /// Propogates negation such that they are only at atom level
    pub fn denegify(self) -> Self {
        match self {
            Formula::Atom(v) => Formula::Atom(v),
            Formula::Not(v) => v.denegify_neg(),
            Formula::Exists(a, v) => Formula::Exists(a, Box::new(v.denegify())),
            Formula::Forall(a, v) => Formula::Forall(a, Box::new(v.denegify())),
            Formula::And(a, b) => Formula::And(Box::new(a.denegify()), Box::new(b.denegify())),
            Formula::Or(a, b) => Formula::Or(Box::new(a.denegify()), Box::new(b.denegify())),
        }
    }

    fn denegify_neg(self) -> Self {
        match self {
            Formula::Atom(v) => !v,
            Formula::Not(v) => v.denegify(),
            Formula::Exists(a, v) => Formula::Forall(a, Box::new(v.denegify_neg())),
            Formula::Forall(a, v) => Formula::Exists(a, Box::new(v.denegify_neg())),
            Formula::And(a, b) => Formula::Or(Box::new(a.denegify_neg()), Box::new(b.denegify_neg())),
            Formula::Or(a, b) => Formula::And(Box::new(a.denegify_neg()), Box::new(b.denegify_neg())),
        }
    }

    pub fn prenexify(self) -> Formula {
        let mut prenex = Vec::new();
        let mut ret = self.prenexify_inner(&mut prenex);
        for (is_exists, atom) in prenex.into_iter().rev() {
            ret = if is_exists {
                Self::Exists(atom, Box::new(ret))
            }
            else {
                Self::Forall(atom, Box::new(ret))
            };
        }
        ret
    }

    fn prenexify_inner(self, prenex: &mut Vec<(bool, Atom)>) -> Formula {
        match self {
            Formula::Atom(v) => Formula::Atom(v),
            Formula::Not(_) => panic!("Cannot prenex with not. denegify must be called first."),
            Formula::Exists(a, f) => {
                prenex.push((true, a));
                f.prenexify_inner(prenex)
            },
            Formula::Forall(a, f) => {
                prenex.push((false, a));
                f.prenexify_inner(prenex)
            },
            Formula::And(a, b) => Formula::And(Box::new(a.prenexify_inner(prenex)), Box::new(b.prenexify_inner(prenex))),
            Formula::Or(a, b) => Formula::Or(Box::new(a.prenexify_inner(prenex)), Box::new(b.prenexify_inner(prenex))),
        }
    }

    pub fn implies(self, other: Self) -> Self {
        !self | other
    }

    pub fn ite(self, then: Self, other: Self) -> Self {
        self.clone().implies(then) & (!self).implies(other)
    }

    pub fn prenex_to_prenex_cnf(&self) -> (Vec<(Quantifier, Atom)>, Vec<Vec<Atom>>) {
        let mut acc = Vec::new();
        let matrix = self.prenex_cnf_inner(&mut acc);
        (acc, matrix)
    }

    fn prenex_cnf_inner(&self, acc: &mut Vec<(Quantifier, Atom)>) -> Vec<Vec<Atom>> {
        match self {
            Formula::Exists(a, v) => {
                acc.push((Quantifier::Exists, *a));
                v.prenex_cnf_inner(acc)
            },
            Formula::Forall(a, v) => {
                acc.push((Quantifier::Forall, *a));
                v.prenex_cnf_inner(acc)
            },
            _ => {
                let (a, mut matrix) = self.tseitin(acc);
                matrix.push(vec![a]);
                matrix
            }
        }
    } 

    fn tseitin(&self, low: &mut Vec<(Quantifier, Atom)>) -> (Atom, Vec<Vec<Atom>>) {
        match self {
            Formula::Atom(v) => (*v, vec![]),
            Formula::And(a, b) => {
                let (aa, mut at) = a.tseitin(low);
                let (ba, bt) = b.tseitin(low);
                at.extend(bt);
                let ca = atom();
                low.push((Quantifier::Exists, ca));
                at.push(vec![ca, aa.invert(), ba.invert()]);
                at.push(vec![aa, ca.invert()]);
                at.push(vec![ba, ca.invert()]);
                eprintln!("-----\nf{}\nf{} {} {}", self, ca, aa.invert(), ba.invert());
                eprintln!("f{} {}", ca.invert(), ba);
                eprintln!("f{} {}", ca.invert(), aa);
                (ca, at)
            },
            Formula::Or(a, b) => {
                let (aa, mut at) = a.tseitin(low);
                let (ba, bt) = b.tseitin(low);
                at.extend(bt);
                let ca = atom();
                low.push((Quantifier::Exists, ca));
                at.push(vec![ca.invert(), aa, ba]);
                at.push(vec![aa.invert(), ca]);
                at.push(vec![ba.invert(), ca]);
                eprintln!("-----\nf{}\nf{} {} {}", self, ca.invert(), aa, ba);
                eprintln!("f{} {}", ca, ba.invert());
                eprintln!("f{} {}", ca, aa.invert());
                (ca, at)
            }
            _ => panic!("Disallowed in tseitin")
        }
    }
}

impl BitOr for Formula {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self::Or(Box::new(self), Box::new(rhs))
    }
}

impl BitAnd for Formula {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self::And(Box::new(self), Box::new(rhs))
    }
}

impl Not for Formula {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self::Not(Box::new(self))
    }
}

pub struct BitVector {
    pub bits: Vec<Atom>,
}

impl BitVector {
    #[track_caller]
    pub fn new(size: usize) -> Self {
        assert!(size > 0, "size must be positive");
        Self { bits: repeat_with(atom).take(size).collect() }
    }

    pub fn exists(&self, other: Formula) -> Formula {
        self.bits.iter().fold(other, |other, atom| atom.exists(other))
    }
    
    pub fn forall(&self, other: Formula) -> Formula {
        self.bits.iter().fold(other, |other, atom| atom.forall(other))
    }

    #[track_caller]
    pub fn equal(&self, mut val: u64) -> Formula {
        assert!(1 << self.bits.len() > val, "value overflowed bitsize");
        self.bits.iter().map(|atom| {
                let bit = val & 1 == 1;
                val >>= 1;
                if bit { !!*atom } else { !*atom }
            })
            .reduce(|a, b| a & b)
            .expect("bitvector is never empty")
    }

    #[track_caller]
    pub fn le(&self, val: u64) -> Formula {
        assert!(1 << self.bits.len() > val, "value overflowed bitsize");
        let mut form = if val & 1 == 1 {
            !!self.bits[0] | !self.bits[0]
        }
        else {
            !self.bits[0]
        };
        for shift in 1..self.bits.len() {
            let bit = (val >> shift) & 1 == 1;
            form = if bit {
                !self.bits[shift] | !!self.bits[shift] & form
            }
            else {
                !self.bits[shift] & form
            };
        }
        dbg!(&form);
        form
    }
}

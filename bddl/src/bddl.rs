#[derive(Debug, Clone)]
pub struct Condition {
    pub sub_cond: Vec<SubCondition>,
}

#[derive(Debug, Clone, Copy)]
pub enum SubCondition {
    Id {
        pred: Pred,
        x_e: E,
        y_e: E,
    },
    Not {
        pred: Pred,
        x_e: E,
        y_e: E,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pred {
    Open,
    White,
    Black,
}

#[derive(Debug, Clone, Copy)]
pub enum E {
    Add(i64),
    Sub(i64),
    Int(i64),
    Identity,
    Min,
    Max,
}

impl E {
    pub fn noramlize(&self, v: i64, max: i64) -> Option<i64> {
        match self {
            E::Add(u) => (v + u < max).then_some(v + u),
            E::Sub(u) => (v - u >= 0).then_some(v - u),
            E::Int(u) => Some(*u),
            E::Identity => Some(v),
            E::Min => Some(0),
            E::Max => Some(max - 1),
        }
    }
    
    pub fn noramlize_t(&self, v: i64, max: i64) -> i64 {
        match self {
            E::Add(u) => v + u,
            E::Sub(u) => v - u,
            E::Int(u) => *u,
            E::Identity => v,
            E::Min => 0,
            E::Max => max - 1,
        }
    }
}

#[derive(Debug)]
pub struct Domain {
    pub black_actions: Vec<Action>,
    pub white_actions: Vec<Action>,
}

#[derive(Debug, Clone)]
pub struct Action {
    pub name: String,
    pub precondition: Condition,
    pub effect: Condition,
}

#[derive(Debug)]
pub struct Problem {
    pub size: Size,
    pub init: Vec<InitPred>,
    pub depth: u64,
    pub white_goals: Vec<Condition>,
    pub black_goals: Vec<Condition>,
}

#[derive(Debug, Clone, Copy)]
pub struct Size {
    pub x: i64,
    pub y: i64,
}

#[derive(Debug)]
pub struct InitPred {
    pub pred: Pred,
    pub x: i64,
    pub y: i64,
}

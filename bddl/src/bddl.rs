#[derive(Debug)]
pub struct Condition {
    pub sub_cond: Vec<SubCondition>,
}

impl Condition {
    pub fn parse(src: &str) -> Option<Condition> {
        let sub_cond = src.split_whitespace().map(SubCondition::parse).collect::<Option<Vec<SubCondition>>>()?;
        Some(Condition { sub_cond })
    }
}

#[derive(Debug)]
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

impl SubCondition {
    pub fn parse(src: &str) -> Option<SubCondition> {
        unimplemented!()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pred {
    Open,
    White,
    Black,
}

impl Pred {
    pub fn parse(src: &str) -> Option<Pred> {
        match src {
            "open" => Some(Pred::Open),
            "white" => Some(Pred::White),
            "black" => Some(Pred::Black),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum E {
    Add(i64),
    Sub(i64),
    Identity,
    Min,
    Max,
}

#[derive(Debug)]
pub struct Domain {
    pub black_actions: Vec<Action>,
    pub white_actions: Vec<Action>,
}

#[derive(Debug)]
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

impl Problem {
    pub fn parse(src: &str) -> Option<Problem> {
        let (_, acc) = src.split_once("#boardsize")?;
        let (size, acc) = acc.split_once("#init")?;
        let (init, acc) = acc.split_once("#depth")?;
        let (depth, acc) = acc.split_once("#blackgoal")?;
        let (black_goal, white_goal) = acc.split_once("#whitegoal")?;
        let size = Size::parse(size)?;
        let init = init.split_whitespace().map(InitPred::parse).collect::<Option<Vec<InitPred>>>()?;
        let depth = depth.parse().ok()?;
        // This is actually split by parenthesis and not lines and i cannot bother now so
        todo!();
        let white_goals = white_goal.lines().map(Condition::parse).collect::<Option<Vec<Condition>>>()?;
        let black_goals = black_goal.lines().map(Condition::parse).collect::<Option<Vec<Condition>>>()?;
        Some(Problem { size, init, depth, white_goals, black_goals })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Size {
    pub x: i64,
    pub y: i64,
}

impl Size {
    pub fn parse(src: &str) -> Option<Size> {
        let mut iter = src.split_whitespace();
        let x = iter.next()?.parse().ok()?;
        let y = iter.next()?.parse().ok()?;
        Some(Size { x, y })
    }
}

#[derive(Debug)]
pub struct InitPred {
    pub pred: Pred,
    pub x: i64,
    pub y: i64,
}

impl InitPred {
    pub fn parse(src: &str) -> Option<InitPred> {
        let (pred, rest) = src.split_once("(")?;
        let (x, rest) = rest.split_once(",")?;
        let (y, _) = rest.split_once(")")?;
        let x = x.parse().ok()?;
        let y = y.parse().ok()?;
        let pred = Pred::parse(pred)?;
        Some(InitPred { pred, x, y })
    }
}
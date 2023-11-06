use super::bddl::*;

pub fn domain() -> Domain {
    let black_actions = Action {
        name: "fill".to_owned(),
        precondition: Condition {
            sub_cond: vec![SubCondition::Id { pred: Pred::Open, x_e: E::Identity, y_e: E::Identity }],
        },
        effect: Condition {
            sub_cond: vec![SubCondition::Id { pred: Pred::Black, x_e: E::Identity, y_e: E::Identity }],
        },
    };
    let white_actions = Action {
        name: "fill".to_owned(),
        precondition: Condition {
            sub_cond: vec![SubCondition::Id { pred: Pred::Open, x_e: E::Identity, y_e: E::Identity }],
        },
        effect: Condition {
            sub_cond: vec![SubCondition::Id { pred: Pred::White, x_e: E::Identity, y_e: E::Identity }],
        },
    };
    Domain {
        black_actions: vec![black_actions],
        white_actions: vec![white_actions],
    }
}

fn goals(pred: Pred) -> Vec<Condition> {
    let horizontal = Condition {
        sub_cond: vec![
            SubCondition::Id { pred, x_e: E::Sub(1), y_e: E::Identity },
            SubCondition::Id { pred, x_e: E::Identity, y_e: E::Identity },
            SubCondition::Id { pred, x_e: E::Add(1), y_e: E::Identity },
        ],
    };
    let vertical = Condition {
        sub_cond: vec![
            SubCondition::Id { pred, x_e: E::Identity, y_e: E::Sub(1) },
            SubCondition::Id { pred, x_e: E::Identity, y_e: E::Identity },
            SubCondition::Id { pred, x_e: E::Identity, y_e: E::Add(1) },
        ],
    };
    let diag_1 = Condition {
        sub_cond: vec![
            SubCondition::Id { pred, x_e: E::Sub(1), y_e: E::Sub(1) },
            SubCondition::Id { pred, x_e: E::Identity, y_e: E::Identity },
            SubCondition::Id { pred, x_e: E::Add(1), y_e: E::Add(1) },
        ],
    };
    let diag_2 = Condition {
        sub_cond: vec![
            SubCondition::Id { pred, x_e: E::Add(1), y_e: E::Sub(1) },
            SubCondition::Id { pred, x_e: E::Identity, y_e: E::Identity },
            SubCondition::Id { pred, x_e: E::Sub(1), y_e: E::Add(1) },
        ],
    };
    vec![horizontal, vertical, diag_1, diag_2]
}

pub fn problem() -> Problem {
    let size = Size { x: 3, y: 3 };
    let blacks = vec![];
    let whites = vec![];
    let init = blacks.into_iter().map(|(x, y)| InitPred { pred: Pred::Black, x, y })
        .chain(whites.into_iter().map(|(x, y)| InitPred { pred: Pred::White, x, y }))
        .collect();

    Problem {
        size,
        init,
        depth: 5,
        white_goals: goals(Pred::White),
        black_goals: goals(Pred::Black),
    }
}

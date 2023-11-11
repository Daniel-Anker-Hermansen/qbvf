use logos::Logos;

#[derive(Logos, Clone, Debug)]
pub enum Token {
    #[regex(r"\s", logos::skip)]

    #[token("-")] Minus,
    #[token("+")] Plus,
    #[token("(")] Lparen,
    #[token(")")] Rparen,
    #[token("#")] Hash,
    #[token(":")] Colon,
    #[token(",")] Comma,
    #[token("?")] QuestionMark,

    #[token("blackactions")] BlackActions,
    #[token("whiteactions")] WhiteActions,
    #[token("action")] Action,
    #[token("parameters")] Parameters,
    #[token("precondition")] Precondition,
    #[token("open")] Open,
    #[token("white")] White,
    #[token("black")] Black,
    #[token("x")] X,
    #[token("y")] Y,
    #[token("xmin")] Xmin,
    #[token("xmax")] Xmax,
    #[token("ymin")] Ymin,
    #[token("ymax")] Ymax,
    #[token("boardsize")] Boardsize,
    #[token("init")] Init,
    #[token("depth")] Depth,
    #[token("blackgoals")]#[token("blackgoal")] BlackGoals,
    #[token("whitegoals")]#[token("whitegoal")] WhiteGoals,
    #[token("effect")] Effect,
    #[token("NOT")] Not,

    #[regex(r"\d+", |lex| lex.slice().parse().ok())] Int(i64),
    #[regex(r"[a-zA-Z]([a-zA-Z0-9]*)", |lex| lex.slice().to_owned())] String(String),
}

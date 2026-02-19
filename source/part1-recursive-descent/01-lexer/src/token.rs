#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Let,
    For,
    In,
    Fn,
    Type,
    Match,
    With,
    End,
    If,
    Then,
    Else,
    Return,
    Null,
    Ident(String),
    IntLit(i64),
    StrLit(String),
    Eq,
    EqEq,
    Ne,
    Arrow,
    Pipe,
    Lt,
    Le,
    Gt,
    Ge,
    Comma,
    Colon,
    Semicolon,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Dot,
    DotDot,
    DotDotEq,
    Typeof,
    Eof,
}

impl Token {
    pub fn is_keyword(s: &str) -> bool {
        matches!(
            s,
            "let"
                | "for"
                | "in"
                | "fn"
                | "type"
                | "match"
                | "with"
                | "end"
                | "if"
                | "then"
                | "else"
                | "return"
                | "null"
                | "typeof"
        )
    }
}

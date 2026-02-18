#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Let,
    Fn,
    Type,
    Match,
    With,
    End,
    If,
    Then,
    Else,
    Ident(String),
    IntLit(i64),
    Eq,
    Arrow,
    Pipe,
    Lt,
    Gt,
    Comma,
    Semicolon,
    LParen,
    RParen,
    Plus,
    Minus,
    Star,
    Eof,
}

impl Token {
    pub fn is_keyword(s: &str) -> bool {
        matches!(
            s,
            "let" | "fn" | "type" | "match" | "with" | "end" | "if" | "then" | "else"
        )
    }
}

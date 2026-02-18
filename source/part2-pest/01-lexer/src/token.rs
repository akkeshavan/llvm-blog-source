// Token type - same as Part 1 for pipeline compatibility

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

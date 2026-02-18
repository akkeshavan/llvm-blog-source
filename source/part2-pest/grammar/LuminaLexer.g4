// Part 2: ANTLR4 lexer grammar (reference - implementation uses pest)
// To use with ANTLR4: antlr4 -Dlanguage=Rust LuminaLexer.g4

lexer grammar LuminaLexer;

LET   : 'let' ;
FN    : 'fn' ;
TYPE  : 'type' ;
MATCH : 'match' ;
WITH  : 'with' ;
END   : 'end' ;
IF    : 'if' ;
THEN  : 'then' ;
ELSE  : 'else' ;

INT   : [0-9]+ ;
IDENT : [a-zA-Z_][a-zA-Z0-9_]* ;

EQ    : '=' ;
ARROW : '->' ;
PIPE  : '|' ;
LT    : '<' ;
GT    : '>' ;
COMMA : ',' ;
SEMI  : ';' ;
LPAREN: '(' ;
RPAREN: ')' ;
PLUS  : '+' ;
MINUS : '-' ;
STAR  : '*' ;

WS    : [ \t\r\n]+ -> skip ;

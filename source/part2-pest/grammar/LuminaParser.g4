// Part 2: ANTLR4 parser grammar (reference - implementation uses pest)
// To use with ANTLR4: antlr4 -Dlanguage=Rust LuminaParser.g4

parser grammar LuminaParser;

options { tokenVocab = LuminaLexer; }

program : expr EOF ;

expr : expr (PLUS | MINUS) expr   # AddSub
     | expr STAR expr             # Mul
     | INT                        # IntLit
     | IDENT                      # Var
     | LPAREN expr RPAREN         # ParenExpr
     ;

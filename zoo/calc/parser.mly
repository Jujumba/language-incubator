%{
    open Syntax
%}

%token <int> NUMERAL
%token PLUS
%token MINUS
%token TIMES
%token DIVIDE
%token UMINUS
%token LPAREN
%token RPAREN
%token EOF

%left PLUS MINUS
%left TIMES DIVIDE
%nonassoc UMINUS

%start toplevel
%type <Syntax.expression> toplevel

%%

toplevel: expression EOF { $1 }
;

expression:
    | NUMERAL                           { Numeral $1 }
    | expression TIMES  expression      { Times ($1, $3) }
    | expression PLUS   expression      { Plus  ($1, $3) }
    | expression MINUS  expression      { Minus ($1, $3) }
    | expression DIVIDE expression      { Divide ($1, $3) }
    | MINUS expression %prec UMINUS     { Negate $2 }
    | LPAREN expression RPAREN          { $2 }
;


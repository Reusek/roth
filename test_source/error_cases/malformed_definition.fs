( Test malformed function definitions )
( Incomplete or invalid definitions )

( Missing semicolon )
: INCOMPLETE DUP +

( Missing colon )
INVALID DUP + ;

( Nested definitions )
: OUTER : INNER + ; ;

( Empty definition )
: EMPTY ;
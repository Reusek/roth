( Enhanced Roth Forth Test )
( Tests new string support and enhanced features )

( Test string literals and output )
"Hello" TYPE SPACE "World" TYPE CR

( Test basic arithmetic )
5 3 + . CR

( Test conditionals with strings )
: TEST-IF ( n -- )
  DUP 0 > IF 
    "Positive" TYPE 
  ELSE 
    "Non-positive" TYPE 
  THEN 
  CR ;

5 TEST-IF
-3 TEST-IF

( Test loops with output )
: COUNT-UP ( n -- )
  0 DO 
    I . SPACE 
  LOOP 
  CR ;

5 COUNT-UP

( Test word definitions with strings )
: GREET ( -- )
  "Welcome to Roth Forth!" TYPE CR ;

GREET

( Test stack operations )
1 2 3 .S CR
DROP DROP DROP

( Test more complex operations )
: SQUARE-AND-TELL ( n -- )
  DUP DUP * 
  "Square of " TYPE 
  SWAP . 
  " is " TYPE 
  . CR ;

7 SQUARE-AND-TELL
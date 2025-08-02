( Test ; word )
( Function definition end )

: TRIPLE DUP DUP + + ;
: NEGATE 0 SWAP - ;
: ABS DUP 0 < IF NEGATE THEN ;

3 TRIPLE .
5 NEGATE .
-7 ABS .
CR
( Test ; word )
( Function definition end )

: TRIPLE DUP DUP + + ;
: MY-NEGATE 0 SWAP - ;
: MY-ABS DUP 0 < IF MY-NEGATE THEN ;

3 TRIPLE .
5 MY-NEGATE .
-7 MY-ABS .
CR
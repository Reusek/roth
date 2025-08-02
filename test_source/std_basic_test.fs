( Basic STD Library Test )
( Tests core functionality without advanced features )

( Test basic arithmetic )
5 1 + .
5 1 - .
CR

( Test comparisons )
0 0 = .
5 0 = .
-3 0 < .
7 0 > .
CR

( Test simple loop )
5 0 DO I . LOOP
CR

( Test simple word definitions )
: DOUBLE DUP + ;
: TRIPLE DUP DUP + + ;

5 DOUBLE .
4 TRIPLE .
CR

( Test stack operations )
1 2 3 .S
CR

( Test more arithmetic )
6 DUP * .
3 4 + 5 * .
CR

( Test sum of squares without recursion )
: SUM-OF-SQUARES 
  DUP * SWAP DUP * + ;

3 4 SUM-OF-SQUARES .
CR

( Test simple conditional )
: ABS-SIMPLE
  DUP 0 < IF NEGATE THEN ;

-5 ABS-SIMPLE .
7 ABS-SIMPLE .
CR
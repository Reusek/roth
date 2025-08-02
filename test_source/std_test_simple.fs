( STD Library Simple Test )
( Tests basic STD library functionality using only supported syntax )

( Test basic arithmetic and stack operations )
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

( Test word definitions )
: DOUBLE DUP + ;
: TRIPLE DUP DUP + + ;

5 DOUBLE .
4 TRIPLE .
CR

( Test factorial )
: FACTORIAL 
  DUP 1 <= IF DROP 1 ELSE
    DUP 1- FACTORIAL *
  THEN ;

5 FACTORIAL .
CR

( Test sum of squares )
: SUM-OF-SQUARES 
  DUP * SWAP DUP * + ;

3 4 SUM-OF-SQUARES .
CR
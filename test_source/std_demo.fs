( STD Library Demonstration )
( Practical examples using the standard library )

INCLUDE std/std.rt

CR ." === STD Library Practical Demo ===" CR CR

( Example 1: Number classification )
: CLASSIFY-NUMBER ( n -- )
  DUP ." Number " . ." is: "
  DUP ZERO? IF ." zero " THEN
  DUP POSITIVE? IF ." positive " THEN
  DUP NEGATIVE? IF ." negative " THEN
  DUP EVEN? IF ." even " THEN
  DUP ODD? IF ." odd " THEN
  DROP CR ;

." Classifying numbers:" CR
5 CLASSIFY-NUMBER
-3 CLASSIFY-NUMBER  
0 CLASSIFY-NUMBER
8 CLASSIFY-NUMBER
CR

( Example 2: Stack manipulation demo )
: STACK-DEMO ( -- )
  ." Starting with: " 10 20 30 .S CR
  ." After 2DUP: " 2DUP .S CR
  ." After TUCK: " TUCK .S CR
  ." After NIP: " NIP .S CR
  ." Cleaning up..." DROP DROP DROP DROP CR ;

STACK-DEMO
CR

( Example 3: Mathematical calculations )
: DISTANCE ( x1 y1 x2 y2 -- distance² )
  ROT - SQUARE >R - SQUARE R> + ;

: HYPOTENUSE ( a b -- c² )
  SQUARE SWAP SQUARE + ;

." Distance² between (1,2) and (4,6): " 1 2 4 6 DISTANCE . CR
." Hypotenuse² of 3,4 triangle: " 3 4 HYPOTENUSE . CR
CR

( Example 4: Formatted output )
: PRINT-TABLE ( n -- )
  ." n" TAB ." n²" TAB ." n³" CR
  ." -" TAB ." --" TAB ." --" CR
  1+ 1 DO
    I . TAB I SQUARE . TAB I CUBE . CR
  LOOP ;

." Table of squares and cubes:" CR
5 PRINT-TABLE
CR

( Example 5: Conditional operations )
: SAFE-DIVIDE ( a b -- result )
  DUP ZERO? IF
    ." Division by zero!" CR DROP DROP 0
  ELSE
    /
  THEN ;

." Safe division 10/2: " 10 2 SAFE-DIVIDE . CR
." Safe division 10/0: " 10 0 SAFE-DIVIDE . CR
CR

( Example 6: Loop variations )
: PRINT-PATTERN ( n -- )
  ." Pattern with " DUP . ." iterations:" CR
  0 DO
    I 1+ 0 DO 42 EMIT LOOP CR
  LOOP ;

3 PRINT-PATTERN
CR

." === Demo Complete ===" CR
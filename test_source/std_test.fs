( STD Library Comprehensive Test )
( Tests all modules of the Roth Forth Standard Library )

( Load the standard library )
INCLUDE std/std.rt

CR ." === Roth Forth STD Library Test ===" CR CR

( Test Core Module )
." Testing CORE module..." CR
." Boolean constants: " TRUE . FALSE . CR
." Increment/decrement: " 5 1+ . 5 1- . CR
." Zero comparisons: " 0 0= . 5 0= . -3 0< . 7 0> . CR
." Predicates: " 5 POSITIVE? . -3 NEGATIVE? . 0 ZERO? . CR
CR

( Test Stack Module )
." Testing STACK module..." CR
." Original stack: " 1 2 3 .S CR
." After 2DUP: " 2DUP .S CR
." After 2DROP: " 2DROP .S CR
." Testing NIP: " 10 20 30 NIP .S CR
." Testing TUCK: " 40 50 TUCK .S CR
." Clear stack: " DROP DROP DROP DROP CR
CR

( Test Math Module )
." Testing MATH module..." CR
." ABS tests: " -5 ABS . 7 ABS . CR
." MIN/MAX tests: " 3 7 MIN . 3 7 MAX . CR
." SQUARE test: " 6 SQUARE . CR
." SIGN tests: " -5 SIGN . 0 SIGN . 8 SIGN . CR
." EVEN/ODD tests: " 4 EVEN? . 5 EVEN? . 4 ODD? . 5 ODD? . CR
CR

( Test I/O Module )
." Testing I/O module..." CR
." Spaces test: " 5 SPACES ." <-- 5 spaces" CR
." Character output: " 65 EMIT 66 EMIT 67 EMIT CR
CR

( Test Control Module )
." Testing CONTROL module..." CR
." Counting 5 times: " 5 0 DO I . LOOP CR
." Countdown from 3: " 3 COUNTDOWN CR
CR

( Test Complex Operations )
." Testing complex operations..." CR

( Factorial using STD library words )
: FACTORIAL ( n -- n! )
  DUP 1 <= IF DROP 1 ELSE
    DUP 1- FACTORIAL *
  THEN ;

." 5 factorial: " 5 FACTORIAL . CR

( Sum of squares using STD library )
: SUM-OF-SQUARES ( a b -- a²+b² )
  SQUARE SWAP SQUARE + ;

." Sum of squares 3,4: " 3 4 SUM-OF-SQUARES . CR

( Test range operations )
." Range test 5 in [1,10]: " 5 1 10 WITHIN . CR
." Range test 15 in [1,10]: " 15 1 10 WITHIN . CR

( Test mathematical helpers )
." Division with remainder 17/5: " 17 5 /MOD . ." remainder " . CR

( Test clamping )
." Clamp 15 to [5,10]: " 15 5 10 CLAMP . CR
." Clamp 2 to [5,10]: " 2 5 10 CLAMP . CR

CR ." === All STD Library Tests Complete ===" CR
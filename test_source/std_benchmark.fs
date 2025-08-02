( STD Library Performance Benchmark )
( Tests performance of STD library words vs compiler primitives )

INCLUDE std/std.rt

CR ." === STD Library Benchmark ===" CR CR

( Benchmark helper )
: BENCHMARK ( n xt -- )
  ." Running " OVER . ." iterations..." CR
  0 DO DUP EXECUTE LOOP DROP ;

( Test 1: Stack operations performance )
: STACK-TEST ( -- )
  1 2 3 2DUP 2DROP NIP ;

." Stack operations test:" CR
1000 ' STACK-TEST BENCHMARK
CR

( Test 2: Math operations performance )
: MATH-TEST ( -- )
  -5 ABS 3 7 MIN 4 SQUARE + + DROP ;

." Math operations test:" CR
1000 ' MATH-TEST BENCHMARK
CR

( Test 3: Comparison operations )
: COMPARE-TEST ( -- )
  5 POSITIVE? DROP
  0 ZERO? DROP
  -3 NEGATIVE? DROP ;

." Comparison operations test:" CR
1000 ' COMPARE-TEST BENCHMARK
CR

( Test 4: Complex calculation )
: COMPLEX-CALC ( -- )
  5 DUP SQUARE SWAP 1+ SQUARE + ABS DROP ;

." Complex calculation test:" CR
500 ' COMPLEX-CALC BENCHMARK
CR

( Test 5: Fibonacci using STD library )
: FIB-STD ( n -- fib )
  DUP 2 < IF DROP 1 ELSE
    DUP 1- RECURSE SWAP 2- RECURSE +
  THEN ;

." Fibonacci calculation (recursive):" CR
." Fib(10) = " 10 FIB-STD . CR
CR

( Test 6: Iterative Fibonacci using STD library )
: FIB-ITER-STD ( n -- fib )
  DUP 2 < IF DROP 1 ELSE
    0 1 ROT 2- 0 DO OVER + SWAP LOOP NIP
  THEN ;

." Fibonacci calculation (iterative):" CR
." Fib(20) = " 20 FIB-ITER-STD . CR
CR

." === Benchmark Complete ===" CR
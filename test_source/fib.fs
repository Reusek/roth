: fib-iter ( n -- f )
  0 1 rot 0 ?do over + swap loop drop ;

20 fib-iter .

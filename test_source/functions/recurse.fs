( Test RECURSE word )
( Recursive function calls )

: FACTORIAL 
  DUP 1 <= IF 
    DROP 1 
  ELSE 
    DUP 1- RECURSE * 
  THEN ;

: COUNTDOWN 
  DUP . 
  DUP 0 > IF 
    1- RECURSE 
  ELSE 
    DROP 
  THEN ;

5 FACTORIAL .
CR
3 COUNTDOWN
CR
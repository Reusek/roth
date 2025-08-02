( Working Test - Basic Operations )

( Simple arithmetic )
5 1 + .
CR

( Stack operations )
3 DUP * .
CR

( Simple word definition )
: DOUBLE DUP + ;
5 DOUBLE .
CR

( Loop test )
3 0 DO I . LOOP
CR
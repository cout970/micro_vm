main:
call fibonacci
dbg z, 255
exit
nop

fibonacci:
set a, 1
set b, 1
set d, 2000
loop:
dbg a, 0
mov c, a
mov a, b
add b, c
lt b, d
then
jmp loop
ret

; Some ideas

; !macro if(cond, ifTrue, ifFalse):
; ${cond}
; else
; jmp ${macro_id}_else
; ${ifTrue}
; jmp end
; ${macro_id}_else:
; ${ifFalse}
; ${macro_id}_end:
; !endmacro

; fibonacci:
;     push a
;     push b
;     push c
;     push d
;     set a, 1
;     set b, 1
;     set d, 2000
; loop:
;     mov c, a
;     mov a, b
;     add b, c
;     lt b, d
;     then
;     jmp loop
;     pop d
;     pop c
;     pop b
;     pop a
;     ret

; fun fibonacci($a, $b) use ($c, $d) {
;     push $a
;     push $b
;     push $c
;     push $d
;     set $a, 1
;     set $b, 1
;     set $d, 2000
; ${fun}_loop:
;     mov $c, $a
;     mov $a, $b
;     add $b, $c
;     lt $b, $d
;     then
;     jmp ${fun}_loop
;     pop $d
;     pop $c
;     pop $b
;     pop $a
;     ret
; }

; call fibonacci(a, b) use (c, d)
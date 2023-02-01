/* 
    SPDX-License-Identifier: Mozilla Public License 2.0
    Copyrigth (c) 2022-2023 François-Frédéric Ozog
    
*/

.text
    b       1f

// update with actual barekit size
payload_size:
    .long   2200000

// platform dependent
sec_mem_base:
    .quad   0x0e000000

1:
    adrp    x4, 0
    add     x1, x4, #4096
    ldr     x0, [x4, #8]
    mov     x5, x0
    ldr     w2, [x4, #4]

m_loop16:
	cmp     x2, #16
	b.lo	m_loop1
	ldp     x3, x4, [x1], #16
	stp     x3, x4, [x0], #16
	sub     x2, x2, #16
	b       m_loop16
/* copy byte per byte */
m_loop1:
	cbz     x2, m_end
	ldrb	w3, [x1], #1
	strb	w3, [x0], #1
	subs	x2, x2, #1
	b.ne	m_loop1

m_end:

    isb

    mov     x0, xzr
    mov     x1, xzr
    mov     x2, xzr
    mov     x3, xzr

    br      x5


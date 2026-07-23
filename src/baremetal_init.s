/*
 * AArch64 Universal Early Initialization (EL3 / EL2 / EL1)
 *
 * Rust is not that easy to deal with baremetal:
 *     - Reompile core with no floating-point and neon instructions (hence the complex build)
 *     - unaligned memory operation, even with core:ptr:unaligned_read and alignment check 
 *       disabled in SCTLR, can cause alignment exception, because MMU off means essentially Dev memory 
 *        which translates into unaligned access cause exception
 * So, when not started from EFI, we create an EFI-like memory environment: identity mapping of 
 * first 4GB of address space as normal RAM and some of it as MMIO.
 *
 * Phase 1: Identity-maps 4GB (0x0 to 0x1_0000_0000) as Normal RAM (MAIR[2]).
 * Phase 2: Overwrites MMIO region (MMIO_START to MMIO_START + MMIO_SPACE)
 *          in 2MB chunks with Device attributes (MAIR[0] + UXN/PXN).
 * Phase 3: Runtime Exception Level detection & configuration.
 */


/* ======================================================================
   Global ASM Friendly Macros (no C preprocessor)
   ====================================================================== */

/* 1. Function Start Definition */
.macro FUNC_START name
    .global \name
    .p2align 2
\name:
.endm

/* 2. Function End Definition */
.macro FUNC_END name
.endm

/* 3. Load nearby symbol address */
.macro LOAD_ADDR reg, sym
    adr     \reg, \sym
.endm



/* ----------------------------------------------------------------------
   MMIO Region Configuration
   ---------------------------------------------------------------------- */
.equ MMIO_START, 0x08000000               /* Start PA of MMIO space */
.equ MMIO_SPACE, (128 * 1024 * 1024)       /* Size of MMIO space (128MB = 64 x 2MB blocks) */


FUNC_START _install_exception_table_vbar
        adr     x0, _exception_table
        mrs     x1, CurrentEL
        ubfx    x1, x1, #2, #2
        cmp     x1, #3
        b.eq    1f
        cmp     x1, #2
        b.eq    2f
        msr     VBAR_EL1, x0
        b       3f
1:
        msr     VBAR_EL3, x0
        b       3f
2:
        msr     VBAR_EL2, x0
3:
        dsb     sy
        isb
        ret
FUNC_END _install_exception_table_vbar

FUNC_START _el_init_for_rust

    // just preserve the return address, don't make a stack frame as there may be issues with stack itself
    mov x27,x30
    bl _install_exception_table_vbar
    mov x30, x27

  /* ==================================================================
     * BUILD PAGE TABLES (Common for all Exception Levels)
     * Scratch Registers Used: x20, x21, x22, x23, x24
     * Preserved Registers:    x0 - x18
     * ================================================================== */

    /* ------------------------------------------------------------------
     * Step 1: Link L1 Table to L2 Tables (0 to 4GB)
     * ------------------------------------------------------------------ */
    LOAD_ADDR x20, _l1_table

    /* Link L1[0..3] -> l2_table_0..3 */
    LOAD_ADDR x21, _l2_table_0
    orr     x22, x21, #0x3       /* Valid Table Descriptor (Bits[1:0] = 0b11) */
    str     x22, [x20, #(0 * 8)]

    LOAD_ADDR x21, _l2_table_1
    orr     x22, x21, #0x3
    str     x22, [x20, #(1 * 8)]

    LOAD_ADDR x21, _l2_table_2
    orr     x22, x21, #0x3
    str     x22, [x20, #(2 * 8)]

    LOAD_ADDR x21, _l2_table_3
    orr     x22, x21, #0x3
    str     x22, [x20, #(3 * 8)]

    /* ------------------------------------------------------------------
     * Step 2: Phase 1 - Map full 4GB as Normal RAM (MAIR[2])
     * Attribute: Block(01) | MAIR[2](010) | SH_ISH(11) | AF(1) = 0x705
     * ------------------------------------------------------------------ */
    mov     x21, #0x705
    LOAD_ADDR x22, _l2_table_0
    mov     x23, #2048          /* 4 tables x 512 entries = 2048 blocks */

fill_ram_loop:
    str     x21, [x22], #8
    add     x21, x21, #(2 * 1024 * 1024)
    subs    x23, x23, #1
    b.ne    fill_ram_loop

    /* ------------------------------------------------------------------
     * Step 3: Phase 2 - Overwrite MMIO region with MAIR[0] + UXN/PXN
     * ------------------------------------------------------------------ */
    ldr     x20, =MMIO_START
    ldr     x21, =MMIO_SPACE
    cbz     x21, setup_mmu_by_el

    /* Align base address to 2MB boundary */
    lsr     x20, x20, #21
    lsl     x20, x20, #21

    /* Calculate starting descriptor entry address in l2_table_0 */
    LOAD_ADDR x22, _l2_table_0
    lsr     x23, x20, #21
    add     x22, x22, x23, lsl #3

    /* Calculate block count */
    mov x28, #(2 * 1024 * 1024 - 1)
    add     x21, x21, x28
    lsr     x23, x21, #21

    /* Attribute: Block(01) | MAIR[0](000) | SH_OSH(10) | AF(1) | UXN(1) | PXN(1) */
    movz    x24, #0x0701
    movk    x24, #0x0060, lsl #48
    orr     x24, x20, x24

fill_mmio_loop:
    str     x24, [x22], #8
    add     x24, x24, #(2 * 1024 * 1024)
    subs    x23, x23, #1
    b.ne    fill_mmio_loop

/* ======================================================================
   EXCEPTION LEVEL DETECTION & MMU SETUP
   Scratch Registers Used: x20
   Preserved Registers:    x0 - x18
   ====================================================================== */
setup_mmu_by_el:
    mrs     x20, CurrentEL
    lsr     x20, x20, #2        /* Extract EL bits [3:2] */
    cmp     x20, #3
    b.eq    init_el3
    cmp     x20, #2
    b.eq    init_el2
    cmp     x20, #1
    b.eq    init_el1
    b       .                   /* Fallback trap if EL0 */

/* ----------------------------------------------------------------------
   EL3 Configuration
   ---------------------------------------------------------------------- */
init_el3:
    /* MAIR_EL3 */
    ldr x20, =0x00FF0400
    msr     mair_el3, x20

    /* TCR_EL3 (PS=bits[18:16]=5 -> 48-bit PA) */
    ldr     x20, =0x00053520
    msr     tcr_el3, x20

    /* TTBR0_EL3 */
    LOAD_ADDR x20, _l1_table
    msr     ttbr0_el3, x20
    isb

    /* TLB & Cache Invalidate */
    tlbi    alle3
    dsb     ish
    ic      ialluis
    dsb     ish
    isb

    /* SCTLR_EL3 - Setup but no MMU yet */
    /* Page tables are set up, but we'll enable MMU later */
    mrs     x20, sctlr_el3
    bic     x20, x20, #(1 << 1)  /* A: Alignment check disable */
    /* Don't enable WXN yet - we need to execute from writable pages */
    msr     sctlr_el3, x20
    isb
    ret

/* ----------------------------------------------------------------------
   EL2 Configuration
   ---------------------------------------------------------------------- */
init_el2:
    /* MAIR_EL2 */
    ldr x20, =0x00FF0400
    msr     mair_el2, x20

    /* TCR_EL2 (PS=bits[18:16]=5 -> 48-bit PA) */
    ldr     x20, =0x00053520
    msr     tcr_el2, x20

    /* TTBR0_EL2 */
    LOAD_ADDR x20, _l1_table
    msr     ttbr0_el2, x20
    isb

    /* TLB & Cache Invalidate */
    tlbi    alle2
    dsb     ish
    ic      ialluis
    dsb     ish
    isb

    /* SCTLR_EL2 - Setup but no MMU yet */
    /* Page tables are set up, but we'll enable MMU later */
    mrs     x20, sctlr_el2
    bic     x20, x20, #(1 << 1)  /* A: Alignment check disable */
    /* Don't enable WXN yet - we need to execute from writable pages */
    msr     sctlr_el2, x20
    isb
    ret

/* ----------------------------------------------------------------------
   EL1 Configuration
   ---------------------------------------------------------------------- */
init_el1:
    /* MAIR_EL1 */
    
    ldr x20, =0x00FF0400
    msr     mair_el1, x20

    /* TCR_EL1 (IPS=bits[34:32]=5 -> 48-bit PA) */
    ldr     x20, =0x0000000500003520
    msr     tcr_el1, x20

    /* TTBR0_EL1 */
    LOAD_ADDR x20, _l1_table
    msr     ttbr0_el1, x20
    isb

    /* TLB & Cache Invalidate */
    tlbi    vmalle1is
    dsb     ish
    ic      ialluis
    dsb     ish
    isb

    /* SCTLR_EL1 - Setup but no MMU yet */
    /* Page tables are set up but MMU not enabled yet.
       We need to execute from writable pages, so WXN must be 0. */
    mrs     x20, sctlr_el1
    bic     x20, x20, #(1 << 1)   /* A: Alignment check disable */
    bic     x20, x20, #(1 << 19)  /* WXN: Allow execution on writable pages */
    msr     sctlr_el1, x20
    isb
    ret

FUNC_END _el_init_for_rust


.globl _exception_table
.align 12

.macro VECTOR_SLOT_HVC
.Lslot_hvc_\@:
        brk #1
        wfi
        movk w0, #0x8400 , lsl #16
        hvc #0
        .space 0x80 - (. - .Lslot_hvc_\@)
.endm

.macro VECTOR_SLOT_SMC
.Lslot_smc_\@:
        brk #1
        wfi
        movk w0, #0x8400 , lsl #16
        smc #0
        .space 0x80 - (. - .Lslot_smc_\@)
.endm

.macro VECTOR_SLOT_HVC_0x8
.Lslot_hvc8_\@:
        brk #1
        wfi
        mov w0, #0x0008
        movk w0, #0x8400 , lsl #16
        hvc #0
        .space 0x80 - (. - .Lslot_hvc8_\@)
.endm

_exception_table:

.globl _trap_exception
_trap_exception:
        VECTOR_SLOT_SMC
        VECTOR_SLOT_HVC
        VECTOR_SLOT_HVC
        VECTOR_SLOT_HVC
        VECTOR_SLOT_HVC
        VECTOR_SLOT_HVC
        VECTOR_SLOT_HVC
        VECTOR_SLOT_HVC
        VECTOR_SLOT_HVC_0x8
        VECTOR_SLOT_HVC_0x8
        VECTOR_SLOT_HVC_0x8
        VECTOR_SLOT_HVC_0x8
        VECTOR_SLOT_HVC_0x8
        VECTOR_SLOT_HVC_0x8




/* ======================================================================
   DYNAMIC UNIFORM PAGE TABLES 
   ====================================================================== */


.align 12                       /* 4KB page boundary aligned */

_l1_table:
    .space 4096                 /* 512 x 8-byte entries for L1 */

_l2_table_0:                     /* Mappings for 0x0000_0000 - 0x3FFF_FFFF (0 to 1GB) */
    .space 4096

_l2_table_1:                     /* Mappings for 0x4000_0000 - 0x7FFF_FFFF (1GB to 2GB) */
    .space 4096

_l2_table_2:                     /* Mappings for 0x8000_0000 - 0xBFFF_FFFF (2GB to 3GB) */
    .space 4096

_l2_table_3:                     /* Mappings for 0xC000_0000 - 0xFFFF_FFFF (3GB to 4GB) */
    .space 4096



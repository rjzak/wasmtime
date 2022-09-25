// A WORD OF CAUTION
//
// This entire file basically needs to be kept in sync with itself. It's not
// really possible to modify just one bit of this file without understanding
// all the other bits. Documentation tries to reference various bits here and
// there but try to make sure to read over everything before tweaking things!

use wasmtime_asm_macros::{asm_func, asm_sym};

// fn(top_of_stack(r1): *mut u8)
asm_func!(
    "wasmtime_fiber_switch",
    "
        // We're switching to arbitrary code somewhere else, so pessimistically
        // assume that all callee-save register are clobbered. This means we need
        // to save/restore all of them.
        //
        // Note that this order for saving is important since we use CFI directives
        // below to point to where all the saved registers are.
        //
        // Adapted from 64-bit ELF V2 ABI Specification document, revision 1.4
        
        std r14, -144(r1)
        std r15, -136(r1)
        std r16, -128(r1)
        std r17, -120(r1)
        std r18, -112(r1)
        std r19, -104(r1)
        std r20, -96(r1)
        std r21, -88(r1)
        std r22, -80(r1)
        std r23, -72(r1)
        std r24, -64(r1)
        std r25, -56(r1)
        std r26, -48(r1)
        std r27, -40(r1)
        std r28, -32(r1)
        std r29, -24(r1)
        std r30, -16(r1)
        std r31, -8(r1)
        std r0, 16(r1)

        // Load pointer that we're going to resume at and store where we're going
        // to get resumed from. This is in accordance with the diagram at the top
        // of unix.rs.
        std r1, 24(r1)
        std r2, 32(r1)
        
        // Swap stacks and restore all our callee-saved registers
        ld r14,-144(r1)
        ld r15,-136(r1)
        ld r16,-128(r1)
        ld r17,-120(r1)
        ld r18,-112(r1)
        ld r19,-104(r1)
        ld r20,-96(r1)
        ld r21,-88(r1)
        ld r22,-80(r1)
        ld r23,-72(r1)
        ld r24,-64(r1)
        ld r25,-56(r1)
        ld r26,-48(r1)
        ld r27,-40(r1)
        ld r28,-32(r1)
        ld r0, 16(r1)
        ld r29,-24(r1)
        mtlr r0
        ld r30,-16(r1)
        ld r31,-8(r1)
        //ld r0, 16(r1)
        blr
    "
);

// fn(
//    top_of_stack(r1): *mut u8,
//    entry_point(rsi): extern fn(*mut u8, *mut u8),
//    entry_arg0(rdx): *mut u8,
// )
#[rustfmt::skip]
asm_func!(
    "wasmtime_fiber_init",
    "
        // Here we're going to set up a stack frame as expected by
        // `wasmtime_fiber_switch`. The values we store here will get restored into
        // registers by that function and the `wasmtime_fiber_start` function will
        // take over and understands which values are in which registers.
        //
        // The first 16 bytes of stack are reserved for metadata, so we start
        // storing values beneath that.
        lea rax, ", asm_sym!("wasmtime_fiber_start"), "[rip]
        mov -0x18[rdi], rax
        mov -0x20[rdi], rdi   // loaded into rbp during switch
        mov -0x28[rdi], rsi   // loaded into rbx during switch
        mov -0x30[rdi], rdx   // loaded into r12 during switch

        // And then we specify the stack pointer resumption should begin at. Our
        // `wasmtime_fiber_switch` function consumes 6 registers plus a return
        // pointer, and the top 16 bytes are reserved, so that's:
        //
        //	(6 + 1) * 16 + 16 = 0x48
        lea rax, -0x48[rdi]
        mov -0x10[rdi], rax
        ret
    ",
);

use core::arch::asm;

pub fn delay_cycles_short(mut cycles: u8) {
    unsafe {
        asm!(
            "1:",
            "subi {r0}, 2",
            "brcc 1b",

            r0 = inout(reg_upper) cycles,
        )
    }
}

/// Арифметический сдвиг вправо.
pub fn sar(value: u32, shift: usize) -> u32 {
    ((value as i32) >> (shift & 31)) as u32
}

/// Знаковое расширение до 32 бит.
pub fn sign_extend(value: u8) -> u32 {
    value as i8 as i32 as u32
}
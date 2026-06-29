/// # Reversed by @flush2002 aka @RichardWGNR
/// # IC204 ASM version.
fn main() {
}

struct Software {
    pub part_number: &'static str,
    pub salt: &'static str,
    pub pairs: Vec<(u8, [u8; 8], [u8; 8])>,
}

fn sar(value: u32, shift: usize) -> u32 {
    ((value as i32) >> (shift & 31)) as u32
}

fn sign_extend(value: u8) -> u32 {
    value as i8 as i32 as u32
}

impl Software {
    fn calc_key(&self, level: u8, seed: [u8; 8]) -> Option<[u8; 8]> {
        let mut stack: [u8; 24] = [0; 24];

        // r27 - sf_unlock_idx
        let (_, sf_unlock_idx) = Self::get_subfunction_index(level)?;

        stack[16] = seed[7]; // sld.bu 0x7[ep],r11 | st.b r11,0x10[sp]
        stack[17] = seed[4]; // sld.bu 0x4[ep],r10 | st.b r10,0x11[sp]
        stack[18] = seed[3]; // sld.bu 0x3[ep],r9 | st.b param4,0x12[sp]
        stack[19] = seed[6]; // sld.bu 0x6[ep],r8 | st.b param3,0x13[sp]
        stack[20] = seed[5]; // sld.bu 0x5[ep],r7 | st.b param2,0x14[sp]
        stack[21] = seed[1]; // sld.bu 0x1[ep],r6 | st.b param1,0x15[sp]
        stack[22] = seed[0]; // sld.bu 0x0[ep],r12 | st.b r12,0x16[sp]
        stack[23] = seed[2]; // sld.bu 0x2[ep],r11 | st.b r11,0x17[sp]

        // mov r27,r10 <- r10 индекс подфункции
        // add sp,r10 <- r10 stack[индекс подфункции]
        // ld.b 0x10[r10],r8 <- грузим байт stack[16 + индекс подфункции] в регистр r8

        let r8 = crate::sign_extend(stack[16 + sf_unlock_idx as usize]); // mov r8,r2 <- r2 буфер seed, так как дальше идут перестановки

        // movea 0x10,sp,r9 <- r9 указывает на stack[16] - мертвый код
        // mov r27,r7 <- r7 теперь индекс подфункции и уйдет в salt fun

        let r28 = (r8 & 0x07).wrapping_add(2) & 0xFF; // andi 0x7,r8,r28 | add 0x2,r28 | zxb r28

        // movea 0x8,sp,r6 <- r6 = sp+8 указатель на текущий стек куда мы кладём соль, передающийся в recombine salt
        stack[8..16].copy_from_slice(&self.recombine_salt(sf_unlock_idx));

        let mut r29 = 0;
        loop {
            // LAB_4AAF8
            if r29 >= r28 { // был косяк
                break;
            }

            // LAB_4AA88
            // mov ep,r13 <- бэкап ep
            // mov sp,ep <- ep указывает на stack
            let r10 = crate::sign_extend(stack[8]); // sld.b 0x8[ep],r10
            let r12 = crate::sign_extend(stack[9]); // sld.b 0x9[ep],r12
            let r7 = crate::sign_extend(stack[10]); // sld.b 0xA[ep],r7
            let r8 = crate::sign_extend(stack[11]); // sld.b 0xB[ep],r8

            let r6 = crate::sar(r10 & 0x08, 3); // andi 0x8,r10,r61 | sar 0x3,r6 !!! арифметический сдвиг unsigned байта по идее даст логический сдвиг ! был косяк
            let r11 = (r12 & 0x01) ^ r6; // andi 0x1,r12,r11 | xor r61,r11

            let r9 = crate::sar(r7 & 0x02, 1); // andi 0x2,r7,r9 | sar 0x1,r9
            let r9 = r9 ^ r11; // xor r11,r9

            let r10 = crate::sar(r8 & 0x80, 7); // andi 0x80,r8,r10 | sar 0x7,r10
            let r10 = r10 ^ r9; // xor r9,r10
            // nop
            let r9 = crate::sign_extend(stack[12]); // sld.b 0xC[ep],r9

            let r11 = crate::sar(r9 & 0x20, 5); // andi 0x20,r9,r11 | sar 0x5,r11
            let r11 = r11 ^ r10; // xor r10,r11

            let r2 = crate::sign_extend(stack[15]); // sld.b 0xF[ep],r2
            let r10 = crate::sign_extend(stack[13]); // sld.b 0xD[ep],r2

            let r9 = r2 & 0x10; // andi 0x10,r2,r9

            let r12 = crate::sar(r10 & 0x04, 2) ^ r11; // andi 0x4,r10,r12 | sar 0x2,r12 | xor r12,r11
            let r9 = crate::sar(r9, 4); // sar 0x4,r9

            let r11 = crate::sign_extend(stack[14]); // sld.b 0xE[ep],r11

            let r6 = r2 & 0x7F; // andi 0x7F,r2,r61
            let r8 = r9; // mov r9,r8

            let r7 = crate::sar(r11 & 0x40, 6) ^ r12; // andi 0x40,r11,r7 | sar 0x6,r7 | xor r12,r7

            let r8 = r8 ^ r7; // xor r7,r8
            let r7 = r8 << 7; // mov r8,r7 | shl 0x7,r7

            let r6 = r6 | r7; // or r7,r6

            stack[15] = r6 as _; // sst.b r61,0xF[ep]
            // mov r13,ep <- восстановили ep

            // movea 0x8,sp,r61 <- r6 указывает на stack + 8
            // mov 0x8,r7 <- r7 = 8
            Self::bit_rotations(&mut stack[8..16], 8); // jarl MaybeRotate,lp

            r29 = r29 + 1;
        }

        // LAB_4AAFE
        for i in 0..8 { // mov 0x0,r29 | add 0x1,r29 | zxb r29 | cmp 0x8,r29 | bc LAB_4AAFE <- r29 счетчик цикла
            // mov r29,r12 <- r12 счетчик цикла
            // add sp,r12 <- r12 указывает на stack + i
            let r10 = stack[16 + i] as u32; // ld.bu 0x10[r12],r10
            // mov sp, r7 <- r7 указывает на stack
            // add r29, r7 <- r7 указывает на stack + i
            stack[i] = r10 as u8; // st.b r10,0x0[param2]
        }

        // mov 0x0,r26
        let mut r26 = 0;
        loop {
            // LAB_4AB18

            // LAB_4AB1A
            for i in 0..4 { // mov 0x0,r29 | add 0x1,r29 | zxb r29 | cmp 0x4,r29 | bc LAB_4AB1A <- r29 счетчик цикла
                // movea 0x10,sp,r8 <- r8 указывает на stack + 16
                // mov r29,r12 <- r12 счетчик цикла
                // add r8,r12 <- r12 указывает на stack + 16 + i
                let r6 = stack[16 + 4 + i] as u32; // ld.bu 0x4[r12],r61 <- ld.bu читает по оффсету +0x4 от r12
                // mov r29,r2 <- r2 счетчик цикла
                // add r8,r2 <- r2 указывает на stack + 16 + i
                let r9 = stack[16 + i] as u32; // ld.bu 0x0[r2],r9
                let r6 = r6 ^ r9; // xor r9,r61
                stack[16 + i] = r6 as u8; // st.b r61,0x0[r2] <- r2 указывает на stack + 16 + i + 0
            }
            // mov r27,r6 <- r6 указывает на MAGIC OFFSET
            // add sp,r6 <- r6 указывает на stack + MAGIC OFFSET
            let r11 = crate::sign_extend(stack[sf_unlock_idx as usize]); // ld.b 0x0[r6],r11 !!!!
            let r28 = (r11 & 0x03).wrapping_add(1) & 0xFF; // andi 0x3,r11,r28 | add 0x1,r28 | zxb r28

            let mut r29 = 0;
            loop {
                // LAB_4AB5E
                if r29 >= r28 {
                    break;
                }
                // LAB_4AB50
                // movea 0x10,sp,r61 <- r6 указывает на stack + 16
                // mov 0x4,r7 <- r7 = 0x4
                Self::bit_rotations(&mut stack [16..20], 4);
                r29 = r29 + 1; // add 0x1,r29 | zxb r29
            }

            let r6 = stack[19] as u32; // ld.bu 0x13[sp],r61
            let r11 = stack[18] as u32; // ld.bu 0x12[sp],r11
            let r8 = stack[17] as u32; // ld.bu 0x13[sp],r8
            let r12 = stack[16] as u32; // ld.bu 0x10[sp],r12

            let r6 = r6 << 24; // shl 0x18,r61
            let r11 = (r11 << 16).wrapping_add(r6); // shl 0x10,r11 | add r61,r11
            // !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
            let r8 = (r8 << 8).wrapping_add(r11); // shl 0x8,r8 | add r11,r8
            let r12 = r12.wrapping_add(r8); // add r8,r12
            // mov ep,r8 <- бэкапим ep
            // mov sp,ep <- ep указывает на stack
            //let r11 = (stack[11] as u32) << 24; // sld.bu 0xB[ep],r11 | shl 0x18,r11
            let r11 = stack[11] as u32; // sld.bu 0xB[ep],r11 <- !!! SHL позже
            let r7 = 0xFF00u32 << 16; // movhi -0x100,r0,r7
            //let r9 = ((stack[10] as u32) << 16).wrapping_add(r11); // sld.bu 0xA[ep],r9 | shl 0x10,r9 | add r11,r9
            let r9 = stack[10] as u32; // sld.bu 0xA[ep],r9 <- !!! SHL и ADD позже
            //let r6 = ((stack[9] as u32) << 8).wrapping_add(r9); // sld.bu 0x9[ep],r6 | shl 0x8,r6 | add r9,r6
            let r6 = stack[9] as u32; // sld.bu 0x9[ep],r6 <- !!! SHL и ADD позже
            let r11 = r11 << 24; // shl 0x18,r11
            //let r2 = (stack[8] as u32).wrapping_add(r6).wrapping_add(r12); // sld.bu 0x8[ep],r2 | add r6,r2 | add r12,r2
            let r2 = stack[8] as u32; // sld.bu 0x8[ep],r2 <- !!! SHL и ADD позже
            let r9 = (r9 << 16).wrapping_add(r11); // shl 0x10,r9 | add r11,r9
            let r6 = (r6 << 8).wrapping_add(r9); // shl 0x8,r6 | add r9,r6
            let r2 = r2.wrapping_add(r6).wrapping_add(r12); // add r6,r2 | add r12,r2
            let r7 = (r7 & r2) >> 24; // and r2,r7 | shr 0x18,r7
            stack[19] = r7 as _; // sst.b r7,0x13[ep]
            let r11 = (0xFFu32 << 16) & r2; // movhi 0xFF,r0,r11 | and r2,r11
            let r11 = r11 >> 16; // shr 0x10,r11
            stack[18] = r11 as _; // sst.b r11,0x12[ep]
            let r10 = (r2 & 0xFF00) >> 8; // andi 0xFF00,r2,r10 | shr 0x8,r10
            stack[17] = r10 as _; // sst.b r10,0x11[ep]
            stack[16] = r2 as _; // sst.b r2,0x10[ep]
            // mov r8,ep <- восстанавливаем ep

            // LAB_4ABBA
            for i in 0..4 { // mov 0x0,r29 | add 0x1,r29 | zxb r29 | cmp 0x4,r29 | bc LAB_4ABBA <- r29 счетчик цикла
                // mov sp,r10 <- r10 указывает на stack
                // add r29,r10 <- r10 указывает на stack + i
                let r8 = stack[i] as u32; // ld.bu 0x0[r10],r8
                // mov r29,r12 <- r12 указывает на stack
                // add sp,r12 <- r12 указывает на stack + i
                stack[20 + i] = r8 as u8; // st.b r8,0x14[r12]
            }

            // LAB_4ABD4
            for i in 0..8 { // mov 0x0,r29 | add 0x1,r29 | zxb r29 | cmp 0x8,r29 | bc LAB_4ABD4 <- r29 счетчик цикла
                // mov r29,r11 <- r11 счетчик цикла
                // add sp,r11 <- r11 указывает на stack + i
                let r9 = stack[16 + i] as u32; // ld.bu 0x10[r11],r9
                // mov sp,r61 <- r6 указывает на stack
                // add r29,r61 <- r6 указывает на stack + i
                stack[i] = r9 as u8; // st.b r9,0x0[r61]
            }

            // LAB_4ABEE
            for i in 0..4 { // mov 0x0,r29 | add 0x1,r29 | zxb r29 | cmp 0x4,r29 | bc LAB_4ABEE
                // movea 0x10,sp,r7 <- r7 указывает на stack + 16
                // mov r29,r11 <- r11 счетчик цикла
                // add r7,r11 <- r11 указывает на stack + 16 + i
                let r6 = stack[16 + 4 + i] as u32; // ld.bu 0x4[r11],r61
                // mov r29,r2 <- r2 счетчик цикла
                // add r7,r2 <- r2 указывает на stack + 16 + i
                let r8 = stack[16 + i] as u32; // ld.bu 0x0[r2],r8
                let r6 = r6 ^ r8; // xor r8,r61
                stack[16 + i] = r6 as u8; // st.b param1,0x0[r2]
            }
            // mov r27,r12 <- r12 указывает на MAGIC OFFSET
            // add sp,r12 <- указывает на stack + MAGIC OFFSET
            let r10 = crate::sign_extend(stack[sf_unlock_idx as usize]); // ld.b 0x0[r12],r10
            let r28 = (r10 & 0x03).wrapping_add(1) & 0xFF; // andi 0x3,r10,r28 | add 0x1,r28 | zxb r28

            let mut r29 = 0; // mov 0x0,r29
            loop {
                // LAB_4AC32
                if r29 >= r28 {
                    break;
                }

                // LAB_4AC24
                // movea 0x10,sp,r61 <- r6 указывает на stack + 16
                // mov 0x4,r7 <- r7 = 0x4
                Self::bit_rotations(&mut stack[16..20], 4); // jarl MaybeRotate,lp

                r29 = r29 + 1; // add 0x1,r29 | zxb r29
            }
            //let r12 = (stack[19] as u32) << 24; // ld.bu 0x13[sp],r12 | shl 0x18,r12
            let r12 = stack[19] as u32; // ld.bu 0x13[sp],r12 <- !!! SHL позже
            //let r10 = (stack[18] as u32) << 16; // ld.bu 0x12[sp],r10 | shl 0x10,r10
            let r10 = stack[18] as u32; // ld.bu 0x12[sp],r10 <- !!! SHL позже
            //let r7  = (stack[17] as u32) << 8; // ld.bu 0x11[sp],r7 | shl 0x8,r7
            let r7 = stack[17] as u32;
            let r11 = stack[16] as u32; // ld.bu 0x10[sp],r11
            let r12 = r12 << 24; // shl 0x18,r12
            let r10 = (r10 << 16).wrapping_add(r12); // shl 0x10,r10 | add r10,r12
            let r7 = (r7 << 8).wrapping_add(r10); // shl 0x8,r7 | add r10,r7
            let r11 = r11.wrapping_add(r7); // add r7,r11
            // mov ep,r7 <- бэкапим ep
            // mov sp,ep <- ep указывает на stack
            //let r10 = (stack[15] as u32) << 24; // sld.bu 0xF[ep],r10 | shl 0x18,r10 <- !!! SHL позже
            let r10 = stack[15] as u32; // sld.bu 0xF[ep],r10
            let r6 = 0xFF00u32 << 16; // movhi -0x100,r0,r6
            //let r8 = (stack[14] as u32) << 16; // sld.bu 0xE[ep],r8 | shl 0x10,r8 <- !!! SHL позже
            let r8 = stack[14] as u32; // sld.bu 0xE[ep],r8
            //let r12 = (stack[13] as u32)  << 8; // sld.bu 0xD[ep],r12 | shl 0x8,r12 <- !!! SHL позже
            let r12 = stack[13] as u32; // sld.bu 0xD[ep],r12
            let r10 = r10 << 24; // shl 0x18,r10
            let r2 = stack[12] as u32; // sld.bu 0xC[ep],r2
            let r8 = (r8 << 16).wrapping_add(r10); // shl 0x10,r8 | add r10,r8
            let r12 = (r12 << 8).wrapping_add(r8); // add r8,r12 | shl 0x8,r12
            let r2 = r2.wrapping_add(r12).wrapping_add(r11); // add r12,r2 | add r11,r2
            let r6 = (r6 & r2) >> 24; // and r2,r61 | shr 0x18,r61
            stack[19] = r6 as _; // sst.b r61,0x13[ep]
            let r10 = (0xFF << 16) & r2; // movhi 0xFF,r0,r10 | and r2,r10
            let r10 = r10 >> 16; // shr 0x10,r10
            stack[18] = r10 as _; // sst.b r10,0x12[ep]
            let r9 = (r2 & 0xFF00) >> 8; // andi 0xFF00,r2,r9 | shr 0x8,param4
            stack[17] = r9 as _; // sst.b r9,0x11[ep]
            stack[16] = r2 as _; // sst.b r2,0x10[ep]
            // mov param2,ep <- восстанавливаем ep

            // LAB_4AC8E
            for i in 0..4 { // mov 0x0,r29 | add 0x1,r29 | zxb r29 | cmp 0x4,r29 | bc LAB_4AC8E | <- r29 счетчик цикла
                // mov sp,r9 <- r9 указывает на stack
                // add r29,r9 <- r9 указывает на stack + i
                let r7 = stack[i] as u32; // ld.bu 0x0[r9],r7
                // mov r29,r11 <- r11 счетчик цикла
                // add sp,r11 <- r11 указывает на stack + i
                stack[20 + i] = r7 as u8; // sst.b r7,0x14[r11]
            }

            // LAB_4ACA8
            for i in 0..8 { // mov 0x0,r29 | add 0x1,r29 | zxb r29 | cmp 0x8,r29 | bc LAB_4ACA8
                // mov r29,r10 <- r10 счетчик цикла
                // add sp,r10 <- r10 указывает на stack + i
                let r8 = stack[16 + i] as u32; // ld.bu 0x10[r10],r8
                // mov sp,r12 <- r12 указывает на stack
                // mov r29,r12 <- r12 указывает на stack + i
                stack[i] = r8 as u8; // st.b r8,0x0[r12]
            }

            r26 = r26 + 1; // add 0x1,r26 | zxb r26
            if r26 == 2 || r26 > 2 { // cmp 0x2,r26 >= 2
                break; // bnc LAB_4ACCC
            }
            // jr LAB_4AB18
        }

        // LAB_4ACCC
        // mov ep,r13 <- бэкапим ep
        // mov sp,ep <- ep указывает на stack
        let r11 = stack[5] as u32; // sld.bu 0x5[ep],r11
        let r10 = stack[6] as u32; // sld.bu 0x6[ep],r10
        let r9  = stack[1] as u32; // sld.bu 0x1[ep],r9
        let r8  = stack[0] as u32; // sld.bu 0x0[ep],r8
        stack[17] = r11 as _; // sst.b r11,0x11[ep]
        let r7  = stack[7] as u32; // sld.bu 0x7[ep],r7
        let r12 = stack[3] as u32; // sld.bu 0x3[ep],r12
        stack[18] = r10 as _; // sst.b r10,0x12[ep]
        let r6  = stack[4] as u32; // sld.bu 0x4[ep],r61
        stack[19] = r9 as _; // sst.b r9,0x13[ep]
        stack[16] = r12 as _; // sst.b r12,0x10[ep]
        let r12 = stack[2] as u32; // sld.bu 0x2[ep],r12
        stack[20] = r8 as _; // sst.b r8,0x14[ep]
        stack[21] = r7 as _; // sst.b r7,0x15[ep]
        stack[22] = r6 as _; // sst.b r61,0x16[ep]
        stack[23] = r12 as _; // sst.b r12,0x17[ep]
        // mov r13,ep <- восстанавливаем ep

        Some([
            stack[16], stack[17], stack[18], stack[19],
            stack[20], stack[21], stack[22], stack[23]
        ])
    }

    /// Возвращает индексы подфункций запрошенного уровня.
    fn get_subfunction_index(level: u8) -> Option<(u8, u8)> {
        // В памяти прошивки таблица подфункций UDS сервиса хранится в виде массива
        //
        // 0: 01 00 00 00 9C C7 00 00
        // 1: 02 00 00 00 B0 C7 00 00
        // 2: 03 00 00 00 C4 C7 00 00
        // 3: 04 00 00 00 D8 C7 00 00
        // 4: 09 00 00 00 EC C7 00 00
        // 5: 0A 00 00 00 00 C8 00 00
        // 6: 0D 00 00 00 14 C8 00 00
        // 7: 0E 00 00 00 28 C8 00 00
        //
        // Если нас просят индексы подфункций уровня 0x9, то согласно таблице, этими индексами
        // будут являться числа: 4,5.
        // Это правило актуально и для других уровней.
        Some(match level {
            0x01 => (0,1),
            0x03 => (2,3),
            0x09 => (4,5),
            0x0D => (6,7),
            _ => None?
        })
    }

    fn bytes_salt(&self) -> Vec<u8> {
        assert_eq!(self.salt.len(), 64, "Invalid salt length");
        hex::decode(&self.salt).expect("Correct salt")
    }

    fn recombine_salt(&self, unlock_subfunction_idx: u8) -> [u8; 8] { // data r6, salt_idx r7
        let mut stack: [u8; 20] = [0; 20];

        // mov r61,r27 <- r27 указывает на внешний стек + 8, чтобы в конце записать соль
        // zxb r7 <- неизвестное
        // add -0x1,r7 <- убавляем единицу от него

        let subbed_r7 = unlock_subfunction_idx.wrapping_sub(1); // zxb r7:param_2 | add -0x1,r7 <- убавляем единицу от него

        let bytes_salt = self.bytes_salt();

        stack[12..20].copy_from_slice(&match subbed_r7 {
            0 => [0xB8, 0xCC, 0x61, 0xCC, 0x61, 0x18, 0x36, 0x26],
            1 => [0x53, 0xA8, 0x2C, 0xCF, 0x99, 0xEB, 0xFD, 0x58],
            2 => [0x3A, 0x72, 0x10, 0x33, 0x3D, 0xA2, 0x93, 0x10],
            3 => [0x6D, 0x2B, 0xFB, 0xF9, 0xCE, 0x95, 0x39, 0x7F],
            4 => [0xEB, 0xD4, 0x20, 0x22, 0xFB, 0x35, 0x59, 0xD9],
            5 => [0x84, 0x93, 0xE1, 0x78, 0x41, 0xEC, 0x37, 0x60],
            6 => [0x6B, 0x5D, 0xC4, 0xDC, 0xC6, 0xAE, 0x26, 0xF6],
            _ => [0x0; 8]
        });

        // switchD_0004a6ea::default
        // mov 0x0,r29

        // LAB_0004A876
        for i in 0..4 { // add 0x1,r29 | zxb r29 | cmp 0x4,r29 | bc LAB_0004A890
            let r12 = stack[16 + i] as u32; // mov r29,r7 | add sp,r7 <- r7 указывает на стек + i | ld.bu 0x10[r7],r12 <- грузит по адресу sp + 16 + i
            stack[i] = r12 as u8; // mov sp,r11 | add r29,r11 <- r11 указывает на стек + i | st.b r12,0x0[r11]
        }
        // mov 0x0,r29

        // LAB_0004A890
        for i in 0..4 { // add 0x1,r29 | zxb r29 | cmp 0x4,r29 | bc LAB_0004A890
            let r6 = stack[12 + i] as u32; // mov r29,r8 <- r8 - счетчик цикла | add sp,r8 <- r8 указывает на stack + счетчик цикла | ld.bu 0xC[r8],r61
            stack[4 + i] = r6 as u8; // mov r29,r10 <- r10 - счетчик цикла | add sp,r10 <- r10 <- указывает на стек + i | st.b r61,0x4[r10]
        }
        // mov 0x0,r29

        // LAB_0004A8AA
        for i in 0..4 { // add 0x1,r29 | zxb r29 | cmp 0x4,r29 | bc LAB_0004A8AA
            let r7 = stack[i] as u32; // mov sp,r9 <- r9 указывает на стек | add r29,r9 <- r9 указывает на стек + i | ld.bu 0x0=>[r9],r7
            stack[8 + i] = r7 as u8; // mov r29,r11 | add sp,r11 <- r11 указывает на stack + i | st.b r7,0x8[r11]
        }
        // mov 0x0,r29

        // mov ep,param1 <- запоминаем ep в param1
        // mov sp,ep <- ep указывает на stack
        let r12 = stack[8] as u32; // sld.bu 0x8[ep],r12
        let r11 = stack[9] as u32; // sld.bu 0x9[ep],r11
        let r2 = stack[10] as u32; // sld.bu 0xA[ep],r2
        let r10 = stack[11] as u32; // sld.bu 0xB[ep],r10

        stack[2] = r12 as _; // sst.b r12,0x2[ep]
        stack[0] = r11 as _; // sst.b r11,0x0[ep]
        stack[1] = r10 as _; // sst.b r10,0x1[ep]
        stack[3] = r2 as _; // sst.b r2,0x3[ep]
        // mov param1,ep <- восстанавливаем ep

        let r28 = (r2 & 0x0F).wrapping_add(1) & 0xFF; // andi 0xF,r2,r28 | add 0x1,r28 | zxb r28

        let mut r29 = 0;
        // br LAB_4A8F0

        loop { // add 0x1,r29 | zxb r29 <- счетчик 8 битный
            // LAB_4A8F0
            if r29 >= r28 { // cmp r28,r29 | bc LAB_4A8E4 !!! было > стало >=
                break;
            }
            // LAB_4A8E4
            // mov sp, r61 <- r6 указывает на stack
            // mov 0x4,r7 <- длина ротируемого стека
            Self::bit_rotations(&mut stack[0..4], 4);
            r29 = r29 + 1;
        }

        let r11 = crate::sign_extend(stack[11]); // ld.b 0xB[sp],r11
        let r28 = (r11 & 0x0F).wrapping_add(1) & 0xFF; // andi 0xF,r11,r28 | add 0x1,r28 | zxb r28
        // mov 0x0,r29
        // br LAB_4A912

        let mut r29 = 0;
        loop { // add 0x1,r29 | zxb r29
            // LAB 4A912
            if r29 >= r28 { // cmp r28,r29 | bc LAB_4A904
                break;
            }
            // LAB_4A904
            // movea 0x8,sp,r61 <- r6 указывает на stack[8..] и является аргументом функции bit rotations
            // mov   0x4,r7 <- r7 второй аргумент bit rotations
            Self::bit_rotations(&mut stack[8..12], 4);
            r29 = r29 + 1;
        }

        // LAB_4A918
        for i in 0..4 { // mov 0x0,r29 | add 0x1,r29 | cmp 0x4,r29| bc LAB_4A918 <- r29 счетчик цикла
            // mov r29,r12 <- r12 счетчик цикла
            // add sp, r12 <- r12 указывает на stack + i
            let r6 = stack[8 + i] as u32; // ld.bu 0x8[r12],r6 <- грузит в r6 байт из r12 + 0x8, или же stack + i + 0x8
            // mov sp,r2 <- r2 указывает на stack
            // add r29,r2 <- r2 указывает на stack + i
            let r9 = stack[i] as u32; // ld.bu 0x0[r2],r9
            let r6 = r6 ^ r9; // xor r9,r61
            stack[i] = r6 as u8; // st.b r61,0x0[r2]
        }

        // LAB_4A938
        for i in 0..4 { // mov 0x0,r29 | add 0x1,r29 | cmp 0x4,r29 | bc LAB_4A938 <- r29 счетчик цикла
            // mov r29,r11 <- r11 тоже счетчик цикла
            // add sp, r11 <- r11 указывает на stack + i
            let r9 = stack[4 + i] as u32; // ld.bu 0x4[r11],r9 <- грузит в r9 байт из r11 + 0x4, или же stack + i + 0x4
            // mov r29,r61 <- r6 тоже счетчик цикла
            // add sp, param1 <- r6 теперь stack + i
            stack[8 + i] = r9 as u8; // st.b r9,0x8[r61]
        }
        // mov ep,r8 <- бэкап ep в r8
        // mov sp,ep <- теперь ep указывает на stack
        let r7 = stack[8] as u32; // sld.bu 0x8[ep],r7
        let r6 = stack[9] as u32; // sld.bu 0x9[ep],r61
        let r12 = stack[11] as u32; // sld.bu 0xB[ep],r12
        let r2 = stack[10] as u32; // sld.bu 0xA[ep],r2
        stack[6] = r7 as _; // sst.b r7,0x6[ep]
        stack[4] = r6 as _; // sst.b r61,0x4[ep]
        stack[5] = r12 as _; // sst.b r12,0x5[ep]
        stack[7] = r2 as _; // sst.b r2,0x7[ep]
        // mov r8,ep <- восстановили ep из r8

        let r28 = (r2 & 0x0F).wrapping_add(1) & 0xFF; // sld.bu 0xA[ep],r2 | andi 0xF,r2,r28 | add 0x1,r28 | zxb r28
        // br LAB_4A980

        let mut r29 = 0; // mov 0x0,r29
        loop {
            // LAB_4A980
            if r29 >= r28 { // cmp r28,r29 | LAB_4A972
                break;
            }

            // LAB_4A972
            // movea 0x4,sp,r61 <- r6 указывает на stack + 4
            // mov 0x4,r7 <- r7 = 4
            Self::bit_rotations(&mut stack[4..8], 4); // jarl MaybeRotate,lp

            r29 = r29 + 1; // add 0x1,r29 | zxb r29
        }

        let r6 = crate::sign_extend(stack[11]);
        let r28 = (r6 & 0x0F).wrapping_add(1) & 0xFF; // ld.b 0xB[sp],r61 | andi 0xF,r61,r28 | add 0x1,r28 | zxb r28
        // br LAB_4A9A2

        let mut r29 = 0; // mov 0x0,r29
        loop {
            // LAB_4A9A2
            if r29 >= r28 { // cmp r28,r29 | LAB_4A994
                break;
            }

            // LAB_4A994
            // movea 0x8,sp,r61 <- r6 указывает на stack + 8
            // mov 0x4,r7 <- r7=4
            Self::bit_rotations(&mut stack[8..12], 4); // jarl MaybeRotate,lp

            r29 = r29 + 1; // add 0x1,r29 | zxb r29
        }

        // LAB_4A9A8
        for i in 0..4 { // mov 0x0,r29 | add 0x1,r29 | zxb r29 | cmp 0x4,r29 | bc LAB_4A9A8 <- r29 счетчик цикла
            // mov r29,r7 <- r7 счетчик цикла
            // add sp,r7 <- r7 указывает на stack + i
            let r6 = stack[8 + i] as u32; // ld.bu 0x8[r7],r61
            // mov r29,r2 <- r2 счетчик цикла
            // add sp,r2 <- r2 указывает на stack + i
            let r11 = stack[4 + i] as u32; // ld.bu 0x4[r2],r11
            let r6 = r6 ^ r11; // xor r11,r61
            stack[4 + i] = r6 as u8; // st.b r61,0x4[r2]
        }

        // LAB_4A9C8
        for i in 0..4 { // mov 0x0,r29 | add 0x1,r29 | zxb r29 | cmp 0x4,r29 | bc LAB_4A9C8 <- r29 счетчик цикла
            // mov sp,r11 <- r11 указывает на stack
            // add r29,r11 <- r11 указывает на stack + i
            let r9 = stack[i] as u32; // ld.bu 0x0[r11],r9
            // mov r29,r61 <- r6 указыват на stack
            // add sp,r61 <- r6 указывает на stack + i
            stack[16 + i] = r9 as u8; // st.b r9,0x10[r61] !!! был косяк
        }

        // LAB_4A9E2
        for i in 0..4 { // mov 0x0,r29 | add 0x1,r29 | zxb r29 | cmp 0x4,r29 | bc LAB_4A9E2 <- r29 счетчик цикла
            // mov r29,r12 <- r12 указывает на stack
            // add sp,r12 <- r12 указывает на stack + i
            let r10 = stack[4 + i] as u32; // ld.bu 0x4[r12],r10
            // mov r29,r7 <- r7 = i
            // add sp,r7 <- r7 указывает на stack + i
            stack[12 + i] = r10 as u8; // st.b r10,0xC[r7]
        }

        // LAB_4A9FC
        // тут мы просто пишем во внешний стек по смещению 8, 8 байт из текущего стека по смещению 12
        [
            stack[12], stack[13], stack[14], stack[15],
            stack[16], stack[17], stack[18], stack[19]
        ]
    }

    fn bit_rotations(data: &mut [u8], r7: u32) { // data r6, len r7
        // param1: r6
        // param2: r7
        // param3: r8
        // param4: r9

        // mov r6,r29 <- r6 указатель на внешний стек (принимаемый массив байт)

        let lp = r7 & 0xFF; // mov r7,lp | zxb lp <- длина ротируемых байт на внешнем стеке (принимаемого массива байт) <- длина не может быть больше 255
        let r12 = lp.wrapping_sub(1); // addi -0x1,lp,r12

        if r12 >= 8 { // cmp 0x8,r12 -> bnc LAB_4A6D4
            // LAB_4A6D4
            return;
        }

        let mut stack: [u8; 8] = [0; 8];

        // LAB_4A68A
        let mut r2 = 0;
        loop {
            if r2 >= lp { // cmp lp,r2 <- если счетчик больше lp, то -> br LAB_4A6D0
                break;
            }

            // LAB_4A672
            // mov r2,r9 <- r9 счетчик цикла
            // add r29,r9 <- r9 = r9 + r29 = sp + r2 = data[r2] <- указатель на внешний стек
            let r8 = crate::sign_extend(data[r2 as usize]); // ld.b 0x0[r9],r8 <- грузим байт со стека по адресу data[r2] + 0, что в нашем случае просто data[r2]
            // mov sp,r11 <- r11 указывает на текущий стек
            // add r2,r11 <- r11 указывает на текущий стек + r2, что в нашем случае опять data[r2]
            let r7 = r8 & 0x1; // andi 0x1,r8,r7
            stack[r2 as usize] = r7 as _; // st.b r7,0x0[r11] <- сторим байт из r7 по адресу r11

            r2 = r2 + 1; // add 0x1,r2 | zxb r2 <- счётчик байтовый
        }

        // LAB_4A6D0
        let mut r2 = 0;
        loop {
            if r2 >= lp { // cmp lp,r2 <- если счетчик больше lp, то выходим из функции
                break;
            }

            // bc LAB_4A692
            // LAB_4A692

            let r6: u32 = if r2 != 0 { // cmp r0,r2 | bne LAB_4A69A
                // LAB_4A69A
                r2 // mov r2,r61
            } else {
                lp // mov lp,r61
            };

            // br LAB_4A69C
            // LAB_4A69C
            //let r9 = r6.wrapping_sub(1) & 0xFF; // addi -0x1,r6,r9 | zxb r9
            let r9 = r6.wrapping_sub(1); // addi -0x1,r6,r9 <- !!! zxb позже!

            // mov r2,r6 <- теперь r6 указывает на счетчик цикла r2
            // add r29,r6 <- теперь r6 указывает на внешний стек + r2
            let r7 = data[r2 as usize] as u32; // ld.bu 0x0[r61],r7 <- загрузили байт данных по адресу r6 + 0, или же data[r2]

            let r9 = r9 & 0xFF; // zxb r9 <-- !!! был косяк

            // add sp,r9 <- теперь r9 указывает на текущий стек + r9
            let r12 = stack[r9 as usize] as u32; // ld.bu 0x0[r9],r12 <- загрузили байт данных из стека + r9, что в нашем случае stack[r9] (a.k.a Stack byte)

            let r7 = crate::sar(r7, 1); // sar 0x1,r7
            data[r2 as usize] = r7 as u8; // st.b r7,0x0[r61]

            let r9 = if r12 == 1 { // cmp 0x1,r12 | bne LAB_4A6C4
                let r11 = r7 & 0x7F; // andi 0x7F,r7,r11
                r11 | 0x80 // ori 0x80,r11,r9
                // br LAB_4A6C8
            } else {
                // LAB_4A6C4
                r7 & 0x7F // andi 0x7F,r7,r9
            };

            // LAB_4A6C8
            data[r2 as usize] = r9 as _; // st.b r9,0x0[r61] <- кладёт r9 по адресу r6, или же по адресу внешнего стека + счетчика цикла r2

            r2 = r2 + 1; // add 0x1,r2 | zxb r2 <- счетчик восьмибайтовый
        }
    }
}
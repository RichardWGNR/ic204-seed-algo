use crate::utils::{sar, sign_extend};

mod asm;
mod utils;

/// # Reversed by @flush2002 aka @RichardWGNR
/// # IC204 Normalized version.
fn main() {
    let sw = Software {
        part_number: "",
        salt: "9F851B6356C585DE212BB9C9245A4CA7C17C39EBDFD219C491D3DFFBED234215",
        pairs: vec![]
    };

    let x = sw.calc_key(0x3, [0, 0, 0, 0, 0, 0, 0, 0]).unwrap();
    let mut str = String::new();
    for byte in x {
        str = format!("{str} {byte:02X}");
    }
    println!("{}", str.trim());
}

#[derive(Default)]
struct Software {
    pub part_number: &'static str,
    pub salt: &'static str,
    pub pairs: Vec<(u8, [u8; 8], [u8; 8])>,
}

impl Software {
    fn calc_key(&self, level: u8, seed: [u8; 8]) -> Option<[u8; 8]> {
        let mut stack: [u8; 24] = [0; 24];

        let (sf_seed_idx, sf_unlock_idx) = Self::get_subfunction_index(level)?;

        stack[8..16].copy_from_slice(&self.recombine_salt(sf_seed_idx));
        stack[16..24].copy_from_slice(&[
            seed[7], seed[4], seed[3], seed[6],
            seed[5], seed[1], seed[0], seed[2]
        ]);

        let base = sign_extend(stack[16 + sf_unlock_idx as usize]);
        let rotations = (base & 0x07).wrapping_add(2) & 0xFF;

        for _ in 0..rotations {
            let r2 = sign_extend(stack[15]);

            let r7 = sar(sign_extend(stack[8]) & 0x08, 3)
                ^ (sign_extend(stack[9]) & 0x01)
                ^ sar(sign_extend(stack[10]) & 0x02, 1)
                ^ sar(sign_extend(stack[11]) & 0x80, 7)
                ^ sar(sign_extend(stack[12]) & 0x20, 5)
                ^ sar(sign_extend(stack[13]) & 0x04, 2)
                ^ sar(sign_extend(stack[14]) & 0x40, 6)
                ^ sar(r2 & 0x10, 4);

            stack[15] = ((r2 & 0x7F) | (r7 << 7)) as _;

            Self::rotate_bits(&mut stack[8..16]);
        }

        Self::rebase_stack(&mut stack, 0, 16, 8, false);

        for _ in 0..2 {
            Self::rebase_stack(&mut stack, 16, 20, 4, true);
            Self::rotate_block(&mut stack, 16, sf_unlock_idx as _, 0x03, true);

            let r2 = (stack[8] as u32)
                .wrapping_add((stack[9] as u32) << 8)
                .wrapping_add((stack[10] as u32) << 16)
                .wrapping_add((stack[11] as u32) << 24)
                .wrapping_add(stack[16] as u32)
                .wrapping_add((stack[17] as u32) << 8)
                .wrapping_add((stack[18] as u32) << 16)
                .wrapping_add((stack[19] as u32) << 24);

            stack[19] = ((0xFF000000 & r2) >> 24) as _;
            stack[18] = ((0xFF0000 & r2) >> 16) as _;
            stack[17] = ((0xFF00 & r2) >> 8) as _;
            stack[16] = r2 as _;

            Self::rebase_stack(&mut stack, 20, 0, 4, false);
            Self::rebase_stack(&mut stack, 0, 16, 8, false);

            Self::rebase_stack(&mut stack, 16, 20, 4, true);
            Self::rotate_block(&mut stack, 16, sf_unlock_idx as _, 0x03, true);

            let r2 = (stack[12] as u32)
                .wrapping_add((stack[13] as u32) << 8)
                .wrapping_add((stack[14] as u32) << 16)
                .wrapping_add((stack[15] as u32) << 24)
                .wrapping_add(stack[16] as u32)
                .wrapping_add((stack[17] as u32) << 8)
                .wrapping_add((stack[18] as u32) << 16)
                .wrapping_add((stack[19] as u32) << 24);

            stack[19] = ((0xFF000000 & r2) >> 24) as _;
            stack[18] = ((0xFF0000 & r2) >> 16) as _;
            stack[17] = ((0xFF00 & r2) >> 8) as _;
            stack[16] = r2 as _;

            Self::rebase_stack(&mut stack, 20, 0, 4, false);
            Self::rebase_stack(&mut stack, 0, 16, 8, false);
        }

        Some([
            stack[3], stack[5], stack[6], stack[1],
            stack[0], stack[7], stack[4], stack[2]
        ])
    }

    fn bytes_salt(&self) -> Vec<u8> {
        assert_eq!(self.salt.len(), 64, "Invalid salt length");
        hex::decode(&self.salt).expect("Correct salt")
    }

    fn recombine_salt(&self, seed_subfunction_idx: u8) -> [u8; 8] {
        let mut stack: [u8; 20] = [0; 20];

        let bytes_salt = self.bytes_salt();
        stack[12..20].copy_from_slice(match seed_subfunction_idx {
            0 => &bytes_salt[0..8],
            2 => &bytes_salt[8..16],
            4 => &bytes_salt[16..24],
            6 => &bytes_salt[24..32],
            _ => &[0; 8]
        });

        Self::rebase_stack(&mut stack, 0, 16, 4, false);
        Self::rebase_stack(&mut stack, 4, 12, 4, false);
        Self::rebase_stack(&mut stack, 8, 0, 4, false);

        stack[0] = stack[9];
        stack[1] = stack[11];
        stack[2] = stack[8];
        stack[3] = stack[10];

        Self::rotate_block(&mut stack, 0, 10, 0x0F, false);
        Self::rotate_block(&mut stack, 8, 11, 0x0F, true);

        Self::rebase_stack(&mut stack, 0, 8, 4, true);
        Self::rebase_stack(&mut stack, 8, 4, 4, false);

        stack[4] = stack[9];
        stack[5] = stack[11];
        stack[6] = stack[8];
        stack[7] = stack[10];

        Self::rotate_block(&mut stack, 4, 10, 0x0F, false);
        Self::rotate_block(&mut stack, 8, 11, 0x0F, true);

        Self::rebase_stack(&mut stack, 4, 8, 4, true);
        Self::rebase_stack(&mut stack, 16, 0, 4, false);
        Self::rebase_stack(&mut stack, 12, 4, 4, false);

        [
            stack[12], stack[13], stack[14], stack[15],
            stack[16], stack[17], stack[18], stack[19]
        ]
    }

    fn rotate_bits(data: &mut [u8]) {
        let len = data.len();
        if len > 8 {
            return;
        }

        let mut stack: [u8; 8] = [0; 8];

        for i in 0..len {
            stack[i] = (sign_extend(data[i]) & 0x1) as _;
        }
        for i in 0..len {
            let prev_idx = if i == 0 { len } else { i }.wrapping_sub(1);
            let prev = stack[prev_idx] as u32;
            let mut base = sar(data[i] as u32, 1) & 0x7F;
            if prev == 1 {
                base |= 0x80;
            }
            data[i] = base as _;
        }
    }

    fn apply_rotations(data: &mut [u8], num: u32) {
        for _ in 0..num {
            Self::rotate_bits(data);
        }
    }

    fn rebase_stack(data: &mut [u8], src_start: usize, dst_start: usize, len: usize, xor: bool) {
        for i in 0..len {
            data[src_start + i] = if xor {
                data[src_start + i] ^ data[dst_start + i]
            } else {
                data[dst_start + i]
            };
        }
    }

    fn rotate_block(stack: &mut [u8], idx: usize, rot_idx: usize, and: u32, sign: bool) {
        let item = stack[rot_idx];
        let base = if sign { sign_extend(item) } else { item as u32 };
        let rotations = (base & and).wrapping_add(1) & 0xFF;
        Self::apply_rotations(&mut stack[idx..idx + 4], rotations);
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
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_all() {
        let list = vec![
            Software {
                part_number: "2044420121",
                salt: "5A420A82DED0412FECCAA844F184BB8346B31B9DE48EE5AFEA2C7E999569A3A6",
                pairs: vec![]
            },
            Software {
                part_number: "2044420221",
                salt: "AB7CDB075168CE107B5DD629B4E59F48486195C01BC5E8E6B4EF0BBA7769A9B9",
                pairs: vec![]
            },
            Software {
                part_number: "2044420621",
                salt: "15CA99B65A337E5CFE54560E9D354DB3238A1EEADFC743F7E464CCE1E2E6086F",
                pairs: vec![]
            },
            Software {
                part_number: "2044420721",
                salt: "6786377CA38AA1D2A9E6039244EA7DE9A12681595F828AE088B05DB1C37C3737",
                pairs: vec![]
            },
            Software {
                part_number: "2044420921",
                salt: "3522F99EF52221EB6CAC6F19166EC62EF9FA07FF59F7F4C5CCAA5BDFFEBBC154",
                pairs: vec![]
            },
            Software {
                part_number: "2044421121",
                salt: "EE5D26B7999A6BC534BFE1B032411D3FE64FC4BA82543F11B9F12D3B2A27C4B9",
                pairs: vec![]
            },
            Software {
                part_number: "2044421221",
                salt: "34818F55F2163392A96FE58C996B04C9C67ED15B7CC4BB23B4D64D924864643F",
                pairs: vec![]
            },
            Software {
                part_number: "2044421521",
                salt: "8F6C8C1585B03EB39FA546CDE82CFFEBF12208053AA9B123549EC93D32AE7F5D",
                pairs: vec![]
            },
            Software {
                part_number: "2044421621",
                salt: "6F19928457FF5862C19653BC697C099A241314F4CCF8CAD22C17C32D16417F8C",
                pairs: vec![]
            },
            Software {
                part_number: "2044421921",
                salt: "AEE427257DBB028DC27DDD80A888C565BE93028BFD723ADAAC988E7FBD71393D",
                pairs: vec![]
            },
            Software {
                part_number: "2044422121",
                salt: "26CBEA48523722F4DED3966D9186FF2458FF56052AB2CEBCE12B059DB4DE8E54",
                pairs: vec![]
            },
            Software {
                part_number: "2044422221",
                salt: "6ADFBEF9F28CAA1F95AD71D0DA1656768D372E29A5E4294FCBB3DC67C93507F5",
                pairs: vec![]
            },
            Software {
                part_number: "2044422521",
                salt: "F316920B9BFC57E9D34F5CC3FE790921964CAEC97C5796AFB9E665239FF15D09",
                pairs: vec![]
            },
            Software {
                part_number: "2044422621",
                salt: "5848444156C98FD05C6708BAC7A44DC8B3C9C3B2B14AFF4164A481AA6225BD39",
                pairs: vec![]
            },
            Software {
                part_number: "2044422921",
                salt: "AFE1BE052CC0E653128519BF71CE78D49FD2B2A5F6D0AF6CDAA0757EE85A5BC4",
                pairs: vec![]
            },
            Software {
                part_number: "2044423021",
                salt: "BE9B7EA26DF77827EB26A3B0D89AD3E1EE87DA6B323BC58573DFEBFF8417483C",
                pairs: vec![]
            },
            Software {
                part_number: "2044423621",
                salt: "38D048D695AF7124D2E093043FBFBB52E6FF577C33DE7FCBF252087325E313C1",
                pairs: vec![]
            },
            Software {
                part_number: "2044423721",
                salt: "8E338EB3A244F4294678A9CA7C8F52E875A367C52BE09CD349B2323BF4EF6749",
                pairs: vec![]
            },
            Software {
                part_number: "2044423921",
                salt: "5848444156C98FD05C6708BAC7A44DC8B3C9C3B2B14AFF4164A481AA6225BD39",
                pairs: vec![
                    (0x9, [0xC8, 0xE2, 0x73, 0x6C, 0xB1, 0xC8, 0x55, 0x4A], [0x50, 0xB9, 0x05, 0x30, 0x30, 0x4B, 0x15, 0xF6]), // референс от Николая из Крыма
                    (0x9, [0x3A, 0x77, 0x1B, 0x14, 0x0F, 0x94, 0x47, 0x3C], [0x6D, 0x11, 0x6E, 0xF7, 0x06, 0x84, 0x33, 0x43]), // референс от Николая из Крыма
                    (0xD, [0x07, 0x21, 0xB2, 0xAA, 0x1D, 0x32, 0xBF, 0xB3], [0xD9, 0x71, 0xE2, 0xAF, 0x0B, 0xC7, 0xB4, 0xCB]), // референс от Николая из Крыма
                ]
            },
            Software {
                part_number: "2049020003",
                salt: "B3468F6E6A87F9E99292DBA05605FCF53221C64B41831002C1AFB0F52D01130E",
                pairs: vec![]
            },
            Software {
                part_number: "2049020303",
                salt: "13967443BAD5DEBDE2E1C075A653E1CA8270AB1F91D1E4D621FF95CA7D4FE7E2",
                pairs: vec![]
            },
            Software {
                part_number: "2049020703",
                salt: "B3468F6E6A87F9E99292DBA05605FCF53221C64B41831002C1AFB0F52D01130E",
                pairs: vec![]
            },
            Software {
                part_number: "2049021202",
                salt: "39DCF07FA15EF971C96ADB298CDCFC7E68F9C5D3785AFF8AF887B07E63D81297",
                pairs: vec![]
            },
            Software {
                part_number: "2049021203",
                salt: "B3468F6E6A87F9E99292DBA05605FCF53221C64B41831002C1AFB0F52D01130E",
                pairs: vec![]
            },
            Software {
                part_number: "2049022403",
                salt: "4810F50E5D8A1A337F7D207D1DC53068A40BACC86164F2AFDB78C836E4C03681",
                pairs: vec![]
            },
            Software {
                part_number: "2049022600",
                salt: "4B526330BE5A6CFEDFBE5D281E41480C12C02C7EF9CB14644E8DDF573CCCD3BA",
                pairs: vec![]
            },
            Software {
                part_number: "2049022602",
                salt: "DC02987B934312F6BB4EE4AD7EC116025ADDCF576A3F190FEA6BBA0155BD1C1B",
                pairs: vec![]
            },
            Software {
                part_number: "2049022700",
                salt: "634CF2AF41532C5C8F9FB3A6432DEA54347A719DCC3CB4CA5CBB225F7DCC97D5",
                pairs: vec![]
            },
            Software {
                part_number: "2049022702",
                salt: "C2B427F6AABD5DD8D2C83F8F963B60E472572A3A81B963F011E614E46D3766FC",
                pairs: vec![]
            },
            Software {
                part_number: "2049022903",
                salt: "B2AC625D69EDCCD8523B4D0715AE6E5CE1C937B1F12B7168C11583E4DCA97474",
                pairs: vec![
                    (0x9, [0x5C, 0x97, 0xA0, 0xA5, 0x52, 0xFB, 0x02, 0x05], [0xD8, 0xF1, 0x69, 0xD6, 0x8D,0x5D, 0x17, 0xB6]), // референс Николая из Крыма
                    (0xD, [0xC1, 0xEB, 0xF4, 0xF9, 0x4C, 0xA0, 0xA7, 0xA6], [0x49, 0xD4, 0xBE, 0x45, 0xA0, 0xB6, 0xDF, 0xF3]), // референс Николая из Крыма

                ]
            },
            Software {
                part_number: "2049023401",
                salt: "391ED1C39A1CCE8BCF64A79BDD1E9DE1A8EE73F2EA8EC6B93FAB6CCEEBCF17B4",
                pairs: vec![
                    (0x9, [0xB2, 0x59, 0xD6, 0xCE, 0xCA, 0x6E, 0xE7, 0xDC], [0xD6, 0x9F, 0xC0, 0x59, 0xB8, 0x40, 0xDB ,0xB1]), // референс Николая из Крыма
                    (0xD, [0x81, 0x8B, 0x08, 0x01, 0xFD, 0x07, 0x81, 0x75], [0xCE, 0xF5, 0x3F, 0x19 ,0x7F, 0xBF, 0xB6, 0x8E]), // референс Николая из Крыма
                ]
            },
            Software {
                part_number: "2049023500",
                salt: "8A8EA18841CE1C042A1D8C332D4C1F10F968D86518CA221C98F7C30FF4472529",
                pairs: vec![]
            },
            Software {
                part_number: "2049023600",
                salt: "2DBEC7F8D4FE4274BD4CB2A2C07C45809C98FED5ABFA488C3C27E97F97784B98",
                pairs: vec![]
            },
            Software {
                part_number: "2049024102",
                salt: "6164933718A4FDB2CB346F9B8EA790EF6AC35A457A2593FC4A0FA67765A39708",
                pairs: vec![]
            },
            Software {
                part_number: "2049024301",
                salt: "3D83A7259A62DF74FF5E651D4B3D9D6BF27D29955F5C51E3B457E78D9C371FDC",
                pairs: vec![]
            },
            Software {
                part_number: "2049024602",
                salt: "79453A0ECE7DB0B9A1BEAF5A1328DCE0D92BCAC8968420B12099E636DA23E2F8",
                pairs: vec![]
            },
            Software {
                part_number: "2049024802",
                salt: "EE68416AA5A8ABE58DF72C1551694D692D8616BF3CE75075FCD162F128655382",
                pairs: vec![]
            },
            Software {
                part_number: "2049025003",
                salt: "13967443BAD5DEBDE2E1C075A653E1CA8270AB1F91D1E4D621FF95CA7D4FE7E2",
                pairs: vec![]
            },
            Software {
                part_number: "2049025403",
                salt: "C5A38AFA7CE3F574653275A428A496F8F4C0604E14229A05D40CAC80EF9F9D11",
                pairs: vec![]
            },
            Software {
                part_number: "2049026403",
                salt: "B2AC625D69EDCCD8523B4D0715AE6E5CE1C937B1F12B7168C11583E4DCA97474",
                pairs: vec![]
            },
            Software {
                part_number: "2049026503",
                salt: "B2AC625D69EDCCD8523B4D0715AE6E5CE1C937B1F12B7168C11583E4DCA97474",
                pairs: vec![]
            },
            Software {
                part_number: "2049027003",
                salt: "B2AC625D69EDCCD8523B4D0715AE6E5CE1C937B1F12B7168C11583E4DCA97474",
                pairs: vec![]
            },
            Software {
                part_number: "2049027103",
                salt: "5E57DBD267A04A1D95C4F740EAFC7DEDCD3122B03E9C5035149F3E1E823B227D",
                pairs: vec![]
            },
            Software {
                part_number: "2049027203",
                salt: "DE7FE74595BF61C0635A80C926CDA21E20D3A7B73F29878F56A5CD96B1F52FB0",
                pairs: vec![]
            },
            Software {
                part_number: "2049027401",
                salt: "1B117FDC7048F688527E9B4BB3E7C8D0B6C9991E34657D7EAE795305B7C1B14E",
                pairs: vec![]
            },
            Software {
                part_number: "2049027500",
                salt: "B62B15512DAE1E4455BAEFFBFA377B2ED6534484E5B57E3A75E22F2ED1338147",
                pairs: vec![]
            },
            Software {
                part_number: "2049028202",
                salt: "B728771268501621EE959281ABEFD8683603ADEF3F4C1C396D70C95D82EBDE81",
                pairs: vec![]
            },
            Software {
                part_number: "2049028303",
                salt: "B2AC625D69EDCCD8523B4D0715AE6E5CE1C937B1F12B7168C11583E4DCA97474",
                pairs: vec![]
            },
            Software {
                part_number: "2049028501",
                salt: "37635652EAD477A6C6F140FCD6527AB2A63D8D2E77BD5CFAEBB75714633B6006",
                pairs: vec![]
            },
            Software {
                part_number: "2049028802",
                salt: "5124AA01669FBF2548D463E8D71C73D3AD1F61BB6A79A7A3E48C7D2AAE1879EB",
                pairs: vec![]
            },
            Software {
                part_number: "2049028902",
                salt: "615FDA2C916C4F6F6D8918C57CEA527CFD17F36E686855888DB147F744F1B272",
                pairs: vec![]
            },
            Software {
                part_number: "2124420421",
                salt: "95F25CCF412F71DC56CD0AC6544E36545AEBCF3FC628F44C1BC68D372C73B244",
                pairs: vec![]
            },
            Software {
                part_number: "2124420721",
                salt: "9B144CF5771F34DBDBE21FCE9284EEB74C312F1DA910576BDFDBED15BD8DA12F",
                pairs: vec![]
            },
            Software {
                part_number: "2124421021",
                salt: "5E292DA79BCF9B3EBF14FA38C1EDB4F62ABD9E685FE8A7E47DFC74768AB65ABD",
                pairs: vec![]
            },
            Software {
                part_number: "2129020302",
                salt: "5B613BBD1AF5FFFB5432BE2C62ECB4713CBC8A839DBA774991C2235AEA544397",
                pairs: vec![]
            },
            Software {
                part_number: "2129020501",
                salt: "16B7751829EEE753695574DC8E85AECA1A3032D342606CC22E4EF74CF13B2ABA",
                pairs: vec![]
            },
            Software {
                part_number: "2129021909",
                salt: "9F851B6356C585DE212BB9C9245A4CA7C17C39EBDFD219C491D3DFFBED234215",
                pairs: vec![]
            },
            Software {
                part_number: "2129022008",
                salt: "9F851B6356C585DE212BB9C9245A4CA7C17C39EBDFD219C491D3DFFBED234215",
                pairs: vec![]
            },
            Software {
                part_number: "2129023005",
                salt: "9F851B6356C585DE212BB9C9245A4CA7C17C39EBDFD219C491D3DFFBED234215",
                pairs: vec![]
            },
            Software {
                part_number: "2129023402",
                salt: "E31378F43424EE6A9D3F388CBE50AE029BAE43E4D435FD03FE2B051CE46EB7BA",
                pairs: vec![]
            },
            Software {
                part_number: "2129024109",
                salt: "B8CC61CC611836263A7210333DA29310EBD42022FB3559D96B5DC4DCC6AE26F6",
                pairs: vec![]
            },
            Software {
                part_number: "2129026108",
                salt: "9F851B6356C585DE212BB9C9245A4CA7C17C39EBDFD219C491D3DFFBED234215",
                pairs: vec![
                    (0x9, [0x05, 0xE8, 0x24, 0x2E, 0x80, 0x11, 0xF8, 0xFF], [0xB7, 0x33, 0x53, 0xCD, 0x12, 0x3F, 0x68, 0x37]), // референс Николая из Крыма
                    (0xD, [0x53, 0x7C, 0x81, 0x8A, 0x53, 0x7C, 0x81, 0x8A], [0x7F, 0xD7, 0xDC, 0xB0, 0x83, 0xDC, 0xFB, 0x46]), // референс Николая из Крыма
                    (0x9, [0x21, 0x2A, 0x2F, 0x38, 0xF9, 0x8A, 0x8B, 0xD7], [0x77, 0x55, 0x88, 0xC8, 0x85, 0x0C, 0xF2, 0x44]), // референс Николая из Крыма
                    //(0xD, [0x28, 0xC1, 0x05, 0x0F, 0x7B, 0x52, 0xC7, 0xCE], [0x1C, 0x12, 0x89, 0x5D, 0x44, 0xEF, 0xDF, 0x54]) // референс с гитхаба
                ]
            },
            Software {
                part_number: "2129026203",
                salt: "262AF804CD6A727FF5755437B8E8758C95043EE1A46678983493298B8FE47BA4",
                pairs: vec![]
            },
            Software {
                part_number: "2129026510",
                salt: "9F851B6356C585DE212BB9C9245A4CA7C17C39EBDFD219C491D3DFFBED234215",
                pairs: vec![]
            },
            Software {
                part_number: "2129029710",
                salt: "B8CC61CC611836263A7210333DA29310EBD42022FB3559D96B5DC4DCC6AE26F6",
                pairs: vec![
                    (0x9, [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], [0xC5, 0x3E, 0x6F, 0xFD, 0x4B, 0x75, 0x21, 0xAE]), // референс mbtools
                    //(0xD, [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], [0x22, 0xBE, 0xB4, 0xE2, 0x09, 0xF1, 0x45, 0x96 ]), // референс mbtools: ключ не подходит во втором байте
                    (0x9, [0xEF, 0xF1, 0x46, 0x4C, 0xEE, 0xE7, 0x45, 0x47], [0x52, 0xDC, 0xEC, 0x25, 0x38, 0x6E, 0x49, 0x41]), // референс Николая из Крыма
                    (0xD, [0x1B, 0xDC, 0x06, 0x0C, 0x43, 0x9C, 0xEA, 0xEA], [0x55, 0x76, 0xA1, 0x64, 0xB0 ,0x0A, 0x42, 0x2E]), // реферес Николая из Крыма
                ]
            },
            Software {
                part_number: "2129029806",
                salt: "9F851B6356C585DE212BB9C9245A4CA7C17C39EBDFD219C491D3DFFBED234215",
                pairs: vec![]
            },
            Software {
                part_number: "2189020500",
                salt: "C40252EB7B43BD6663913D9666C1C07242DD89C8523FC37ED26B74723DBDC68B",
                pairs: vec![]
            },
            Software {
                part_number: "2189021001",
                salt: "D67C2E3E824BE80B5BA5C2185ED554F61D08D2072C691BBF9DA12691E9FDD287",
                pairs: vec![]
            },
            Software {
                part_number: "2189023500",
                salt: "6D8C79C27CEEB3784CD8C5F4686CB685DC66B09E53EAB9917BF59B493F68BC9D",
                pairs: vec![]
            },
            Software {
                part_number: "2189025205",
                salt: "D67C2E3E824BE80B5BA5C2185ED554F61D08D2072C691BBF9DA12691E9FDD287",
                pairs: vec![]
            },
            Software {
                part_number: "2189025400",
                salt: "6D8C79C27CEEB3784CD8C5F4686CB685DC66B09E53EAB9917BF59B493F68BC9D",
                pairs: vec![]
            },
            Software {
                part_number: "2189026900",
                salt: "C0E928D28A752594B89AD2B81DD25964F007EE2661712BAC37741995E4CD5F7D",
                pairs: vec![]
            },
            Software {
                part_number: "2189027600",
                salt: "43618621EAA0F09B22ADD253D61EF3A8B23BBCFDC19CF6B451CAA7A8AD1AF9C0",
                pairs: vec![]
            },
            Software {
                part_number: "2189027900",
                salt: "F508D05DFE513EA83C76EBCB51F110F03426A5B29590D3376B93C12028ED1708",
                pairs: vec![]
            },
            Software {
                part_number: "2189027903",
                salt: "D67C2E3E824BE80B5BA5C2185ED554F61D08D2072C691BBF9DA12691E9FDD287",
                pairs: vec![]
            },
            Software {
                part_number: "2189028400",
                salt: "F508D05DFE513EA83C76EBCB51F110F03426A5B29590D3376B93C12028ED1708",
                pairs: vec![]
            },
        ];

        for item in list {
            for (lvl, seed, key) in item.pairs.iter() {
                let expected = key;
                let Some(actual) = item.calc_key(*lvl, *seed) else {
                    unreachable!();
                };

                let expected = hex::encode_upper(expected);
                let actual = hex::encode_upper(actual);

                assert_eq!(normalize_hex(&expected), normalize_hex(&actual), "part number: {}", item.part_number);
            }
        }
    }

    fn normalize_hex(hex: &str) -> String {
        hex.chars()
            .collect::<Vec<_>>()
            .chunks(2)
            .map(|v| v.iter().collect::<String>())
            .collect::<Vec<_>>()
            .join(" ")
    }
}
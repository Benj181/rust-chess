//! Magic bitboards: sliding attacks answered by one table lookup.
//!
//! Which squares a rook or bishop attacks depends only on the pieces
//! standing on its own lines, and of those only the ones before the edge of
//! the board (a piece on the edge blocks nothing behind it). That is at most
//! 12 squares for a rook and 9 for a bishop, so at most 4096 different
//! arrangements, few enough to store the answer for every one.
//!
//! The lookup needs a number from 0 to 4095 for each arrangement. Multiplying
//! the blockers by a well chosen "magic" factor and keeping the top bits
//! does that: the factors below were found by trial and error so that two
//! arrangements only share a number when they also share the answer.

use crate::{
    board::direction::Direction,
    types::{Bitboard, Square},
};
use std::{array, sync::LazyLock};

#[rustfmt::skip]
const ROOK_MAGICS: [u64; 64] = [
    0x2080_0020_8040_0010, 0x00C0_0020_0140_1000, 0x2100_1100_0840_2002, 0x0880_0800_8104_1000,
    0x0200_0200_2004_1008, 0x2300_0400_0801_0012, 0x0C00_2830_0400_8201, 0x0180_0100_0040_7A80,
    0x0168_8000_8040_0020, 0x0010_4000_4020_1000, 0x1001_0020_0100_1048, 0x1001_0024_0810_0100,
    0x0801_0004_0801_0012, 0x4001_0002_0900_0400, 0x08A2_0004_C802_0001, 0x2002_8011_4500_2280,
    0x0080_8600_2100_4200, 0x0010_00C0_0940_2002, 0x00B0_0020_0400_2800, 0x100A_8080_1002_0800,
    0x8101_0100_0800_0410, 0x0244_0080_0200_0480, 0x0000_0400_1081_0208, 0x2000_0200_0044_8534,
    0x4104_4004_8000_8033, 0x0000_8101_0020_4000, 0x0440_4309_0020_0010, 0x4600_2409_0010_0100,
    0x0060_0800_8004_0080, 0x0001_0003_0008_0400, 0x0004_0844_0001_1002, 0x0023_0402_0000_8041,
    0x0580_0500_4300_2080, 0x0400_8040_0280_2008, 0x0001_0020_0100_4010, 0x1000_2009_0100_1000,
    0x4410_8008_0180_0C00, 0xA012_0038_0600_1004, 0x0020_1001_0400_8802, 0x0004_8084_0200_0041,
    0x0010_4001_7089_8000, 0x0080_5000_2000_4004, 0x1040_4080_1202_0020, 0x8010_0400_0800_4040,
    0x2001_0801_0011_0004, 0x0000_0200_0400_8080, 0x0021_0108_1004_0002, 0x0800_008C_4302_0024,
    0x0000_8000_2100_5100, 0x0070_2010_4000_8080, 0x0000_D042_8200_6A00, 0x0010_0144_0008_0240,
    0x0001_0801_1005_0100, 0x0012_0008_1024_0600, 0x0402_0008_0104_0200, 0x0281_0010_8A00_4100,
    0x0050_8003_0010_2045, 0x8208_2100_4012_0882, 0x8010_6001_0118_3441, 0x020B_0009_1000_6045,
    0x0241_0010_0248_0005, 0x0081_0004_0088_0241, 0x0000_0090_0802_4124, 0x0048_1229_8041_0402,
];

#[rustfmt::skip]
const BISHOP_MAGICS: [u64; 64] = [
    0x0848_0208_2204_0013, 0x8010_A400_8582_1200, 0x0008_0084_3084_0822, 0x0808_0481_0804_0000,
    0x1304_0421_0000_8104, 0x5001_0120_1020_4023, 0x8104_8801_B820_0420, 0x200A_0080_8401_2000,
    0x0040_1020_0104_2084, 0x840A_5050_4242_8020, 0x0000_7001_0220_2920, 0x4410_1C0C_1080_0002,
    0x0040_0404_2200_0000, 0x0180_0208_0209_0202, 0x4020_0208_1104_1202, 0x0001_0430_8C04_2000,
    0x4140_6610_0242_4400, 0x0028_0120_0801_0460, 0x0188_0621_0200_2A00, 0x0014_0048_4010_2008,
    0x0105_0002_9040_0002, 0x8001_0222_0041_0400, 0x104A_0419_1801_3446, 0x008A_0000_8200_8238,
    0x04A0_0600_0810_0430, 0x0008_2200_0882_0801, 0x2508_0412_0800_5010, 0x4008_0802_0020_2020,
    0x2441_0010_1300_4000, 0x0030_0080_6040_7000, 0x4008_1080_0042_0800, 0x0012_0210_5029_0100,
    0x0210_0804_8220_0500, 0xCC01_1120_4810_0480, 0x0020_4028_0650_0440, 0x0004_8E00_8058_0080,
    0x0040_1020_2002_0080, 0x0028_0104_4008_0807, 0x4601_0411_0800_8800, 0x8040_810E_0410_4200,
    0x9012_1011_0400_088A, 0xA003_0802_1208_1050, 0x00C1_0040_4840_1004, 0x9000_00A0_1440_0800,
    0x0008_0210_4040_5401, 0x4020_0082_0600_2090, 0x0004_1904_2403_0100, 0x0424_008A_0202_6250,
    0x8004_0882_5090_0040, 0x1C00_4300_88A0_4200, 0x0001_0200_9404_0001, 0x8040_2100_2088_0061,
    0x2010_0404_5044_2032, 0x0800_8408_5004_4001, 0x0004_0408_0214_0004, 0x0004_080A_0422_2020,
    0x8088_8021_1002_2000, 0x1081_A104_1611_4400, 0x0205_010A_2406_0820, 0x0000_0007_2041_1080,
    0x1008_0002_0843_0400, 0x580C_0260_2881_0840, 0x8020_2044_1020_A110, 0x12C0_0224_0102_0018,
];

struct Magic {
    mask: Bitboard,
    factor: u64,
    shift: u32,
    offset: usize,
}

impl Magic {
    fn index(&self, occupied: Bitboard) -> usize {
        let hash = (occupied & self.mask).bits().wrapping_mul(self.factor) >> self.shift;
        self.offset
            .saturating_add(usize::try_from(hash).unwrap_or(0))
    }
}

struct Tables {
    rook: [Magic; 64],
    bishop: [Magic; 64],
    attacks: Vec<Bitboard>,
}

static TABLES: LazyLock<Tables> = LazyLock::new(|| {
    let mut attacks = Vec::new();
    let rook = array::from_fn(|index| build(index, &Direction::ROOK, &ROOK_MAGICS, &mut attacks));
    let bishop =
        array::from_fn(|index| build(index, &Direction::BISHOP, &BISHOP_MAGICS, &mut attacks));
    Tables {
        rook,
        bishop,
        attacks,
    }
});

fn walk(sq: Square, occupied: Bitboard, dirs: &[Direction], before_edge: bool) -> Bitboard {
    let mut attacks = Bitboard::EMPTY;
    for &dir in dirs {
        let steps = dir.steps_to_edge(sq).saturating_sub(u8::from(before_edge));
        let mut reached = Bitboard::from_square(sq);
        for _ in 0..steps {
            reached = dir.step(reached);
            attacks |= reached;
            if reached.intersects(occupied) {
                break;
            }
        }
    }
    attacks
}

fn build(
    index: usize,
    dirs: &[Direction],
    factors: &[u64; 64],
    attacks: &mut Vec<Bitboard>,
) -> Magic {
    let sq = u8::try_from(index)
        .ok()
        .and_then(Square::new)
        .unwrap_or(Square::A1);
    let mask = walk(sq, Bitboard::EMPTY, dirs, true);
    let magic = Magic {
        mask,
        factor: factors.get(index).copied().unwrap_or(0),
        shift: 64_u32.saturating_sub(mask.count()),
        offset: attacks.len(),
    };
    let arrangements = 1_usize.checked_shl(mask.count()).unwrap_or(0);
    attacks.resize(magic.offset.saturating_add(arrangements), Bitboard::EMPTY);

    let mut blockers = Bitboard::EMPTY;
    loop {
        if let Some(slot) = attacks.get_mut(magic.index(blockers)) {
            *slot = walk(sq, blockers, dirs, false);
        }
        blockers = Bitboard::from_bits(blockers.bits().wrapping_sub(mask.bits())) & mask;
        if blockers.is_empty() {
            break;
        }
    }
    magic
}

fn lookup(magics: &[Magic; 64], sq: Square, occupied: Bitboard) -> Bitboard {
    magics
        .get(usize::from(sq.index()))
        .and_then(|magic| TABLES.attacks.get(magic.index(occupied)))
        .copied()
        .unwrap_or(Bitboard::EMPTY)
}

pub fn rook(sq: Square, occupied: Bitboard) -> Bitboard {
    lookup(&TABLES.rook, sq, occupied)
}

pub fn bishop(sq: Square, occupied: Bitboard) -> Bitboard {
    lookup(&TABLES.bishop, sq, occupied)
}

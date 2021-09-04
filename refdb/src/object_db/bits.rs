// Add more impls here if needed. We can pull in an external library if we need more flexibility
// than can be trivially implemented

//
// 64-bit bitfield with helper functions
//
#[derive(Copy, Clone, Default)]
pub struct BitsU64 {
    bits: u64
}

impl BitsU64 {
    pub fn is_set(self, index: usize) -> bool {
        (self.bits & (1<<(index as u64))) != 0
    }

    pub fn set(&mut self, index: usize, value: bool) {
        if value {
            self.bits |= (1<<(index as u64));
        } else {
            self.bits &= !(1<<(index as u64));
        }
    }

    pub fn set_first_n(&mut self, count: usize, value: bool) {
        let (bits, _) = 1u64.overflowing_shl(count as u32);
        let (bits, _) = bits.overflowing_sub(1);
        if value {
            self.bits |= bits;
        } else {
            self.bits &= !bits;
        }
    }
}

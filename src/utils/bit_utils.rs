pub trait BitUtils {
    fn at_bit(&self, pos: usize) -> u32;
    fn get_bits(&self, start: usize, end: usize) -> u32;
}

pub trait BitSetUtils {
    fn set_bit(&self, pos: usize, value: bool) -> u32;
}

impl BitSetUtils for u32 {
    fn set_bit(&self, pos: usize, value: bool) -> u32 {
        if value {
            (1 << pos) | self
        } else {
            (!(1 << pos)) & self
        }
    }
}

impl BitUtils for u32 {
    fn at_bit(&self, pos: usize) -> u32 {
        if pos > 32 {
            panic!("pos must be less than 32")
        } else {
            (self >> pos) & 1
        }
    }

    fn get_bits(&self, start: usize, end: usize) -> u32 {
        if end < start {
            panic!("end must larger than start")
        }
        let length = end - start;
        let mask = (1 << (length + 1)) - 1;

        (self >> (start as usize)) & mask
    }
}

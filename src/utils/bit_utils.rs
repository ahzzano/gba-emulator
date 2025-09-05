pub trait BitUtils {
    fn at_bit(&self, pos: usize) -> Option<u32>;
}

impl BitUtils for u32 {
    fn at_bit(&self, pos: usize) -> Option<u32> {
        if pos > 32 { None } else { Some((self >> pos) & 1) }
    }
}

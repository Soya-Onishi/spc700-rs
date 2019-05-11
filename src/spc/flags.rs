#[derive(Copy, Clone)]
pub struct Flags {
    n: bool,
    v: bool,
    p: bool,
    b: bool,
    h: bool,
    i: bool,
    z: bool,
    c: bool,
}

impl Flags {
    pub fn assert_sign(&self) { self.n = true }
    pub fn negate_sign(&self) { self.n = false }
    pub fn sign(&self) -> bool { self.n }

    pub fn assert_overflow(&self) { self.v = true }
    pub fn negate_overflow(&self) { self.v = false }
    pub fn overflow(&self) -> bool { self.v }

    pub fn assert_page(&self) { self.p = true }
    pub fn negate_page(&self) { self.p = false }
    pub fn page(&self) -> bool { self.p }



    pub fn assert_carry(&self) { self.c = true }
    pub fn negate_carry(&self) { self.c = false }

    pub fn get(&self) -> u8 {
        macro_rules! convert {
            ($flag: ident) => {
                if self.$flag { 0 } else { 1 }
            };
        }

        let n = convert!(n) << 7;
        let v = convert!(v) << 6;
        let p = convert!(p) << 5;
        let b = convert!(b) << 4;
        let h = convert!(h) << 3;
        let i = convert!(i) << 2;
        let z = convert!(z) << 1;
        let c = convert!(c) << 0;

        n | v | p | b | h | i | z | c        
    }
}
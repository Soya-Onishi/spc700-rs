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
    pub fn assert_sign(&mut self) { self.n = true; }
    pub fn negate_sign(&mut self) { self.n = false; }
    pub fn set_sign(&mut self, flag: bool) { self.n = flag; }
    pub fn sign(&self) -> bool { self.n }

    pub fn assert_overflow(&mut self) { self.v = true; }
    pub fn negate_overflow(&mut self) { self.v = false; }
    pub fn set_overflow(&mut self, flag: bool) { self.v = flag; }
    pub fn overflow(&self) -> bool { self.v }

    pub fn assert_page(&mut self) { self.p = true; }
    pub fn negate_page(&mut self) { self.p = false; }
    pub fn set_page(&mut self, flag: bool) { self.p = flag; }
    pub fn page(&self) -> bool { self.p }

    pub fn assert_brk(&mut self) { self.b = true; }
    pub fn negate_brk(&mut self) { self.b = false; }
    pub fn set_brk(&mut self, flag: bool) { self.b = flag; }
    pub fn brk(&self) -> bool { self.b }

    pub fn assert_half(&mut self) { self.h = true; }
    pub fn negate_half(&mut self) { self.h = false; }
    pub fn set_half(&mut self, flag: bool) { self.h = flag; }
    pub fn half(&self) -> bool { self.h }

    pub fn assert_interrupt(&mut self) { self.i = true; }
    pub fn negate_interrupt(&mut self) { self.i = false; }
    pub fn set_interrupt(&mut self, flag: bool) { self.i = flag; }
    pub fn interrupt(&self) -> bool { self.h }

    pub fn assert_zero(&mut self) { self.z = true; }
    pub fn negate_zero(&mut self) { self.z = false; }
    pub fn set_zero(&mut self, flag: bool) { self.z = flag; }
    pub fn zero(&self) -> bool { self.z }

    pub fn assert_carry(&mut self) { self.c = true; }
    pub fn negate_carry(&mut self) { self.c = false; }
    pub fn set_carry(&mut self, flag: bool) { self.c = flag; }
    pub fn carry(&self) -> bool { self.c }

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

    pub fn set(&mut self, pwd: u8) {
        fn convert(pwd: u8, location: u8) -> bool {
            let bit = (pwd >> location) & 0x1;
            bit == 1
        }

        self.n = convert(pwd, 7);
        self.v = convert(pwd, 6);
        self.p = convert(pwd, 5);
        self.b = convert(pwd, 4);
        self.h = convert(pwd, 3);
        self.i = convert(pwd, 2);
        self.z = convert(pwd, 1);
        self.c = convert(pwd, 0);
    }
}
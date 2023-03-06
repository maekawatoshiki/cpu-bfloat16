use std::fmt;
use std::ops;

#[derive(Clone, Copy)]
pub struct Float32(u32);

impl Float32 {
    pub fn raw(self) -> u32 {
        self.0
    }

    pub const fn sign(self) -> u32 {
        self.0 >> 31
    }

    pub const fn exp(self) -> u32 {
        (self.0 >> 23) & 0xFF
    }

    pub const fn frac(self) -> u32 {
        self.0 & 0x7FFFFF
    }
}

impl From<f32> for Float32 {
    fn from(f: f32) -> Float32 {
        Float32(f.to_bits())
    }
}

impl From<Float32> for f32 {
    fn from(f: Float32) -> f32 {
        f32::from_bits(f.0)
    }
}

impl ops::Add for Float32 {
    type Output = Float32;

    fn add(self, rhs: Float32) -> Float32 {
        let lhs_sign = self.sign();
        let lhs_exp = self.exp();
        let lhs_frac = self.frac();
        let rhs_sign = rhs.sign();
        let rhs_exp = rhs.exp();
        let rhs_frac = rhs.frac();

        if true {
            println!("lhs_sign = {}", lhs_sign);
            println!("lhs_exp = {}", lhs_exp);
            println!("lhs_frac = {}", lhs_frac);
            println!("rhs_sign = {}", rhs_sign);
            println!("rhs_exp = {}", rhs_exp);
            println!("rhs_frac = {}", rhs_frac);
        }

        let exp = lhs_exp.max(rhs_exp);
        let lhs_frac = (lhs_frac | (1 << 23)) >> (exp - lhs_exp);
        let rhs_frac = (rhs_frac | (1 << 23)) >> (exp - rhs_exp);
        let carry = (lhs_frac + rhs_frac) > 0xffffff;
        let frac = ((lhs_frac + rhs_frac) >> carry as u32) & 0x7FFFFF;
        let exp = exp + carry as u32;

        Float32((lhs_sign << 31) | (exp << 23) | frac)
    }
}

impl fmt::Display for Float32 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", f32::from_bits(self.0))
    }
}

impl fmt::Debug for Float32 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "({}, {}, {}, {})",
            f32::from_bits(self.0),
            self.sign(),
            self.exp(),
            self.frac()
        )
    }
}

#[test]
fn addition_1() {
    let a = Float32::from(1.0);
    let b = Float32::from(2.0);
    let c = a + b;
    println!("c = {}", c);
    assert_eq!(c.raw(), Float32::from(3.0).raw());
}

#[test]
fn addition_2() {
    let a = Float32::from(1.5);
    let b = Float32::from(2.0);
    let c = a + b;
    println!("c = {}", c);
    assert_eq!(c.raw(), Float32::from(3.5).raw());
}

#[test]
fn addition_3() {
    let a = Float32::from(10.2);
    let b = Float32::from(0.1);
    let c = a + b;
    assert_eq!(
        c.raw(),
        Float32::from(10.3).raw(),
        "actual:{:?} vs expected:{:?}",
        c,
        Float32::from(10.3)
    );
}

#[test]
fn addition_4() {
    let a = Float32::from(1.25);
    let b = Float32::from(7.5);
    let c = a + b;
    assert_eq!(
        c.raw(),
        Float32::from(8.75).raw(),
        "actual:{:?} vs expected:{:?}",
        c,
        Float32::from(8.75)
    );
}

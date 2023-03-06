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
        let _sign_equals = lhs_sign == rhs_sign;

        // println!("lhs_sign = {}", lhs_sign);
        // println!("lhs_exp = {}", lhs_exp);
        // println!("lhs_frac = {}", lhs_frac);
        // println!("rhs_sign = {}", rhs_sign);
        // println!("rhs_exp = {}", rhs_exp);
        // println!("rhs_frac = {}", rhs_frac);

        if lhs_exp == 0 {
            return rhs;
        }

        if rhs_exp == 0 {
            return self;
        }

        let exp = lhs_exp.max(rhs_exp);
        let exp_diff = lhs_exp.abs_diff(rhs_exp);
        let lhs_frac = lhs_frac | (1 << 23); // TODO: We should not add (1 << 23) here.
        let rhs_frac = rhs_frac | (1 << 23);

        let lhs_bigger_abs = if lhs_exp > rhs_exp {
            true
        } else if lhs_exp < rhs_exp {
            false
        } else if lhs_frac > rhs_frac {
            true
        } else {
            false
        };
        let frac = if lhs_bigger_abs {
            (lhs_frac << 7) + ((rhs_frac << 7) >> exp_diff)
        } else {
            (rhs_frac << 7) + ((lhs_frac << 7) >> exp_diff)
        };
        let carry = (frac >> 7) > 0xFFFFFF;
        let cfrac = frac >> carry as u32;
        let q = cfrac & 0b1111111;
        let g = (q >> 6) & 1;
        let r = (q >> 5) & 1;
        let s = ((q & 0b11111) > 0) as u32;
        // println!("ulp {} g {} r {} s {}", (cfrac >> 7) & 1, g, r, s);
        let z = g & (((cfrac >> 7) & 1) | r | s);
        let frac = frac >> 7;
        let carry = frac > 0xFFFFFF;
        // println!("{:b}", q);
        let frac = frac + z;
        let frac = (frac >> carry as u32) & 0x7FFFFF;
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
    assert_eq!(
        c.raw(),
        Float32::from(3.0).raw(),
        "actual:{:?} vs expected:{:?}",
        c,
        Float32::from(3.0)
    );
}

#[test]
fn addition_2() {
    let a = Float32::from(1.5);
    let b = Float32::from(2.0);
    let c = a + b;
    println!("c = {}", c);
    assert_eq!(
        c.raw(),
        Float32::from(3.5).raw(),
        "actual:{:?} vs expected:{:?}",
        c,
        Float32::from(3.5)
    );
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
        Float32::from(10.3),
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

#[test]
fn addition_5() {
    let a = 2341.52;
    let b = 71.4;
    let c = Float32::from(a) + Float32::from(b);
    let c_ = a + b;
    assert_eq!(
        c.raw(),
        Float32::from(c_).raw(),
        "actual:{:?} vs expected:{:?}",
        c,
        Float32::from(c_)
    );
}

#[test]
fn sum() {
    let xs = [0.1, 0.2, 0.3, 0.4, 0.5];
    let sum = xs.iter().fold(0.0, |acc, x| acc + x);
    let ys = xs.iter().map(|x| Float32::from(*x));
    let sum2 = ys.fold(Float32::from(0.0), |acc, x| acc + x);
    assert_eq!(
        sum,
        f32::from(sum2),
        "actual:{:?} vs expected:{:?}",
        Float32::from(sum2),
        Float32::from(sum),
    );
}

#[test]
fn sum_rand() {
    use rand;
    let xs = (0..100)
        .map(|_| rand::random::<f32>())
        .collect::<Vec<f32>>();
    let mut sum = 0.0f32;
    let mut sum2 = Float32::from(0.0f32);
    println!("xs = {:?}", xs);
    for x in xs {
        println!("{} + {}", sum, x);
        println!("{} + {}", sum2, Float32::from(x));
        sum = sum + x;
        sum2 = sum2 + Float32::from(x);
        assert_eq!(
            sum,
            f32::from(sum2),
            "actual:{:?} vs expected:{:?}",
            Float32::from(sum2),
            Float32::from(sum),
        );
    }
    // let sum = xs.iter().fold(0.0f32, |acc, x| acc + x);
    // let ys = xs.iter().map(|&x| Float32::from(x));
    // let sum2 = ys.fold(Float32::from(0.0f32), |acc, x| acc + x);
    // assert_eq!(
    //     sum,
    //     f32::from(sum2),
    //     "actual:{:?} vs expected:{:?}",
    //     Float32::from(sum2),
    //     Float32::from(sum),
    // );
}

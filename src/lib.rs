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

// reference:
// unsigned int fadd (unsigned int a, unsigned int b) {
//
//     //Taking care of zero cases
//     if (a==0){
//         return b;
//     }
//     if (b==0){
//         return a;
//     }
//
//     //Extracting information from a and b
//     unsigned int a_sign = (a & 0x80000000)>>31;
//     unsigned int a_enc_exp = (a & 0x7f800000)>>23;
//     unsigned int a_mantissa = (a & 0x7fffff);
//
//
//     unsigned int b_sign = (b & 0x80000000)>>31;
//     unsigned int b_enc_exp = (b & 0x7f800000)>>23;
//     unsigned int b_mantissa = (b & 0x7fffff);
//
//
//     unsigned int a_significand = (a_enc_exp >= 1) ? (a_mantissa | (1<<23)) : a_mantissa;
//     unsigned int b_significand = (b_enc_exp >= 1) ? (b_mantissa | (1<<23)) : b_mantissa;
//
//
//     //Initially shifting a and b 7 bits left to increase precison for rounding
//     unsigned int a_shift_significand = (a_significand << 7);
//     unsigned int b_shift_significand = (b_significand << 7);
//
//     //Taking care of denormal numbers
//     unsigned int a_exp = ((a_enc_exp == 0) ? 1 : a_enc_exp);
//     unsigned int b_exp = ((b_enc_exp == 0) ? 1 : b_enc_exp);
//     unsigned int ans_exp;
//     unsigned int ans_significand;
//     unsigned int ans_sign;
//     bool ans_denormal = false;
//
//     /* Special Cases */
//
//     //Case when a is NaN
//     if (a_exp == 255 && a_mantissa != 0){
//         return 0x7fffffff;
//     }
//
//     //Case when b is NaN
//     if (b_exp == 255 && b_mantissa != 0){
//         return 0x7fffffff;
//     }
//
//     //Case when Infinity - Infinity
//     if (a_exp == 255 && a_mantissa == 0 && b_exp == 255 && b_mantissa == 0 && a_sign != b_sign){
//         return 0x7fffffff;
//     }
//
//     //Case when a is Infinity
//     if (a_exp == 255 && a_mantissa == 0){
//         return a;
//     }
//
//     //Case when b is Infinty
//     if (b_exp == 255 && b_mantissa == 0){
//         return b;
//     }
//
//     /* Making Exponent Same */
//
//
//     if (a_exp >= b_exp){
//         unsigned int shift = a_exp-b_exp;
//
//         b_shift_significand = (b_shift_significand >> ((shift>31) ? 31 : shift) );
//         ans_exp = a_exp;
//
//
//
//     }
//     else {
//         unsigned int shift = b_exp-a_exp;
//         a_shift_significand = (a_shift_significand >> ((shift>31) ? 31 : shift) );
//
//         ans_exp = b_exp;
//     }
//
//     /* Adding Significands */
//     if (a_sign == b_sign){
//         ans_significand = a_shift_significand + b_shift_significand;
//         ans_sign = a_sign;
//
//     }
//     else {
//         if (a_shift_significand > b_shift_significand){
//             ans_sign = a_sign;
//             ans_significand = a_shift_significand - b_shift_significand;
//         }
//         else if (a_shift_significand < b_shift_significand){
//             ans_sign = b_sign;
//             ans_significand = b_shift_significand - a_shift_significand;
//         }
//         else if ((a_shift_significand == b_shift_significand)) {
//             ans_sign = 0;
//             ans_significand = a_shift_significand - b_shift_significand;
//
//         }
//     }
//
//     /* Normalization */
//     int i;
//     for (i=31; i>0 && ((ans_significand>>i) == 0); i-- ){;}
//
//     if (i>23){
//
//         //Rounding
//         unsigned int residual = ((ans_significand&(1<<(i-23-1)))>>(i-23-1));
//
//         unsigned int sticky = 0;
//         for(int j=0;j<i-23-1;j++){
//             sticky = sticky | ((ans_significand & (1<<j))>>j);
//         }
//
//         if ((int(ans_exp) + (i-23) - 7) > 0 && (int(ans_exp) + (i-23) - 7) < 255){
//
//             ans_significand = (ans_significand>>(i-23));
//
//             ans_exp = ans_exp + (i-23) - 7;
//
//             if (residual==1 && sticky == 1){
//                 ans_significand += 1;
//
//             }
//             else if ((ans_significand&1)==1 && residual ==1 && sticky == 0){
//                 ans_significand += 1;
//
//             }
//
//             if ((ans_significand>>24)==1){
//                 ans_significand = (ans_significand>>1);
//                 ans_exp += 1;
//
//             }
//         }
//
//         //Denormal number
//         else if (int(ans_exp) + (i-23) - 7 <= 0) {
//             ans_denormal = true;
//             ans_significand = ans_significand>>7;
//             ans_significand = ans_significand<<(ans_exp-1);
//             ans_exp = 0;
//         }
//
//         //Overflow
//         else if (int(ans_exp) + (i-23) - 7 >= 255){
//             ans_significand = (1<<23);
//             ans_exp = 255;
//         }
//
//     }
//     else if (i<=23 && i!=0){
//         if ((int(ans_exp) - (23-i) - 7) > 0 && (int(ans_exp) - (23-i) - 7) < 255){
//             ans_significand = (ans_significand<<(23-i));
//             ans_exp = ans_exp - (23-i) - 7;
//         }
//
//         //Denormal Number
//         else if (int(ans_exp) - (23-i) - 7 <= 0) {
//             ans_denormal = true;
//             ans_significand = ans_significand>>7;
//             ans_significand = ans_significand<<(ans_exp-1);
//             ans_exp = 0;
//         }
//
//         //Overflow
//         else if ((int(ans_exp) - (23-i) - 7) >= 255){
//             ans_significand = (1<<23);
//             ans_exp = 255;
//         }
//
//
//     }
//
//     //When answer is zero
//     else if (i==0 && ans_exp < 255){
//         ans_exp = 0;
//     }
//
//     /* Constructing floating point number from sign, exponent and significand */
//
//     unsigned int ans = (ans_sign<<31) | (ans_exp<<23) | (ans_significand& (0x7FFFFF));
//     return ans;
// }
// C++ code above can be converted into Rust code below!

// impl ops::Add for Float32 {
//     type Output = Float32;
//
//     fn add(self, rhs: Float32) -> Float32 {
//         type ut = u32;
//         let lhs_sign = self.sign() as ut;
//         let lhs_exp = self.exp() as ut;
//         let lhs_frac = self.frac() as ut;
//         let rhs_sign = rhs.sign() as ut;
//         let rhs_exp = rhs.exp() as ut;
//         let rhs_frac = rhs.frac() as ut;
//
//         if lhs_exp == 0 {
//             return rhs;
//         }
//
//         if rhs_exp == 0 {
//             return self;
//         }
//
//         let lhs_significand = lhs_frac | (1 << 23);
//         let rhs_significand = rhs_frac | (1 << 23);
//
//         let x = 8;
//
//         let mut lhs_shift_significand = lhs_significand << x;
//         let mut rhs_shift_significand = rhs_significand << x;
//
//         let mut ans_sign = 0;
//         let mut ans_exp = 0;
//         let mut ans_significand = 0;
//         let mut ans_denormal = false;
//
//         if lhs_exp >= rhs_exp {
//             let shift = lhs_exp - rhs_exp;
//             rhs_shift_significand = rhs_shift_significand >> shift;
//             ans_exp = lhs_exp;
//         } else {
//             let shift = rhs_exp - lhs_exp;
//             lhs_shift_significand = lhs_shift_significand >> shift;
//             ans_exp = rhs_exp;
//         }
//
//         if (lhs_sign == 0 && rhs_sign == 0) {
//             ans_significand = lhs_shift_significand + rhs_shift_significand;
//             ans_sign = lhs_sign;
//         } else {
//             todo!()
//         }
//
//         // Normalization
//
//         let mut i = 31i32;
//         while i > 0 && (ans_significand >> i) == 0 {
//             i -= 1;
//         }
//
//         if i > 23 {
//             // Rounding
//             let residual = (ans_significand & (1 << (i - 23 - 1))) >> (i - 23 - 1);
//             // let residual = (ans_significand & (1 << (i - 23 - 2))) >> (i - 23 - 2);
//             let mut sticky = 0;
//             for j in 0..(i - 23 - 1) {
//                 sticky |= (ans_significand & (1 << j)) >> j;
//             }
//             if (ans_exp as i32 + (i - 23) - x) > 0 && (ans_exp as i32 + (i - 23) - x) < 255 {
//                 ans_significand = ans_significand >> (i - 23);
//                 ans_exp = ans_exp + (i - 23) as ut - x as ut;
//                 if residual == 1 && sticky == 1 {
//                     ans_significand += 1;
//                 } else if (ans_significand & 1) == 1 && residual == 1 && sticky == 0 {
//                     ans_significand += 1;
//                 }
//                 if ans_significand >> 24 == 1 {
//                     ans_significand = ans_significand >> 1;
//                     ans_exp += 1;
//                 }
//             } else if (ans_exp as i32 + (i - 23) - x) <= 0 {
//                 ans_denormal = true;
//                 ans_significand = ans_significand >> x;
//                 ans_significand = ans_significand << (ans_exp - 1);
//                 ans_exp = 0;
//             } else if (ans_exp as i32 + (i - 23) - x) >= 255 {
//                 ans_significand = (1 << 23);
//                 ans_exp = 255;
//             }
//         } else if i <= 23 && i != 0 {
//             todo!()
//         } else {
//             todo!()
//         }
//
//         let ans = (ans_sign << 31) | (ans_exp << 23) | (ans_significand & (0x7FFFFF));
//         Float32(ans)
//     }
// }

impl ops::Add for Float32 {
    type Output = Float32;

    fn add(self, rhs: Float32) -> Float32 {
        type ut = u64;
        let lhs_sign = self.sign() as ut;
        let lhs_exp = self.exp() as ut;
        let lhs_frac = self.frac() as ut;
        let rhs_sign = rhs.sign() as ut;
        let rhs_exp = rhs.exp() as ut;
        let rhs_frac = rhs.frac() as ut;
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
        println!("expdiff {}", exp_diff);
        let orig_smaller_frac = if lhs_bigger_abs { rhs_frac } else { lhs_frac };
        let s;
        if exp_diff >= 3 {
            s = (orig_smaller_frac & ((1 << exp_diff) - 1))
                & ((1 << (exp_diff.saturating_sub(2))) - 1);
        } else {
            s = 0;
            // s = (orig_smaller_frac & ((1 << exp_diff) - 1))
            //     & ((1 << (exp_diff.saturating_sub(2))) - 1);
        };
        let s = (s > 0) as ut;
        println!("s {}", s);
        let frac = if lhs_bigger_abs {
            (lhs_frac << 8) + ((rhs_frac << 8) >> exp_diff)
        } else {
            (rhs_frac << 8) + ((lhs_frac << 8) >> exp_diff)
        };
        let carry = (frac >> 8) > 0xFFFFFF;
        let cfrac = frac >> carry as ut;
        let q = cfrac & 0b111_11111;
        // let q;
        // if carry {
        //     q = frac & 0b11111111111;
        // } else {
        //     q = frac & 0b1111111111;
        // }
        let g = (q >> 7) & 1;
        let r = (q >> 6) & 1;
        // let s = ((q & 0b11111111) > 0) as ut;
        println!("ulp {} g {} r {} s {}", (cfrac >> 8) & 1, g, r, s);
        let z = g & (((cfrac >> 8) & 1) | r | s);
        let frac = frac >> 8;
        let mut frac = frac + z;
        let carry = frac > 0xFFFFFF;
        if carry {
            frac -= z;
            // let s;
            // let exp_diff = exp_diff + 1;
            // let g = (q >> 8) & 1;
            // let r = (q >> 7) & 1;
            // if exp_diff >= 3 {
            //     s = (orig_smaller_frac & ((1 << exp_diff) - 1))
            //         & ((1 << (exp_diff.saturating_sub(2))) - 1);
            // } else {
            //     s = 0;
            //     // s = (orig_smaller_frac & ((1 << exp_diff) - 1))
            //     //     & ((1 << (exp_diff.saturating_sub(2))) - 1);
            // };
            // let z = g & (((cfrac >> 8) & 1) | r | s);
            // println!("z {}", z);
            // frac = frac + (z << 1);
            let q = cfrac & 0b1111_11111;
            let g = (q >> 8) & 1;
            let r = (q >> 7) & 1;
            let z = g & (((cfrac >> 9) & 1) | r | s);
            println!("ulp {} g {} r {} s {}", (cfrac >> 9) & 1, g, r, s);
            println!("z{}", z);
            frac = frac >> 1;
        }
        println!("{}", carry);
        // let frac = frac + z;
        // let frac = (frac >> carry as ut) & 0x7FFFFF;
        let frac = frac & 0x7FFFFF;
        let exp = exp + carry as ut;

        Float32(((lhs_sign as u32) << 31) | ((exp as u32) << 23) | frac as u32)
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
fn addition_6() {
    let a = 131066.086;
    let b = 93.70508;
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
    let xs = (0..1000000)
        .map(|_| rand::random::<f32>())
        .collect::<Vec<f32>>();
    let mut sum = 0.0f32;
    let mut sum2 = Float32::from(0.0f32);
    // println!("xs = {:?}", xs);
    for x in xs {
        let x = x * 100f32;
        println!("a. {} + {}", sum, x);
        println!("b. {} + {}", sum2, Float32::from(x));
        // let save = sum;
        sum = sum + x;
        sum2 = sum2 + Float32::from(x);
        // if sum2.raw() != Float32::from(sum).raw() {
        //     println!("actual:{:?} vs expected:{:?}", sum2, Float32::from(sum));
        //     sum = sum2.into();
        // }
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

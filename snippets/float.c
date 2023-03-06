#include <stdio.h>
#include <assert.h>
#include <ctype.h>
#include <stdint.h>
#include <stdbool.h>

#define MAX(a, b) ((a) > (b) ? (a) : (b))

typedef unsigned int u32;

typedef struct {
  unsigned int frac : 23;
  unsigned int exp  : 8;
  unsigned int sign : 1;
} f32;

#define min_float 0x00000000
#define max_float 0xffffffff

    #define exponent(x) ((x << 1) >> 24)
    #define mantissa(x) ((x << 9) >> 9)
    #define sign(x) (x >> 31)

uint32_t add2(float x_, float y_) {
  const uint32_t x = *(uint32_t*)&x_;
  const uint32_t y = *(uint32_t*)&y_;
    uint32_t result_mantissa;
    uint32_t result_exponent;
    uint32_t result_sign;

    uint32_t different_sign = sign(x) ^ sign(y); //boolean but lets not do any type casting

    // catch NaN
    if (!(exponent(x) ^ 0xFF) && mantissa(x)) return x;
    if (!(exponent(y) ^ 0xFF) && mantissa(y)) return y;

    // catch Inf
    if (!(exponent(x) ^ 0xFF) && !(exponent(y) ^ 0xFF)) {
        // both are inf
        if (different_sign)
            // Inf - Inf
            return 0x7F800000 + 1; // NaN
        else
            // both Inf or -Inf
            return x;
    }
    else if (!(exponent(x) ^ 0xFF)) return x;
    else if (!(exponent(y) ^ 0xFF)) return y;

    // both numbers are non-special
    uint32_t exp_difference;
    if (different_sign) {
        exp_difference = exponent(y) + exponent(x);
    }
    else {
        // no need to account for constant BO
        // beware of underflow
        if (exponent(x) > exponent(y)) exp_difference = exponent(x) - exponent(y);
        else exp_difference = exponent(y) - exponent(x);
    }


    bool x_bigger_abs;
    if      (exponent(x) > exponent(y)) x_bigger_abs = true;
    else if (exponent(x) < exponent(y)) x_bigger_abs = false;
    else if (mantissa(x) > mantissa(y)) x_bigger_abs = true;
    else                                x_bigger_abs = false;

    if (!different_sign) {
        //both numbers have same sign (this is a sum)
        result_sign = sign(x);

        if (x_bigger_abs) {
            result_mantissa = (mantissa(x) << 1) + (mantissa(y) << 1) >> exp_difference;
            result_exponent = exponent(x);
        }
        else {
            result_mantissa = (mantissa(y) << 1) + ((mantissa(x) << 1) >> exp_difference);
            result_exponent = exponent(y);
        }
        printf("> %d\n", result_mantissa);
        /* if (result_mantissa << 31) result_mantissa = (result_mantissa >> 1) + 1; */
        /* else result_mantissa = (result_mantissa >> 1); */
        /* printf("> %d\n", result_mantissa); */
    }
    else {
        // this actually is a subtraction

        if (x_bigger_abs) {
            result_sign = sign(x);
            result_exponent = exponent(x);

            // subtract and round to 23 bit 
            // this means making room in our 32bit representation
            result_mantissa = (mantissa(x) << 1) - ((mantissa(y) << 1) >> exp_difference );
        }
        else {
            result_sign = sign(y);
            result_exponent = exponent(y);

            // subtract and round to 23 bit 
            // this means making room in our 32bit representation
            result_mantissa = (mantissa(y) << 1) - ((mantissa(x) << 1) >> exp_difference);
        }

        if (result_mantissa << 31)  result_mantissa = ((result_mantissa >> 1) + 1);
        else result_mantissa = (result_mantissa >> 1);


        // normalize mantissa
        uint32_t temp = result_mantissa << 9;
        for (uint32_t count = 0; count < 23; ++count) {
            if (!((temp << count) >> 31)) {
                result_mantissa <<= ++count; // leading 1, so shift 1 more time
                result_exponent -= count;
                break;
            }
        }
    }
    uint32_t result = result_sign << 31 | result_exponent << 23 | result_mantissa;
    return result;
}

static u32 shift32RightJamming(int a, int count)
{
  if(count == 0)       return a;
  else if(count < 32)     return (a >> count) | ((a << ((-count) & 31)) != 0);
  else            return a != 0;
}

static float add_ref(float a_, float b_)
{
  const f32 a = *(f32*)&a_;
  const f32 b = *(f32*)&b_;

	int zExp;
	u32 zFrac;

	u32 aFrac = a.frac;
	u32 bFrac = b.frac;

	int aExp = a.exp;
	int bExp = b.exp;

	u32 aSign = a.sign;
	u32 bSign = b.sign;

	u32 zSign = aSign;

	int expDiff = aExp - bExp;
	aFrac <<= 6;
	bFrac <<= 6;

	// align exponents if needed
	if(expDiff > 0)
	{
		if(bExp == 0) --expDiff;
		else bFrac |= 0x20000000;

		bFrac = shift32RightJamming(bFrac, expDiff);
		zExp = aExp;
	}
	else if(expDiff < 0)
	{
		if(aExp == 0) ++expDiff;
		else aFrac |= 0x20000000;

		aFrac = shift32RightJamming(aFrac, -expDiff);
		zExp = bExp;
	}
	else if(expDiff == 0)
	{
		if(aExp == 0) return (zSign << 31) | ((aFrac + bFrac) >> 13);

		zFrac = 0x40000000 + aFrac + bFrac;
		zExp = aExp;

    assert(0);

		return (zSign << 31) | ((zExp << 23) + (zFrac >> 7));
	}

	aFrac |= 0x20000000;
	zFrac = (aFrac + bFrac) << 1;
	--zExp;

	if((int)zFrac < 0)
	{
		zFrac = aFrac + bFrac;
		++zExp;
	}

    /* assert(0); */
  const f32 z = {
    .sign = zSign,
    .exp  = zExp,
    .frac = zFrac,
  };
  return *(float*)&z;
	// reconstruct the float; I've removed the rounding code and just truncate
	/* return (zSign << 31) | ((zExp << 23) + (zFrac >> 7)); */
}


float add(float x, float y) {
  const f32 xi = *(f32*)&x;
  const f32 yi = *(f32*)&y;
  printf("xi.exp = %d, yi.exp = %d\n", xi.exp, yi.exp);
  unsigned int exp = MAX(xi.exp, yi.exp);
  int exp_diff = xi.exp - yi.exp;
  unsigned int xi_frac = (xi.frac | (1 << 23)) >> (exp - xi.exp);
  unsigned int yi_frac = (yi.frac | (1 << 23)) >> (exp - yi.exp);
  const int carry = (xi_frac + yi_frac) > 0xffffff;

  /* printf("carry: %d\n", carry); */

  const f32 zi = {
    .sign = 0,
    .exp  = exp + carry,
    .frac = (((xi_frac + yi_frac) >> carry) & 0x7fffff) + 0,
  };
  return *(float*)&zi;

#if 0
  const int xi = *(int*)&x;
  const int yi = *(int*)&y;
  const int xi_sign = (xi >> 31) & 0x1;
  const int xi_exp = (xi >> 23) & 0x7f;
  const int xi_frac = xi & 0x7fffff;
  const int yi_sign = (yi >> 31) & 0x1;
  const int yi_exp = (yi >> 23) & 0x7f;
  const int yi_frac = yi & 0x7fffff;
  assert(xi_sign == 0);
  assert(yi_sign == 0);
  assert(xi_exp == 127);
  assert(yi_exp == 127 - 1);
  /* assert(xi_frac == 0); */
  /* assert(yi_frac == 0); */
  const int zi_sign = 0;
  const int zi_exp = 127;
  const int zi_frac = xi_frac + (yi_frac >> 1) + 0x400000;
  const int zi = (zi_sign << 31) | (zi_exp << 23) | zi_frac;
  return *(float*)&zi;
#endif
}

int main() {
  /* 1 + 8 + 7 */
  /* sign exp frac */
  float x = 10.2;
  float y = 0.1;
  uint32_t z_ = add2(x, y);
  /* float z = *(float*)&z_; */
/* #if 0 */
  float z = add(x, y);
/* #else */
  /* float z = add_ref(x, y); */
/* #endif */
  printf("truth: %f, custom: %f\n", x + y, z);
  {
    const float z = x + y;
    const f32 zz = *(f32*)&z;
    printf("truth: sign: %d, exp: %d, frac: %d\n", zz.sign, zz.exp, zz.frac);
  }
  {
    const f32 zz = *(f32*)&z;
    printf("custom: sign: %d, exp: %d, frac: %d\n", zz.sign, zz.exp, zz.frac);
  }
  return 0;
}

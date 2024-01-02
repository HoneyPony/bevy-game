
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct FixP(pub i32);

impl From<FixP> for f32 {
    fn from(mut value: FixP) -> Self {
		if value.0 == 0 { return 0.0; }

		let sign = if value.0 < 0 {
			// TODO: handle MIN_INT
			value.0 = -value.0;
			true
		} else { false };

		// If not zero, we have a leading one. Find that leading one.
		let mut exponent: i32 = 31;
		let mut mantissa_mask: u32 = 0b01111111111111111111111100000000;
		let mut mantissa_shift: i32 = 8;
		while value.0 & (1 << exponent) == 0 {
			exponent -= 1;
			mantissa_mask >>= 1;
			mantissa_shift -= 1;
		}

		// Now we can extract the mantissa.
		let mut mantissa: u32 = (value.0 as u32) & mantissa_mask;
		if mantissa_shift >= 0 {
			mantissa >>= mantissa_shift;
		}
		else {
			mantissa <<= -mantissa_shift;
		}

		// At this point, the mantissa does not have the leading one, and is
		// positioned at the end of the number -- so we just have to insert
		// the exponent field and the sign field.

		// Note that we do not have to handle denormalized numbers, because they
		// have such a small exponent that none of our fixed point values
		// correspond to it.

		let bias: i32 = 127;
		let fixed_point: i32 = 8;
		let sign_bit: u32 = if sign { 1 } else { 0 };

		let bits: u32 = 
			mantissa |
			(((exponent + bias - fixed_point) as u32) << 23) |
			(sign_bit << 31);

		return f32::from_bits(bits);
    }
}
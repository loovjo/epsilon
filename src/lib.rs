#[derive(Copy, Clone, PartialEq, Debug)]
pub struct SampleXYZ {
    pub real: f64,
    pub eps_x: f64,
    pub eps_y: f64,
    pub eps_z: f64,
}

impl std::cmp::PartialOrd for SampleXYZ {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.real.partial_cmp(&other.real)
    }
}

impl From<f64> for SampleXYZ {
    fn from(real: f64) -> Self {
        SampleXYZ {
            real,
            eps_x: 0.,
            eps_y: 0.,
            eps_z: 0.,
        }
    }
}

impl SampleXYZ {
    pub fn cnst(real: f64) -> SampleXYZ {
        SampleXYZ {
            real,
            eps_x: 0.,
            eps_y: 0.,
            eps_z: 0.,
        }
    }

    pub fn xeps(real: f64, eps_x: f64) -> SampleXYZ {
        SampleXYZ {
            real,
            eps_x,
            eps_y: 0.,
            eps_z: 0.,
        }
    }

    pub fn yeps(real: f64, eps_y: f64) -> SampleXYZ {
        SampleXYZ {
            real,
            eps_x: 0.,
            eps_y,
            eps_z: 0.,
        }
    }

    pub fn zeps(real: f64, eps_z: f64) -> SampleXYZ {
        SampleXYZ {
            real,
            eps_x: 0.,
            eps_y: 0.,
            eps_z,
        }
    }

    pub fn x(real: f64) -> SampleXYZ {
        Self::xeps(real, 1.)
    }
    pub fn y(real: f64) -> SampleXYZ {
        Self::yeps(real, 1.)
    }
    pub fn z(real: f64) -> SampleXYZ {
        Self::zeps(real, 1.)
    }

    pub fn pow(self, pow: f64) -> SampleXYZ {
        // power rule: d/dx [x^p] = p x^(p-1)
        SampleXYZ {
            real: self.real.powf(pow),
            eps_x: self.eps_x * pow * self.real.powf(pow - 1.),
            eps_y: self.eps_y * pow * self.real.powf(pow - 1.),
            eps_z: self.eps_z * pow * self.real.powf(pow - 1.),
        }
    }

    pub fn invert(self) -> SampleXYZ {
        self.pow(-1.)
    }

    pub fn sin(self) -> SampleXYZ {
        let r = self.real.sin();
        let dr = self.real.cos();
        SampleXYZ {
            real: r,
            eps_x: self.eps_x * dr,
            eps_y: self.eps_y * dr,
            eps_z: self.eps_z * dr,
        }
    }

    pub fn cos(self) -> SampleXYZ {
        let r = self.real.cos();
        let dr = -self.real.sin();
        SampleXYZ {
            real: r,
            eps_x: self.eps_x * dr,
            eps_y: self.eps_y * dr,
            eps_z: self.eps_z * dr,
        }
    }

    pub fn tan(self) -> SampleXYZ {
        self.sin() / self.cos()
    }
}

impl std::ops::Add<f64> for SampleXYZ {
    type Output = Self;

    fn add(mut self, other: f64) -> SampleXYZ {
        self.real += other;
        self
    }
}

impl std::ops::Add<SampleXYZ> for SampleXYZ {
    type Output = Self;

    fn add(mut self, other: SampleXYZ) -> SampleXYZ {
        self.real += other.real;
        self.eps_x += other.eps_x;
        self.eps_y += other.eps_y;
        self.eps_z += other.eps_z;
        self
    }
}

impl std::ops::Mul<f64> for SampleXYZ {
    type Output = Self;

    fn mul(mut self, other: f64) -> SampleXYZ {
        self.real *= other;
        self.eps_x *= other;
        self.eps_y *= other;
        self.eps_z *= other;
        self
    }
}

impl std::ops::Mul<SampleXYZ> for SampleXYZ {
    type Output = Self;

    fn mul(self, other: SampleXYZ) -> SampleXYZ {
        SampleXYZ {
            real: self.real * other.real,
            eps_x: self.eps_x * other.real + other.eps_x * self.real,
            eps_y: self.eps_y * other.real + other.eps_y * self.real,
            eps_z: self.eps_z * other.real + other.eps_z * self.real,
        }
    }
}

impl std::ops::Div<SampleXYZ> for SampleXYZ {
    type Output = Self;

    fn div(self, other: SampleXYZ) -> SampleXYZ {
        self * other.invert()
    }
}

impl std::ops::Neg for SampleXYZ {
    type Output = SampleXYZ;

    fn neg(self) -> Self::Output {
        self * -1.
    }
}

impl std::ops::Sub<f64> for SampleXYZ {
    type Output = Self;

    fn sub(self, other: f64) -> SampleXYZ {
        self + -other
    }
}

impl std::ops::Sub<SampleXYZ> for SampleXYZ {
    type Output = Self;

    fn sub(self, other: SampleXYZ) -> SampleXYZ {
        self + -other
    }
}

impl std::ops::Div<f64> for SampleXYZ {
    type Output = Self;

    fn div(self, other: f64) -> SampleXYZ {
        self * (1./other)
    }
}

macro_rules! impl_reverse {
    ($t:ty, $op:ident, $fn:ident) => {
        impl std::ops::$op<$t> for f64 {
            type Output = $t;

            fn $fn(self, other: SampleXYZ) -> Self::Output {
                <$t as std::ops::$op>::$fn(<$t as From<f64>>::from(self), other)
            }
        }
    }
}

impl_reverse!{SampleXYZ, Add, add}
impl_reverse!{SampleXYZ, Sub, sub}
impl_reverse!{SampleXYZ, Mul, mul}
impl_reverse!{SampleXYZ, Div, div}

macro_rules! impl_inplace {
    ($t:ty, $op_inplace:ident, $fn_inplace:ident, $op_outofplace:ident, $fn_outofplace:ident) => {
        impl std::ops::$op_inplace<f64> for $t {
            fn $fn_inplace(&mut self, other: f64) {
                *self = <Self as std::ops::$op_outofplace<f64>>::$fn_outofplace(*self, other);
            }
        }
        impl std::ops::$op_inplace<SampleXYZ> for $t {
            fn $fn_inplace(&mut self, other: SampleXYZ) {
                *self = <Self as std::ops::$op_outofplace<SampleXYZ>>::$fn_outofplace(*self, other);
            }
        }
    }
}

impl_inplace!{SampleXYZ, AddAssign, add_assign, Add, add}
impl_inplace!{SampleXYZ, SubAssign, sub_assign, Sub, sub}
impl_inplace!{SampleXYZ, MulAssign, mul_assign, Mul, mul}
impl_inplace!{SampleXYZ, DivAssign, div_assign, Div, div}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_const_ops() {
        assert_eq!(SampleXYZ::x(5.) + 3., SampleXYZ::x(8.));
        assert_eq!(SampleXYZ::y(5.) - 3., SampleXYZ::y(2.));
        assert_eq!(3. - SampleXYZ::y(5.), SampleXYZ::yeps(-2., -1.));
        assert_eq!(
            SampleXYZ::z(2.) * 3.,
            SampleXYZ {
                real: 6.,
                eps_x: 0.,
                eps_y: 0.,
                eps_z: 3.
            }
        );
        assert_eq!(
            SampleXYZ::x(10.).pow(2.),
            SampleXYZ {
                real: 100.,
                eps_x: 20.,
                eps_y: 0.,
                eps_z: 0.,
            }
        );
        assert_eq!(
            SampleXYZ::y(10.).invert(),
            SampleXYZ {
                real: 0.1,
                eps_x: 0.,
                eps_y: -0.01,
                eps_z: 0.,
            }
        );

        let mut v = SampleXYZ::x(3.);
        v += 7.;
        assert_eq!(v, SampleXYZ::x(10.));
    }

    #[test]
    fn test_dist() {
        let x = SampleXYZ::x(1.);
        assert_eq!(
            (x + 1.) * (x + 1.),
            x * x + 2. * x + 1.
        );
        assert_eq!(
            (x + 1.) * (x + 1.),
            (x + 1.).pow(2.),
        );
        assert_eq!(
            (x + 1.) * (x - 1.),
            x * x - 1.
        );
    }

    #[test]
    fn test_trig() {
        let x = SampleXYZ::x(0.);
        assert_eq!(
            x.sin(),
            SampleXYZ::xeps(0., 1.),
        );
        assert_eq!(
            x.cos(),
            SampleXYZ::xeps(1., 0.),
        );
        assert_eq!(
            x.tan(),
            SampleXYZ::xeps(0., 1.),
        );
    }
}

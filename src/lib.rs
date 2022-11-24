#[allow(unused)]
use paste::paste;

macro_rules! impl_reverse {
    ($t:ty, $op:ident, $fn:ident) => {
        impl std::ops::$op<$t> for f64 {
            type Output = $t;

            fn $fn(self, other: $t) -> Self::Output {
                <$t as std::ops::$op>::$fn(<$t as From<f64>>::from(self), other)
            }
        }
    }
}

macro_rules! impl_inplace {
    ($t:ty, $op_inplace:ident, $fn_inplace:ident, $op_outofplace:ident, $fn_outofplace:ident) => {
        impl std::ops::$op_inplace<f64> for $t {
            fn $fn_inplace(&mut self, other: f64) {
                *self = <Self as std::ops::$op_outofplace<f64>>::$fn_outofplace(*self, other);
            }
        }
        impl std::ops::$op_inplace<$t> for $t {
            fn $fn_inplace(&mut self, other: $t) {
                *self = <Self as std::ops::$op_outofplace<$t>>::$fn_outofplace(*self, other);
            }
        }
    }
}


#[macro_export]
macro_rules! make_dual {
    ($name:ident, $($comp:ident,)+) => { paste! {
        #[derive(Copy, Clone, PartialEq, Debug)]
        pub struct $name {
            pub real: f64,
            $(
                pub [< eps_ $comp >]: f64,
            )+
        }

        impl std::cmp::PartialOrd for $name {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                self.real.partial_cmp(&other.real)
            }
        }

        impl From<f64> for $name {
            fn from(real: f64) -> Self {
                $name {
                    real,
                    $(
                        [<eps_$comp>]: 0.,
                    )+
                }
            }
        }

        impl $name {
            $(
                pub fn [<eps_ $comp>](real: f64, [<eps_ $comp>]: f64) -> Self {
                    Self {
                        [<eps_ $comp>]: [<eps_ $comp>],
                        .. Self::from(real)
                    }
                }
            )+
            $(
                pub fn $comp(real: f64) -> Self {
                    Self::[<eps_ $comp>](real, 1.)
                }
            )+

            pub fn pow(self, pow: f64) -> Self {
                // power rule: d/dx [x^p] = p x^(p-1)
                Self {
                    real: self.real.powf(pow),
                    $(
                        [<eps_ $comp>]: self.[<eps_ $comp>] * pow * self.real.powf(pow - 1.),
                    )+
                }
            }

            pub fn invert(self) -> Self {
                self.pow(-1.)
            }

            pub fn sin(self) -> Self {
                let r = self.real.sin();
                let dr = self.real.cos();

                Self {
                    real: r,
                    $(
                        [<eps_ $comp>]: self.[<eps_ $comp>] * dr,
                    )+
                }
            }

            pub fn cos(self) -> Self {
                let r = self.real.cos();
                let dr = -self.real.sin();

                Self {
                    real: r,
                    $(
                        [<eps_ $comp>]: self.[<eps_ $comp>] * dr,
                    )+
                }
            }

            pub fn tan(self) -> Self {
                self.sin() / self.cos()
            }
        }

        impl std::ops::Add<f64> for $name {
            type Output = Self;

            fn add(mut self, other: f64) -> Self::Output {
                self.real += other;
                self
            }
        }

        impl std::ops::Add<$name> for $name {
            type Output = Self;

            fn add(mut self, other: Self) -> Self::Output {
                self.real += other.real;
                $(
                    self.[<eps_ $comp>] += other.[<eps_ $comp>];
                )+
                self
            }
        }

        impl std::ops::Mul<f64> for $name {
            type Output = Self;

            fn mul(mut self, other: f64) -> Self::Output {
                self.real *= other;
                $(
                    self.[<eps_ $comp>] *= other;
                )+
                self
            }
        }

        impl std::ops::Neg for $name {
            type Output = Self;

            fn neg(self) -> Self::Output {
                self * -1.
            }
        }

        impl std::ops::Mul<$name> for $name {
            type Output = Self;

            fn mul(self, other: Self) -> $name {
                Self {
                    real: self.real * other.real,
                    $(
                        [<eps_ $comp>]: self.[<eps_ $comp>] * other.real + other.[<eps_ $comp>] * self.real,
                    )+
                }
            }
        }

        impl std::ops::Div<$name> for $name {
            type Output = Self;

            fn div(self, other: Self) -> Self::Output {
                self * other.invert()
            }
        }

        impl std::ops::Sub<f64> for $name {
            type Output = Self;

            fn sub(self, other: f64) -> Self::Output {
                self + -other
            }
        }

        impl std::ops::Sub<$name> for $name {
            type Output = Self;

            fn sub(self, other: $name) -> Self::Output {
                self + -other
            }
        }

        impl std::ops::Div<f64> for $name {
            type Output = Self;

            fn div(self, other: f64) -> Self::Output {
                self * (1./other)
            }
        }

        impl_reverse!{$name, Add, add}
        impl_reverse!{$name, Sub, sub}
        impl_reverse!{$name, Mul, mul}
        impl_reverse!{$name, Div, div}
        impl_inplace!{$name, AddAssign, add_assign, Add, add}
        impl_inplace!{$name, SubAssign, sub_assign, Sub, sub}
        impl_inplace!{$name, MulAssign, mul_assign, Mul, mul}
        impl_inplace!{$name, DivAssign, div_assign, Div, div}
    } }
}

#[cfg(any(test, doc))]
make_dual! {
    SampleXYZ,
    x, y, z,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_const_ops() {
        assert_eq!(SampleXYZ::x(5.) + 3., SampleXYZ::x(8.));
        assert_eq!(SampleXYZ::y(5.) - 3., SampleXYZ::y(2.));
        assert_eq!(3. - SampleXYZ::y(5.), SampleXYZ::eps_y(-2., -1.));
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
            SampleXYZ::eps_x(0., 1.),
        );
        assert_eq!(
            x.cos(),
            SampleXYZ::eps_x(1., 0.),
        );
        assert_eq!(
            x.tan(),
            SampleXYZ::eps_x(0., 1.),
        );
    }
}


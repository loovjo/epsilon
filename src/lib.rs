//! # `epsilon` - Fast autograd using dual numbers
//!
//! Dual numbers are a straightforward awy of doing forward gradient
//! propagation, i.e. keep track of derivatives for all expressions, to
//! automatically differentiate a function without storing a computation graph.
//!
//! Using dual numbers, one can augment their numbers with a "dual" part,
//! representing the derivative of the term with respect to some input variable.
//! The input variable has a unit dual part of 1, and each resulting expression
//! has a dual part with the derivaite up to that point.
//!
//! This can be trivially extended to multiple variables by storing one dual
//! part per input variable.
//!
//! One can find a more in-depth at [Wikipedia](https://en.wikipedia.org/wiki/Dual_number)
//!
//! ---
//!
//! This crate statically generates code for dual numbers using macros, meaning
//! one can provide names for each dependent variable, and corresponding
//! methods will be generated with names reflecting the name of the variable.
//!
//! The interface exposed by the types generated by this crate is very similar
//! to that of the standard numerical rust types, meaning most code using
//! [f64](f64) should be very straightforward to convert to using dual numbers.
//!
//! Example usage:
//!
//! ```
//! use epsilon::make_dual;
//! // We want to compute dz/dx and dz/dy for z = x^2+y*sin(y) at x=5, y=7
//! make_dual! { MyDual, x, y } // Create a dual number with terms `x` and `y`
//!
//! let (x, y) = (MyDual::x(5.), MyDual::y(7.)); // Perform the calculations, and compute the derivative at x=5, y=7
//!
//! let z = x.powf(2.) + y * y.sin();
//!
//! let dzdx = z.d_dx();
//! let dzdy = z.d_dy();
//!
//! assert_eq!(dzdx, 10.); // 2 * x
//! assert_eq!(dzdy, 5.934302379121921); // y*cos(y) + sin(y)
//! ```

// Reexport to access from macros
#[doc(hidden)]
pub use paste::paste;

#[macro_export]
/// # Create a dual number
/// `$name` specifies the name of the type, $inner specifies the backing type
/// (either `f32 `or `f64`, defaults to `f64`), and each `$comp` is a dual
/// compoment of the type.
///
/// For example, `make_dual! { SampleXYZ: f64, x, y, z, }` will generate a struct
/// ```
/// struct SampleXYZ {
///     real: f64,
///     eps_x: f64,
///     eps_y: f64,
///     eps_z: f64,
/// }
/// ```
///
/// An instance can be created using either the `$comp_eps` function, giving a
/// specified real and dual part, or using the `$comp` function, giving a
/// specified real part and a unit dual part.
///
/// All other functions such as `sin`, and trait implementations such as `Add`
/// will all propagate the dual parts of the numbers.
///
/// Functions with discontinuous intervals or points (such as `abs` at 0, or
/// `sqrt` at negative numbers) may panic if applied at a discontinuous point.
/// These methods all have a `try_`-prefix variant returning an `Option<Self>`.

macro_rules! make_dual {
    ($name:ident, $($comp:ident),+) => { make_dual!{ $name: f64, $($comp,)+ } };
    ($name:ident, $($comp:ident,)+) => { make_dual!{ $name: f64, $($comp,)+ } };
    ($name:ident: $inner:ty, $($comp:ident),+) => { make_dual!{ $name: $inner, $($comp,)+ } };
    ($name:ident: $inner:ty, $($comp:ident,)+) => { $crate::paste! {
        macro_rules! impl_reverse {
            ($t:ty, $op:ident, $fn:ident) => {
                impl std::ops::$op<$t> for $inner {
                    type Output = $t;

                    fn $fn(self, other: $t) -> Self::Output {
                        <$t as std::ops::$op>::$fn(<$t as From<$inner>>::from(self), other)
                    }
                }
            };
        }

        macro_rules! impl_inplace {
            ($t:ty, $op_inplace:ident, $fn_inplace:ident, $op_outofplace:ident, $fn_outofplace:ident) => {
                impl std::ops::$op_inplace<$inner> for $t {
                    fn $fn_inplace(&mut self, other: $inner) {
                        *self = <Self as std::ops::$op_outofplace<$inner>>::$fn_outofplace(*self, other);
                    }
                }
                impl std::ops::$op_inplace<$t> for $t {
                    fn $fn_inplace(&mut self, other: $t) {
                        *self = <Self as std::ops::$op_outofplace<$t>>::$fn_outofplace(*self, other);
                    }
                }
            };
        }

        /// Dual type
        #[derive(Copy, Clone, PartialEq, Debug)]
        pub struct $name {
            /// The real value of the dual type
            pub real: $inner,
            $(
                /// Dual component
                pub [< eps_ $comp >]: $inner,
            )+
        }

        impl std::cmp::PartialOrd for $name {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                self.real.partial_cmp(&other.real)
            }
        }

        impl From<$inner> for $name {
            fn from(real: $inner) -> Self {
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
                /// Create instance with specified real and dual part
                pub fn [<eps_ $comp>](real: $inner, [<eps_ $comp>]: $inner) -> Self {
                    Self {
                        [<eps_ $comp>]: [<eps_ $comp>],
                        .. Self::from(real)
                    }
                }
            )+
            $(
                /// Create instance with specified real part and unit dual part
                pub fn $comp(real: $inner) -> Self {
                    Self::[<eps_ $comp>](real, 1.)
                }
            )+

            $(
                /// Derivative with respect to component
                /// Shorthand for self.eps_$comp
                pub fn [<d_d $comp>](self) -> $inner {
                    self.[<eps_ $comp>]
                }
            )+

            /// Raise `self` to `pow`
            pub fn powf(self, pow: $inner) -> Self {
                // power rule: d/dx [x^p] = p x^(p-1)
                Self {
                    real: self.real.powf(pow),
                    $(
                        [<eps_ $comp>]: self.[<eps_ $comp>] * pow * self.real.powf(pow - 1.),
                    )+
                }
            }

            /// Invert `self` (`1./self`)
            pub fn invert(self) -> Self {
                self.powf(-1.)
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

        impl std::ops::Add<$inner> for $name {
            type Output = Self;

            fn add(mut self, other: $inner) -> Self::Output {
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

        impl std::ops::Mul<$inner> for $name {
            type Output = Self;

            fn mul(mut self, other: $inner) -> Self::Output {
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

        impl std::ops::Sub<$inner> for $name {
            type Output = Self;

            fn sub(self, other: $inner) -> Self::Output {
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

            fn div(self, other: $inner) -> Self::Output {
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

trait Numerical:
    Copy
    + std::fmt::Debug
    + std::fmt::Display
    + std::ops::Add<Output = Self>
    + std::ops::Sub<Output = Self>
    + std::ops::Mul<Output = Self>
    + std::ops::Div<Output = Self>
    + std::ops::AddAssign
    + std::ops::SubAssign
    + std::ops::MulAssign
    + std::ops::DivAssign
{
    fn powf(self, pow: f64) -> Self;
    fn invert(self) -> Self;
    fn sin(self) -> Self;
    fn cos(self) -> Self;
    fn tan(self) -> Self;
}

impl Numerical for f64 {
    fn powf(self, pow: f64) -> Self {
        f64::powf(self, pow)
    }

    fn invert(self) -> Self {
        1. / self
    }

    fn sin(self) -> Self {
        f64::sin(self)
    }

    fn cos(self) -> Self {
        f64::cos(self)
    }

    fn tan(self) -> Self {
        f64::tan(self)
    }
}

#[cfg(any(test, doc))]
/// # Sample type
///
pub mod sample {
    //! As all types are generated at compile time using [`make_dual`](crate::make_dual), this module serves to show an example generated dual type.
    //!
    //! The type is called `SampleXYZ` and has the fields (components) `x`, `y` and `z`. Function names such as `eps_x` are generated based on the names of the components.
    crate::make_dual! { SampleXYZ: f64, x, y, z }
}

#[cfg(test)]
mod tests {
    use super::sample::SampleXYZ;

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
            SampleXYZ::x(10.).powf(2.),
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
        assert_eq!((x + 1.) * (x + 1.), x * x + 2. * x + 1.);
        assert_eq!((x + 1.) * (x + 1.), (x + 1.).powf(2.),);
        assert_eq!((x + 1.) * (x - 1.), x * x - 1.);
    }

    #[test]
    fn test_diff() {
        let x = SampleXYZ::x(1.);
        let z = x * x;
        assert_eq!(z.d_dx(), 2.);
    }

    #[test]
    fn test_trig() {
        let x = SampleXYZ::x(0.);
        assert_eq!(x.sin(), SampleXYZ::eps_x(0., 1.),);
        assert_eq!(x.cos(), SampleXYZ::eps_x(1., 0.),);
        assert_eq!(x.tan(), SampleXYZ::eps_x(0., 1.),);
    }
}

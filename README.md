# `epsilon` - Fast autograd using dual numbers

Dual numbers are a straightforward way of doing forward gradient
propagation, i.e. keep track of derivatives for all expressions, to
automatically differentiate a function without storing a computation graph.

Using dual numbers, one can augment their numbers with a "dual" part,
representing the derivative of the term with respect to some input variable.
The input variable has a unit dual part of 1, and each resulting expression
has a dual part with the derivaite up to that point.

This can be trivially extended to multiple variables by storing one dual
part per input variable.

One can find a more in-depth at [Wikipedia](https://en.wikipedia.org/wiki/Dual_number)

---

This crate statically generates code for dual numbers using macros, meaning
one can provide names for each dependent variable, and corresponding
methods will be generated with names reflecting the name of the variable.

The interface exposed by the types generated by this crate is very similar
to that of the standard numerical rust types, meaning most code using
[f64](f64) should be very straightforward to convert to using dual numbers.

Example usage:

```
use epsilon::make_dual;
// We want to compute dz/dx and dz/dy for z = x^2+y*sin(y) at x=5, y=7
make_dual! { MyDual, x, y } // Create a dual number with terms `x` and `y`

let (x, y) = (MyDual::x(5.), MyDual::y(7.)); // Perform the calculations, and compute the derivative at x=5, y=7

let z = x.pow(2.) + y * y.sin();

let dzdx = z.d_dx();
let dzdy = z.d_dy();

assert_eq!(dzdx, 10.); // 2 * x
assert_eq!(dzdy, 5.934302379121921); // y*cos(y) + sin(y)
```

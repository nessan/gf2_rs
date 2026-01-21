# The `BitPoly` Type

## Introduction

A [`BitPoly`] is a polynomial over [GF(2)]:

```txt
p(x) = c0 + c1 x + c2 x^2 + ... + cn x^n
```

where each coefficient `ci` is either `0` or `1,` and arithmetic is done modulo `2`.

The polynomial's coefficients are stored in a [`BitVec`], and the generic `Word` parameter specifies the underlying [`Unsigned`] integer type used to back that vector.
The default `Word` is usually the most efficient type for the target platform.
On most modern CPU's, that `usize` will be a 64-bit unsigned integer.

If your application requires a large number of low-degree polynomials, you might consider using `u8` as the `Word` type to save memory.

**Note:** You might notice that many of the [doctests] in the library use 8-bit underlying words. The reason is that we want to exercise the various methods across word boundaries for modest, easily readable, bit-polynomials.

### Example

Here is a simple example where we create a `BitPoly` and then compute `x^2` modulo that polynomial:

```rust
use gf2::*;
let p: BitPoly = BitPoly::ones(1);
println!("Polynomial p(x) = {}", p);
println!("x^2 mod p(x) = {}", p.reduce_x_to_the(2));
```

Output:

```txt
Polynomial p(x) = 1 + x
x^2 mod p(x) = 1
```

**Note:** This makes sense as `(x + 1) * (x - 1) = x^2 - 1` so `x^2 = q(x)*r(x) + 1` where `q(x) = x - 1`.<br> Therefore `x^2 ≡ 1 mod p(x)`.

## Constructors

The following constructors are available for `BitPoly` objects:

| Method Name                    | Description                                                  |
| ------------------------------ | ------------------------------------------------------------ |
| [`BitPoly::new`]               | Creates the zero bit-polynomial `p(x) ≡ 0` with no coefficients. |
| [`BitPoly::from_coefficients`] | Constructs a bit-vector by _consuming_ the passed bit-vector of coefficients. |
| [`BitPoly::zero`]              | Returns the constant zero polynomial `p(x) ≡ 0`.             |
| [`BitPoly::one`]               | Returns the constant polynomial `p(x) ≡ `                    |
| [`BitPoly::constant`]          | Returns `p(x) ≡ 0` or `p(x) ≡ 1` depending on the passed value. |
| [`BitPoly::zeros`]             | Returns `p(x) ≡ 0 + 0x + 0x^2 + ... + 0x^n` where `n` is the passed argument. |
| [`BitPoly::ones`]              | Returns `p(x) ≡ 1 + x + x^2 + ... + x^d` where `d` is the passed argument. |
| [`BitPoly::x_to_the`]          | Returns `p(x) ≡ x^n` where `n` is the passed argument.       |
| [`BitPoly::from_fn`]           | Returns a polynomial with coefficients determined by repeatedly calling a function. |
| [`BitPoly::random`]            | Returns a random polynomial of a particular degree.          |
| [`BitPoly::random_seeded`]     | Returns a random polynomial of a particular degree. The RNG is seeded for repeatability. |

It is worth noting that there are multiple representations possible for the zero polynomial `p(x) ≡ 0`.
The _empty polynomial_ (the one with no coefficients at all) is considered to be a zero polynomial.
So also is the polynomial `0 + 0x + 0x^2 + ... + 0x^n` for any `n ≥ 0` --- this is what is returned by [`BitPoly::zeros`].

**Note:** The [`Default`] trait for `BitPoly` forwards to the [`BitPoly::new`] constructor.

## Queries

There following methods query a `BitPoly` object:

| Method Name              | Description                                                                |
| ------------------------ | -------------------------------------------------------------------------- |
| [`BitPoly::degree`]      | Returns the degree of the polynomial (returns 0 for any zero polynomial).  |
| [`BitPoly::len`]         | Returns the number of polynomial coefficients.                             |
| [`BitPoly::is_zero`]     | Returns `true` if this is some form of the zero polynomial.                |
| [`BitPoly::is_non_zero`] | Returns `true` if this is not some form of the zero polynomial.            |
| [`BitPoly::is_one`]      | Returns `true` if this is the polynomial $p(x) \coloneqq 1$ .              |
| [`BitPoly::is_constant`] | Returns `true` if this is the polynomial $p(x) \coloneqq 0 \text{ or } 1$. |
| [`BitPoly::is_monic`]    | Returns `true` if there are no high order zero coefficients.               |
| [`BitPoly::is_empty`]    | Returns `true` if this polynomial has no coefficients                      |

A polynomial is considered _monic_ if its leading coefficient (the coefficient of the highest-degree term) is 1.

For example, the polynomial `p(x) ≡  x^3 + x + 1` is monic, while `p(x) ≡  0x^4 + x^3 + x + 1` is not monic because its leading coefficient is 0.
Both those polynomials have degree 3.

## Coefficient Access

There are methods to access and modify the polynomial coefficients either individually or as a whole:

| Method Name                | Description                                                                 |
| -------------------------- | --------------------------------------------------------------------------- |
| [`BitPoly::coefficients`]  | Returns a read-only reference to the coefficient bit-vector.                |
| [`BitPoly::coeff`]         | Returns the value of a coefficients as a boolean.                           |
| [`BitPoly::set_coeff`]     | Sets a coefficient to the passed boolean value.                             |
| [`BitPoly::clear`]         | Sets the polynomial back to `p(x) ≡ 0`.                                     |
| [`BitPoly::resize`]        | Resizes the polynomial to have the `n` coefficients (added ones are zeros). |
| [`BitPoly::shrink_to_fit`] | Calls [`BitVec::shrink_to_fit`] on the coefficient bit-vector.              |
| [`BitPoly::make_monic`]    | Kills any high order zero coefficients to make the polynomial _monic_.      |

The [`BitPoly::coeff`] and [`BitPoly::set_coeff`] methods range check the coefficient index in debug builds.

**Note:** We have also implemented the [`std::ops::Index`] foreign trait to provide indexing operator for coefficient access. That forwards to the [`BitPoly::coeff`] method.

## Arithmetic Operations

We have all the usual arithmetic operations defined for bit-polynomial objects.
Addition and subtraction operations are identical since we are working over GF(2).

**Note:** Unlike bit-vectors, we obviously do not require bit-polynomials to be of the same degree/size to perform arithmetic operations.

| Method                      | Description                                                                   |
| --------------------------- | ----------------------------------------------------------------------------- |
| [`BitPoly::plus_eq`]        | Adds the passed bit-polynomial to this one.                                   |
| [`BitPoly::minus_eq`]       | Subtracts the passed bit-polynomial from this one.                            |
| [`BitPoly::plus`]           | Adds two bit-polynomials and returns the result as a new bit-polynomial.      |
| [`BitPoly::minus`]          | Subtracts two bit-polynomials and returns the result as a new bit-polynomial. |
| [`BitPoly::convolved_with`] | Convolves two bit-polynomials and returns the result as a new bit-polynomial. |

Multiplication of two arbitrary bit-polynomials, `p(x)` and `q(x)`, is performed using the [`BitStore::convolved_with`] method, which implements efficient convolutions of bit-stores.

**Note:** We have also implemented the [`std::ops::AddAssign`], [`std::ops::SubAssign`], [`std::ops::MulAssign`], [`std::ops::Add`], [`std::ops::Sub`], and [`std::ops::Mul`], foreign traits to provide operator overloads for the arithmetic operations. Those implementations forward to the methods above.

### Extra Fast Methods

There are a couple of extra "fast" methods for common arithmetic operations:

| Method Name                 | Description                                                             |
| --------------------------- | ----------------------------------------------------------------------- |
| [`BitPoly::square_into`]    | Computes the polynomial `p(x)^2` and puts it in the passed destination. |
| [`BitPoly::squared`]        | Constructs the polynomial `p(x)^2` as a new bit-polynomial              |
| [`BitPoly::times_x_to_the`] | Performs the in-place operation `p(x) 🡒 x^n p(x)` .                     |

The squaring operation is optimised since in GF(2), squaring a polynomial simply involves inserting zero coefficients between each existing coefficient (see [`BitStore::riffled_into`] method).

The [`BitPoly::square_into`] is passed a pre-allocated polynomial to store the result --- this is important for algorithms that require _repeated squaring_ to avoid unnecessary allocations. See for example, the [modular reduction] technical note for details.

Multiplication by `x^n` is also optimised since it simply involves bit-shifting the coefficients.

Those methods are much faster than using the general multiplication operator `p(x) * q(x)` when `q(x) = x^n` or `p(x)`.

## Polynomial Evaluation

There are methods to evaluate a bit-polynomial for a scalar value or for any _square_ bit-matrix:

| Method Name              | Description                                                |
| ------------------------ | ---------------------------------------------------------- |
| [`BitPoly::eval_bool`]   | Evaluates the polynomial for bit value argument.           |
| [`BitPoly::eval_matrix`] | Evaluates the polynomial for a square bit-matrix argument. |

Matrix evaluation uses [Horner's method] to evaluate `p(M)` where `M` is a square matrix.
The result is returned as a new bit-matrix.

<div style="border: 2px solid #ccc; border-radius: 8px; padding: 16px; margin: 16px 0; display: flex; align-items: center;">
<div style="font-size: 48px; margin-right: 12px; color: #666;">📝</div>

If the compiler supports the `unboxed_closures` & `fn_traits` features, we can use the `BitPoly` type as a
function over the field GF(2). You can then use the natural call `p(x)` instead of the long hand `p.eval_bool(x)`.
You can also write `p(M)` instead of the long hand `p.eval_matrix(M)`.
Enable the use of unstable Rust features by setting the `RUSTC_BOOTSTRAP` environment variable to `1` when building your project.
This allows us to use nightly-only features on _stable_ Rust compilers.
You also need to build with the `unstable` feature enabled for the `gf2` crate.

</div>

## Modular Reduction

We have a method to compute `x^N mod p(x)` where `p(x)` is a bit-polynomial and `N` is a potentially huge integer:

| Method Name                           | Description                                                               |
| ------------------------------------- | ------------------------------------------------------------------------- |
| [`BitPoly::reduce_x_to_the`]          | Returns the polynomial`x^N mod p(x)` where `N` is the passed integer.     |
| [`BitPoly::reduce_x_to_the_2_to_the`] | Returns the polynomial`x^(2^N) mod p(x)` where `N` is the passed integer. |
| [`BitPoly::reduce_x_to_power`]        | Returns the polynomial`x^e mod p(x)` where `e` is either `N` or `2^N`.    |

This method can handle _very_ large exponents. <br>
See the [modular reduction] technical note for more details.

## Stringification

The following methods return a string representation for a bit-polynomial.

| Method                               | Description                                                                  |
| ------------------------------------ | ---------------------------------------------------------------------------- |
| [`BitPoly::to_string_with_var`]      | Zero coefficients are not shown, you set the "variable" name.                |
| [`BitPoly::to_full_string_with_var`] | Zero coefficients are shown, you set the "variable" name.                    |
| [`std::string::ToString::to_string`] | Delegates to [`BitPoly::to_string_with_var`] using `x` as the variable name. |
| [`BitPoly::to_full_string`]          | Zero coefficients are shown, variable is `x`.                                |

**Note:** We have also implemented the [`std::fmt::Display`] foreign trait to provide default stringification for bit-polynomials. That outputs the bit-polynomial in terms of `x`. Zero coefficients are not shown unless the `alternate` format specifier is used.

### Example

```rust
use gf2::*;
let p: BitPoly = BitPoly::ones(3);
assert_eq!(p.to_string(), "1 + x + x^2 + x^3");
assert_eq!(p.to_string_with_var("M"), "1 + M + M^2 + M^3");
```

## Foreign Traits

We have implemented the following foreign traits for any individual bit-polynomial:

| Trait Name            | Description                                   |
| --------------------- | --------------------------------------------- |
| [`std::ops::Index`]   | Forwarded to [`BitPoly::coeff`].              |
| [`std::fmt::Display`] | Forwarded to [`BitPoly::to_string_with_var`]. |
| [`std::fmt::Debug`]   | Forwarded to [`BitPoly::to_full_string`].     |

We have also implemented arithmetic traits from the standard library for pairs of bit-polynomials:

| Trait Name              | Description                              |
| ----------------------- | ---------------------------------------- |
| [`std::ops::AddAssign`] | Forwarded to [`BitPoly::plus_eq`]        |
| [`std::ops::SubAssign`] | Forwarded to [`BitPoly::minus_eq`]       |
| [`std::ops::MulAssign`] | Forwarded to [`BitPoly::convolved_with`] |
| [`std::ops::Add`]       | Forwarded to [`BitPoly::plus`]           |
| [`std::ops::Sub`]       | Forwarded to [`BitPoly::minus`]          |
| [`std::ops::Mul`]       | Forwarded to [`BitPoly::convolved_with`] |

These pairwise traits were implemented for all combinations of references and values for the two types:

- `BitPoly` and `BitPoly`
- `&BitPoly` and `BitPoly`
- `BitPoly` and `&BitPoly`
- `&BitPoly` and `&BitPoly`

For example, if `p` and `q` are bit-polynomial instances, then the following expressions will all work:

```rust
use gf2::*;
let p: BitPoly = BitPoly::random(7);
let q: BitPoly = BitPoly::random(10);
let a = &p + &q;    // `a` is a new `BitPoly`; `p` and `q` are both preserved.
let b = &p + q;     // `b` is a new `BitPoly`; we cannot use `q` again.
let c = p + &b;     // `c` is a new `BitPoly`; we cannot use `p` again.
let d = b + c;      // `d` is a new `BitPoly`; we cannot use either `b` or `c` again.
```

The need to use references in the most common use case `let r = &p + &q;` is annoying but unavoidable due to Rust's ownership rules.
The C++ equivalent `auto r = p + q;`, where the `p` and `q` operands are preserved, is arguably more ergonomic.

<!-- Internal Reference Links -->

[`BitPoly`]: crate::BitPoly
[`BitVec`]: crate::BitVec
[`Unsigned`]: crate::Unsigned

<!-- External Reference Links -->

[doctests]: https://doc.rust-lang.org/rustdoc/documentation-tests.html
[GF(2)]: https://en.wikipedia.org/wiki/Finite_field_arithmetic
[modular reduction]: https://nessan.github.io/gf2/Reduction.html
[Horner's method]: https://en.wikipedia.org/wiki/Horner%27s_method

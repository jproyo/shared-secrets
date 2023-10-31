//! This module was copy from https://github.com/bilowik/sss-rs/blob/master/src/geometry/galois_polynomial.rs for educational purpose only.
//! Since this module was not expose publicly we cannot use the original to proactive sharing
//! secret mechanism.
use galois_2p8::*;
use lazy_static::*;
use std::ops::{Add, Deref, Div, Mul, Sub};

lazy_static! {
    // The field to use for all the finite field arithmetic
    static ref FIELD: PrimitivePolynomialField =
        PrimitivePolynomialField::new(field::PRIMITIVES[0]).unwrap();
}

/// A wrapper around u8, used to implement arithmetic operations over a finite field
#[derive(Clone, Copy, Debug)]
pub struct Coeff(pub u8);

impl Deref for Coeff {
    type Target = u8;

    fn deref(&self) -> &u8 {
        &self.0
    }
}

impl From<u8> for Coeff {
    fn from(source: u8) -> Coeff {
        Coeff(source)
    }
}

impl Add for Coeff {
    type Output = Coeff;
    fn add(self, rhs: Coeff) -> Coeff {
        Coeff(FIELD.add(*self, *rhs))
    }
}
impl Sub for Coeff {
    type Output = Coeff;
    fn sub(self, rhs: Coeff) -> Coeff {
        Coeff(FIELD.sub(*self, *rhs))
    }
}
impl Mul for Coeff {
    type Output = Coeff;
    fn mul(self, rhs: Coeff) -> Coeff {
        Coeff(FIELD.mult(*self, *rhs))
    }
}
impl Div for Coeff {
    type Output = Coeff;
    fn div(self, rhs: Coeff) -> Coeff {
        Coeff(FIELD.div(*self, *rhs))
    }
}

#[derive(Clone, Debug)]
pub struct GaloisPolynomial {
    coeffs: Vec<Coeff>,
}

impl GaloisPolynomial {
    /// Constructs a polynomail with no coefficients
    pub fn new() -> GaloisPolynomial {
        Self {
            coeffs: Vec::with_capacity(8),
        }
    }

    /// Sets the coefficient at the given index to the given co
    pub fn set_coeff(&mut self, co: Coeff, index: usize) {
        if self.coeffs.len() < index + 1 {
            self.coeffs.resize_with(index + 1, || Coeff(0));
        }

        self.coeffs[index] = co;
    }

    /// Calculates the y-value given an x-value
    pub fn get_y_value(&self, x_val: u8) -> u8 {
        let x_val_coeff = Coeff(x_val);
        // This needs to be reversed since we are assuming the y-intercept in the field is the
        // left-most byte rather than the right-most.
        *(&self.coeffs)
            .iter()
            .rev()
            .fold(Coeff(0u8), |acc, co| (acc * x_val_coeff) + *co)
    }
}


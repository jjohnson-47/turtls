use crate::finite_field::FieldElement;

use super::{EllipticCurve, ProjectivePoint};
/// A point on an elliptic curve in affine representation.
#[derive(Clone, Debug, Copy, Eq, PartialEq)]
pub struct AffinePoint<C: EllipticCurve> {
    x: FieldElement<C>,
    y: FieldElement<C>,
}

impl<C: EllipticCurve> AffinePoint<C> {
    /// Returns the x-value of `self`.
    pub fn x(&self) -> &FieldElement<C> {
        &self.x
    }

    /// Returns the y-value of `self`.
    pub fn y(&self) -> &FieldElement<C> {
        &self.y
    }

    /// Converts `self` into its projective representation.
    pub const fn as_projective(self) -> ProjectivePoint<C> {
        // TODO: is there a better way to do this?

        // # SAFETY: The projective value is still on the curve.
        unsafe { ProjectivePoint::new_unsafe(self.x, self.y, C::ONE) }
    }

    /// Creates a new [`Point`] without verifying that it is on the curve specified b `P`.
    ///
    /// # Safety
    /// The point must be on the curve. If the point isn't on the curve, it will result in
    /// undefined behavior.
    pub const unsafe fn new_unchecked(x: FieldElement<C>, y: FieldElement<C>) -> Self {
        Self { x, y }
    }

    /// Multiplies a [`Point`] by a [`FieldElement`]
    pub fn mul_scalar(&self, scalar: FieldElement<C>) -> Self {
        todo!()
    }

    /// Adds `self` and `rhs`, returning the result.
    pub fn add(&self, rhs: &Self) -> Self {
        // TODO: use `assign` variants to avoid extra duplications
        let lambda = rhs.y.sub(&self.y).div(&rhs.x.sub(&self.x));
        let x = lambda.sqr().sub(&self.x).sub(&rhs.x);
        let y = lambda.mul(&self.x.sub(&rhs.x)).sub(&self.y);
        Self { x, y }
    }

    pub fn add_assign(&mut self, rhs: &Self) {
        *self = self.add(rhs);
    }

    pub fn double(&self) -> Self {
        todo!()
    }

    pub fn double_assign(&mut self) {
        *self = self.double();
    }

    pub fn neg(&self) -> Self {
        Self {
            x: self.x,
            y: self.y.neg(),
        }
    }

    pub fn neg_assign(&mut self) {
        self.y.neg_assign();
    }
}

impl<C: EllipticCurve> From<ProjectivePoint<C>> for AffinePoint<C> {
    fn from(value: ProjectivePoint<C>) -> Self {
        value.as_affine()
    }
}
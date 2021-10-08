// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! A private Jones matrix type exclusively for testing.

use marlu::{
    num_traits::{Float, Num},
    Complex, Jones,
};

#[derive(Clone, Copy, Default, PartialEq)]
pub(crate) struct TestJones<F: Float + Num>(Jones<F>);

impl<F: Float> From<Jones<F>> for TestJones<F> {
    #[inline]
    fn from(j: Jones<F>) -> Self {
        Self(j)
    }
}

impl<F: Float> std::ops::Deref for TestJones<F> {
    type Target = Jones<F>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<F: Float> std::ops::DerefMut for TestJones<F> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl std::fmt::Display for TestJones<f32> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "[[{:e}{:+e}j, {:e}{:+e}j] [{:e}{:+e}j, {:e}{:+e}j]]",
            self[0].re,
            self[0].im,
            self[1].re,
            self[1].im,
            self[2].re,
            self[2].im,
            self[3].re,
            self[3].im,
        )
    }
}

impl std::fmt::Display for TestJones<f64> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "[[{:e}{:+e}j, {:e}{:+e}j] [{:e}{:+e}j, {:e}{:+e}j]]",
            self[0].re,
            self[0].im,
            self[1].re,
            self[1].im,
            self[2].re,
            self[2].im,
            self[3].re,
            self[3].im,
        )
    }
}

impl std::fmt::Debug for TestJones<f32> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "[[{:e}{:+e}j, {:e}{:+e}j] [{:e}{:+e}j, {:e}{:+e}j]]",
            self[0].re,
            self[0].im,
            self[1].re,
            self[1].im,
            self[2].re,
            self[2].im,
            self[3].re,
            self[3].im,
        )
    }
}

impl std::fmt::Debug for TestJones<f64> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "[[{:e}{:+e}j, {:e}{:+e}j] [{:e}{:+e}j, {:e}{:+e}j]]",
            self[0].re,
            self[0].im,
            self[1].re,
            self[1].im,
            self[2].re,
            self[2].im,
            self[3].re,
            self[3].im,
        )
    }
}

impl<F: Float + approx::AbsDiffEq> approx::AbsDiffEq for TestJones<F>
where
    F::Epsilon: Clone,
{
    type Epsilon = F::Epsilon;

    #[inline]
    fn default_epsilon() -> F::Epsilon {
        F::default_epsilon()
    }

    #[inline]
    fn abs_diff_eq(&self, other: &Self, epsilon: F::Epsilon) -> bool {
        (0..4).all(|idx| Complex::<F>::abs_diff_eq(&self[idx], &other[idx], epsilon.clone()))
    }
}

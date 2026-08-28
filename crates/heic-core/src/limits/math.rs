//! Checked arithmetic utilities guarding against integer overflow.

use crate::error::{HeicError, HeicResult};

/// Performs checked multiplication of two `u64` integers, returning a [`HeicError::LimitExceeded`] on overflow.
#[inline]
pub fn checked_mul(a: u64, b: u64) -> HeicResult<u64> {
    a.checked_mul(b).ok_or_else(|| {
        HeicError::LimitExceeded(format!("Integer overflow in arithmetic: {a} * {b}"))
    })
}

/// Performs checked multiplication of two `usize` integers, returning a [`HeicError::LimitExceeded`] on overflow.
#[inline]
pub fn checked_mul_usize(a: usize, b: usize) -> HeicResult<usize> {
    a.checked_mul(b).ok_or_else(|| {
        HeicError::LimitExceeded(format!(
            "Integer overflow in buffer size calculation: {a} * {b}"
        ))
    })
}

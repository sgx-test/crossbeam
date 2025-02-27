#![no_std]
use std::prelude::v1::*;
#[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
use crate::primitive::sync::atomic::compiler_fence;
#[cfg(not(crossbeam_no_atomic))]
use core::sync::atomic::Ordering;

/// Trait which allows reading from primitive atomic types with "consume" ordering.
pub trait AtomicConsume {
    /// Type returned by `load_consume`.
    type Val;

    /// Loads a value from the atomic using a "consume" memory ordering.
    ///
    /// This is similar to the "acquire" ordering, except that an ordering is
    /// only guaranteed with operations that "depend on" the result of the load.
    /// However consume loads are usually much faster than acquire loads on
    /// architectures with a weak memory model since they don't require memory
    /// fence instructions.
    ///
    /// The exact definition of "depend on" is a bit vague, but it works as you
    /// would expect in practice since a lot of software, especially the Linux
    /// kernel, rely on this behavior.
    ///
    /// This is currently only implemented on ARM and AArch64, where a fence
    /// can be avoided. On other architectures this will fall back to a simple
    /// `load(Ordering::Acquire)`.
    fn load_consume(&self) -> Self::Val;
}

#[cfg(not(crossbeam_no_atomic))]
#[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
macro_rules! impl_consume {
    () => {
        #[inline]
        fn load_consume(&self) -> Self::Val {
            let result = self.load(Ordering::Relaxed);
            compiler_fence(Ordering::Acquire);
            result
        }
    };
}

#[cfg(not(crossbeam_no_atomic))]
#[cfg(not(any(target_arch = "arm", target_arch = "aarch64")))]
macro_rules! impl_consume {
    () => {
        #[inline]
        fn load_consume(&self) -> Self::Val {
            self.load(Ordering::Acquire)
        }
    };
}

macro_rules! impl_atomic {
    ($atomic:ident, $val:ty) => {
        #[cfg(not(crossbeam_no_atomic))]
        impl AtomicConsume for core::sync::atomic::$atomic {
            type Val = $val;
            impl_consume!();
        }
        #[cfg(crossbeam_loom)]
        impl AtomicConsume for loom::sync::atomic::$atomic {
            type Val = $val;
            impl_consume!();
        }
    };
}

impl_atomic!(AtomicBool, bool);
impl_atomic!(AtomicUsize, usize);
impl_atomic!(AtomicIsize, isize);
impl_atomic!(AtomicU8, u8);
impl_atomic!(AtomicI8, i8);
impl_atomic!(AtomicU16, u16);
impl_atomic!(AtomicI16, i16);
impl_atomic!(AtomicU32, u32);
impl_atomic!(AtomicI32, i32);
#[cfg(not(crossbeam_no_atomic_64))]
impl_atomic!(AtomicU64, u64);
#[cfg(not(crossbeam_no_atomic_64))]
impl_atomic!(AtomicI64, i64);

#[cfg(not(crossbeam_no_atomic))]
impl<T> AtomicConsume for core::sync::atomic::AtomicPtr<T> {
    type Val = *mut T;
    impl_consume!();
}

#[cfg(crossbeam_loom)]
impl<T> AtomicConsume for loom::sync::atomic::AtomicPtr<T> {
    type Val = *mut T;
    impl_consume!();
}

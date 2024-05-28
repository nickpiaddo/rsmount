// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library

// From standard library
use std::mem::MaybeUninit;

// From this library
use crate::core::errors::GenIteratorError;
use crate::core::iter::Direction;

/// A generic collection iterator.
#[derive(Debug)]
#[repr(transparent)]
pub struct GenIterator {
    pub(crate) inner: *mut libmount::libmnt_iter,
}

impl GenIterator {
    /// Creates a new `GenIterator` instance.
    pub fn new(direction: Direction) -> Result<GenIterator, GenIteratorError> {
        log::debug!("GenIterator::new creating a new `GenIterator` instance");

        let mut inner = MaybeUninit::<*mut libmount::libmnt_iter>::zeroed();

        unsafe {
            inner.write(libmount::mnt_new_iter(direction as i32));
        }

        match unsafe { inner.assume_init() } {
            inner if inner.is_null() => {
                let err_msg = "failed to create a new `GenIterator` instance".to_owned();
                log::debug!(
                    "GenIterator::new {}. libmount::mnt_new_iter returned a NULL pointer",
                    err_msg
                );

                Err(GenIteratorError::Creation(err_msg))
            }
            inner => {
                log::debug!("GenIterator::new created a new `GenIterator` instance");

                let iterator = Self { inner };

                Ok(iterator)
            }
        }
    }

    /// Returns the [`Direction`] of iteration.
    pub fn direction(&self) -> Direction {
        let code = unsafe { libmount::mnt_iter_get_direction(self.inner) };
        let direction = Direction::try_from(code).unwrap();

        log::debug!(
            "GenIterator::direction direction of iteration: {:?}",
            direction
        );

        direction
    }

    /// Resets the position of the next element in the collection to that of the
    /// first element. This method keeps the [`Direction`] of iteration unchanged.
    pub fn reset(&mut self) {
        log::debug!("GenIterator::reset resetting iterator with direction unchanged");
        const UNCHANGED_DIRECTION: libc::c_int = -1;

        unsafe { libmount::mnt_reset_iter(self.inner, UNCHANGED_DIRECTION) }
    }

    /// Resets the position of the next element in the collection to that of the
    /// first element, and sets the [`Direction`] of iteration to [`Direction::Forward`].
    pub fn reset_forward(&mut self) {
        log::debug!(
            "GenIterator::reset_forward resetting iterator, setting direction: {:?}",
            Direction::Forward
        );
        let direction = Direction::Forward;

        unsafe { libmount::mnt_reset_iter(self.inner, direction as i32) }
    }

    /// Resets the position of the next element in the collection to that of the
    /// first element, and sets the [`Direction`] of iteration to [`Direction::Backward`].
    pub fn reset_backward(&mut self) {
        log::debug!(
            "GenIterator::reset_backward resetting iterator, setting direction: {:?}",
            Direction::Backward
        );
        let direction = Direction::Backward;

        unsafe { libmount::mnt_reset_iter(self.inner, direction as i32) }
    }
}

impl AsRef<GenIterator> for GenIterator {
    #[inline]
    fn as_ref(&self) -> &GenIterator {
        self
    }
}

impl Drop for GenIterator {
    fn drop(&mut self) {
        log::debug!("GenIterator::drop deallocating `GenIterator` instance");

        unsafe { libmount::mnt_free_iter(self.inner) }
    }
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use super::*;
    use crate::core::iter::Direction;
    use pretty_assertions::{assert_eq, assert_ne};

    #[test]
    fn gen_iterator_can_create_a_forward_iterator() -> crate::Result<()> {
        let iterator = GenIterator::new(Direction::Forward)?;

        let actual = iterator.direction();
        let expected = Direction::Forward;
        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn gen_iterator_can_reset_backward_a_forward_iterator() -> crate::Result<()> {
        let mut iterator = GenIterator::new(Direction::Forward)?;

        let actual = iterator.direction();
        let expected = Direction::Forward;
        assert_eq!(actual, expected);

        iterator.reset_backward();

        let actual = iterator.direction();
        let expected = Direction::Backward;
        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn gen_iterator_can_reset_forward_a_backward_iterator() -> crate::Result<()> {
        let mut iterator = GenIterator::new(Direction::Backward)?;

        let actual = iterator.direction();
        let expected = Direction::Backward;
        assert_eq!(actual, expected);

        iterator.reset_forward();

        let actual = iterator.direction();
        let expected = Direction::Forward;
        assert_eq!(actual, expected);

        Ok(())
    }
}

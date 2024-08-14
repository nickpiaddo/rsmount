// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library

// From standard library

// From this library

#[allow(unused_macros)]
#[macro_export]
#[doc(hidden)]
macro_rules! gen_stepper {
    ($object:ident, $stepper_type:ident, $function:ident, $table_entry_type:ident) => {
        paste::paste! {
                    // From dependency library

                    // From standard library
                    use std::mem::MaybeUninit;

                    // From this library
                    use $crate::core::iter::{Direction, GenIterator};
                    use $crate::mount::[<$stepper_type IterError>];
                    use $crate::core::entries::$table_entry_type;
                    use $crate::mount::$object;
                    use $crate::mount::StepResult;

                    #[doc = concat!("Iterator to sequentially ", stringify!([<$stepper_type:lower>]), " entries in `/etc/fstab`, or `/proc/self/mountinfo`.")]
                    #[derive(Debug)]
                    pub struct [<$stepper_type Iter>]<'mount> {
                        mount: &'mount mut $object,
                        iterator: GenIterator,
                    }

                    impl<'mount> [<$stepper_type Iter>]<'mount> {
                        #[doc(hidden)]
                        #[doc = concat!("Creates a new `", stringify!($stepper_type), "Iter`.")]
                        #[allow(dead_code)]
                        pub(crate) fn new(
                            mount: &'mount mut $object,
                        ) -> Result<[<$stepper_type Iter>]<'mount>, [<$stepper_type IterError>]> {
                            let iterator = GenIterator::new(Direction::Forward)?;

                            let iterator = Self {
                                mount,
                                iterator,
                            };

                            Ok(iterator)
                        }
                    }

                    impl<'mount> Iterator for [<$stepper_type Iter>]<'mount> {
                        type Item = StepResult;

                        #[doc = concat!("Tries to ", stringify!([<$stepper_type:lower>]), " an entry in `/etc/fstab`, or `/proc/self/mountinfo`.")]
                        ///
                        /// Returns the function's status after execution as a [`StepResult`], or
                        /// `None` if there is no more entry to process or an error occurred.
                        fn next(&mut self) -> Option<Self::Item> {
                            log::debug!(concat!(stringify!($stepper_type), "Iter::next trying to ", stringify!([<$stepper_type>]), " next `", stringify!($table_entry_type), "`"));

                            let mut entry_ptr = MaybeUninit::<*mut libmount::libmnt_fs>::zeroed();
                            let mut mount_return_code = MaybeUninit::<libc::c_int>::zeroed();
                            let mut ignored = MaybeUninit::<libc::c_int>::zeroed();

                            let result = unsafe {  libmount::$function(
                                self.mount.inner,
                                self.iterator.inner,
                                entry_ptr.as_mut_ptr(),
                                mount_return_code.as_mut_ptr(),
                                ignored.as_mut_ptr(),
                            )};

                            match result {
                                0 => {
                                    let ptr = unsafe { entry_ptr.assume_init() };
                                    let rc = unsafe { mount_return_code.assume_init() };
                                    let skipped = unsafe { ignored.assume_init() };
                                    let entry = <$table_entry_type>::from_ptr(ptr);

                                    match (rc, skipped) {
                                        (0, 0) => Some(StepResult::[<$stepper_type Success>](entry)),
                                        (_, 1) => Some(StepResult::[<$stepper_type Skipped>](entry)),
                                        (_, 2) => Some(StepResult::[<$stepper_type AlreadyDone>](entry)),
                                        (_, _) => Some(StepResult::[<$stepper_type Fail>](entry)),
                                    }
                                }
                                1 => {
                                    log::debug!(concat!(stringify!($stepper_type), "Iter::next reached the end of `", stringify!($stepper_type), "Iter`"));

                                    None
                                }
                                code => {
                                    log::debug!(concat!(stringify!($stepper_type), "Iter::next failed to", stringify!([<$stepper_type:lower>]), " next entry. ", stringify!($function), " returned error code: {:?}"), code);

                                    None
                                }
                            }
                        }
                    }
                } //---- END paste
    };
}

// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

#[allow(unused_macros)]
#[macro_export]
#[doc(hidden)]
macro_rules! owning_ref_from_ptr {
    ($object_ref:expr, $output_ref_ty:ident, $ptr: ident) => {{
        let mut obj_ptr = std::ptr::NonNull::from($object_ref.as_ref());
        let boxed = Box::new($ptr);
        let (boxed_ptr, item) = unsafe { <$output_ref_ty>::ref_from_boxed_ptr(boxed) };

        // Adds boxed pointer to garbage collector
        unsafe { obj_ptr.as_mut().gc.push(boxed_ptr.into()) };

        item
    }};
}

#[allow(unused_macros)]
#[macro_export]
#[doc(hidden)]
macro_rules! owning_mut_from_ptr {
    ($object_ref:expr, $output_ref_ty:ident, $ptr: ident) => {{
        let mut obj_ptr = std::ptr::NonNull::from($object_ref.as_ref());
        let boxed = Box::new($ptr);
        let (boxed_ptr, item) = unsafe { <$output_ref_ty>::mut_from_boxed_ptr(boxed) };

        // Adds boxed pointer to garbage collector
        unsafe { obj_ptr.as_mut().gc.push(boxed_ptr.into()) };

        item
    }};
}

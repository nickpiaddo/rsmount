// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library

// From standard library
use std::mem::MaybeUninit;
use std::os::fd::BorrowedFd;
use std::path::Path;

// From this library
use crate::core::errors::TableMonitorError;
use crate::core::fs::{FileChange, MonitorKind, MonitorStatus};
use crate::ffi_utils;

/// Mount table monitor.
#[derive(Debug)]
#[repr(transparent)]
pub struct TableMonitor {
    pub(crate) inner: *mut libmount::libmnt_monitor,
}

impl TableMonitor {
    #[doc(hidden)]
    /// Increments the `TableMonitor`'s reference counter.
    #[allow(dead_code)]
    pub(crate) fn incr_ref_counter(&mut self) {
        unsafe { libmount::mnt_ref_monitor(self.inner) }
    }

    #[doc(hidden)]
    /// Decrements the `TableMonitor`'s reference counter.
    #[allow(dead_code)]
    pub(crate) fn decr_ref_counter(&mut self) {
        unsafe { libmount::mnt_unref_monitor(self.inner) }
    }

    /// Creates a new `TableMonitor`.
    pub fn new() -> Result<TableMonitor, TableMonitorError> {
        log::debug!("TableMonitor::new creating a new `TableMonitor` instance");

        let mut inner = MaybeUninit::<*mut libmount::libmnt_monitor>::zeroed();

        unsafe {
            inner.write(libmount::mnt_new_monitor());
        }

        match unsafe { inner.assume_init() } {
            inner if inner.is_null() => {
                let err_msg = "failed to create a new `TableMonitor` instance".to_owned();
                log::debug!(
                    "TableMonitor::new {}. libmount::mnt_new_monitor returned a NULL pointer",
                    err_msg
                );

                Err(TableMonitorError::Creation(err_msg))
            }
            inner => {
                log::debug!("TableMonitor::new created a new `TableMonitor` instance");
                let monitor = Self { inner };

                Ok(monitor)
            }
        }
    }

    #[doc(hidden)]
    /// Enables/disables userspace mount table monitoring.
    fn set_enable_user_space_monitor(
        monitor: *mut libmount::libmnt_monitor,
        file: *const libc::c_char,
        enable: bool,
    ) -> Result<(), TableMonitorError> {
        let op = if enable { 1 } else { 0 };
        let op_str = if enable {
            "enable".to_owned()
        } else {
            "disable".to_owned()
        };

        let result = unsafe { libmount::mnt_monitor_enable_userspace(monitor, op, file) };

        match result {
            0 => {
                log::debug!(
                    "TableMonitor::set_enable_user_space_monitor {}d userspace mount table monitor",
                    op_str
                );

                Ok(())
            }
            code => {
                let err_msg = format!("failed to {} userspace mount table monitor", op_str);
                log::debug!("TableMonitor::set_enable_user_space_monitor {}. libmount::mnt_monitor_enable_userspace returned error code: {:?}", err_msg, code);

                Err(TableMonitorError::Config(err_msg))
            }
        }
    }

    /// Enables user space mount table monitoring.
    ///
    /// **Note:** user space mount table monitoring is unsupported for systems with a legacy, regular
    /// `/etc/mtab` file.
    pub fn watch_user_space(&mut self) -> Result<(), TableMonitorError> {
        log::debug!("TableMonitor::watch_user_space enabling userspace mount table monitoring");

        Self::set_enable_user_space_monitor(self.inner, std::ptr::null(), true)
    }

    /// Enables user space mount table monitoring on `file`.
    ///
    /// **Note:** once the `file` to monitor is set, it can not be changed. Furthermore, you can
    /// only monitor **one** file in user space at a time.
    pub fn watch_file<T>(&mut self, file: T) -> Result<(), TableMonitorError>
    where
        T: AsRef<Path>,
    {
        let file = file.as_ref();
        let file_cstr = ffi_utils::as_ref_path_to_c_string(file)?;
        log::debug!(
            "TableMonitor::watch_file enabling userspace mount table monitoring on: {:?}",
            file
        );

        Self::set_enable_user_space_monitor(self.inner, file_cstr.as_ptr(), true)
    }

    /// Disables user space mount table monitoring.
    ///
    /// **Note:** user space mount table monitoring is unsupported for systems with a legacy, regular
    /// `/etc/mtab` file.
    pub fn unwatch_user_space(&mut self) -> Result<(), TableMonitorError> {
        log::debug!("TableMonitor::unwatch_user_space disabling userspace mount table monitoring");

        Self::set_enable_user_space_monitor(self.inner, std::ptr::null(), false)
    }

    #[doc(hidden)]
    /// Enables/disables kernel VFS monitoring.
    fn set_enable_kernel_vfs_monitor(
        monitor: *mut libmount::libmnt_monitor,
        enable: bool,
    ) -> Result<(), TableMonitorError> {
        let op = if enable { 1 } else { 0 };
        let op_str = if enable {
            "enable".to_owned()
        } else {
            "disable".to_owned()
        };

        let result = unsafe { libmount::mnt_monitor_enable_kernel(monitor, op) };

        match result {
            0 => {
                log::debug!(
                    "TableMonitor::set_enable_kernel_vfs_monitor {}d kernel VFS monitor",
                    op_str
                );

                Ok(())
            }
            code => {
                let err_msg = format!("failed to {} kernel VFS monitor", op_str);
                log::debug!("TableMonitor::set_enable_kernel_vfs_monitor {}. libmount::mnt_monitor_enable_kernel returned error code: {:?}", err_msg, code);

                Err(TableMonitorError::Config(err_msg))
            }
        }
    }

    /// Enables kernel VFS monitoring.
    pub fn watch_kernel(&mut self) -> Result<(), TableMonitorError> {
        log::debug!("TableMonitor::watch_kernel enabling kernel VFS monitoring");

        Self::set_enable_kernel_vfs_monitor(self.inner, true)
    }

    /// Disables kernel VFS monitoring.
    pub fn unwatch_kernel(&mut self) -> Result<(), TableMonitorError> {
        log::debug!("TableMonitor::unwatch_kernel disabling kernel VFS monitoring");

        Self::set_enable_kernel_vfs_monitor(self.inner, false)
    }

    /// Returns a file descriptor using the
    /// [`epoll`](https://www.man7.org/linux/man-pages/man7/epoll.7.html) I/O event notification
    /// facility to monitor multiple mount table files. Note that after each notification, you must call
    /// either [`TableMonitor::discard_last_event`] or [`TableMonitor::next_file_change`].
    pub fn event_notifier_create(&mut self) -> Result<BorrowedFd, TableMonitorError> {
        log::debug!("TableMonitor::event_notifier_create creating a file event notifier");

        let result = unsafe { libmount::mnt_monitor_get_fd(self.inner) };

        match result {
            code if code < 0 => {
                let err_msg = "failed to create file event notifier".to_owned();
                log::debug!("TableMonitor::event_notifier_create {}. libmount::mnt_monitor_get_fd returned error code: {:?}", err_msg, code);

                Err(TableMonitorError::Event(err_msg))
            }
            fd => {
                log::debug!("TableMonitor::event_notifier_create created a file event notifier");
                let file_descriptor = unsafe { BorrowedFd::borrow_raw(fd) };

                Ok(file_descriptor)
            }
        }
    }

    /// Closes the file descriptor returned by [`TableMonitor::event_notifier_create`].
    pub fn event_notifier_delete(&mut self) -> Result<(), TableMonitorError> {
        log::debug!("TableMonitor::event_notifier_delete destroying a file event notifier");

        let result = unsafe { libmount::mnt_monitor_close_fd(self.inner) };

        match result {
            0 => {
                log::debug!("TableMonitor::event_notifier_delete destroyed file event notifier");

                Ok(())
            }
            code => {
                let err_msg = "failed to delete the file event notifier".to_owned();
                log::debug!("TableMonitor::event_notifier_delete {}. libmount::mnt_monitor_close_fd returned error code: {:?}", err_msg, code);

                Err(TableMonitorError::Event(err_msg))
            }
        }
    }

    /// s for the next mount table file changes. If a change is detected, use
    /// [`TableMonitor::next_file_change`] to get additional details about any modification.
    pub fn wait_for_next_change(&mut self, time_out: i32) -> MonitorStatus {
        log::debug!("TableMonitor::wait_for_next_change waiting for the next file change");

        let result = unsafe { libmount::mnt_monitor_wait(self.inner, time_out) };

        match result {
            0 => {
                log::debug!("TableMonitor::wait_for_next_change time out");

                MonitorStatus::TimeOut
            }
            1 => {
                log::debug!("TableMonitor::wait_for_next_change change detected");

                MonitorStatus::ChangeDetected
            }
            code => {
                let err_msg = "failed to wait for the next file change".to_owned();
                log::debug!("TableMonitor::wait_for_next_change {}. libmount::mnt_monitor_wait returned error code: {:?}", err_msg, code);

                MonitorStatus::Error
            }
        }
    }

    /// Returns details about mount table file changes, or `None` if no change occurred. This
    /// function does not wait for a notification to give details about the last recorded file change.
    pub fn next_file_change(&mut self) -> Result<Option<FileChange>, TableMonitorError> {
        log::debug!("TableMonitor::next_file_change getting next file change");

        let mut file = MaybeUninit::<*const libc::c_char>::zeroed();
        let mut kind = MaybeUninit::<libc::c_int>::zeroed();

        let result = unsafe {
            libmount::mnt_monitor_next_change(self.inner, file.as_mut_ptr(), kind.as_mut_ptr())
        };

        match result {
            0 => {
                log::debug!("TableMonitor::next_file_change got next file change");
                let file = unsafe { file.assume_init() };
                let kind = unsafe { kind.assume_init() };

                let file_name = ffi_utils::const_c_char_array_to_path_buf(file);
                let monitor_kind = MonitorKind::try_from(kind as u32).ok();
                let file_change = FileChange::new(file_name, monitor_kind);

                Ok(Some(file_change))
            }
            1 => {
                log::debug!("TableMonitor::next_file_change no file change");

                Ok(None)
            }
            code => {
                let err_msg = "failed to get next file change".to_owned();
                log::debug!("TableMonitor::next_file_change {}. libmount::mnt_monitor_next_change returned error code: {:?}", err_msg, code);

                Err(TableMonitorError::Event(err_msg))
            }
        }
    }

    /// Clears this `TableMonitor`'s event buffer. Unless you called the
    /// [`TableMonitor::next_file_change`] method, which will automatically handle clean-up
    /// operations, you MUST call this function after an event occurs to be able to receive the
    /// next one.
    pub fn discard_last_event(&mut self) -> Result<(), TableMonitorError> {
        log::debug!("TableMonitor::discard_last_event discarding event");

        let result = unsafe { libmount::mnt_monitor_event_cleanup(self.inner) };

        match result {
            0 => {
                log::debug!("TableMonitor::discard_last_event discarded last event");

                Ok(())
            }
            code => {
                let err_msg = "failed to discard last event".to_owned();
                log::debug!("TableMonitor::discard_last_event {}. libmount::mnt_monitor_event_cleanup returned error code: {:?}", err_msg, code);

                Err(TableMonitorError::Event(err_msg))
            }
        }
    }
}

impl Drop for TableMonitor {
    fn drop(&mut self) {
        log::debug!("TableMonitor::drop deallocating `TableMonitor` instance");

        unsafe { libmount::mnt_unref_monitor(self.inner) }
    }
}

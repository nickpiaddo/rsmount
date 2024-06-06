// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library

// From standard library
use std::mem::MaybeUninit;
use std::path::Path;

// From this library
use crate::core::errors::FileLockError;
use crate::ffi_utils;

/// File lock.
#[derive(Debug)]
#[repr(transparent)]
pub struct FileLock {
    pub(crate) ptr: *mut libmount::libmnt_lock,
}

impl FileLock {
    #[doc(hidden)]
    #[allow(dead_code)]
    /// Wraps a boxed raw `libmount::mnt_lock` pointer in a safe reference.
    pub(crate) unsafe fn ref_from_boxed_ptr<'a>(
        ptr: Box<*mut libmount::libmnt_lock>,
    ) -> (*mut *mut libmount::libmnt_lock, &'a FileLock) {
        let raw_ptr = Box::into_raw(ptr);
        let entry_ref = unsafe { &*(raw_ptr as *const _ as *const FileLock) };

        (raw_ptr, entry_ref)
    }

    #[doc(hidden)]
    #[allow(dead_code)]
    /// Wraps a boxed raw `libmount::mnt_lock` pointer in a safe mutable reference.
    pub(crate) unsafe fn mut_from_boxed_ptr<'a>(
        ptr: Box<*mut libmount::libmnt_lock>,
    ) -> (*mut *mut libmount::libmnt_lock, &'a mut FileLock) {
        let raw_ptr = Box::into_raw(ptr);
        let entry_ref = unsafe { &mut *(raw_ptr as *mut FileLock) };

        (raw_ptr, entry_ref)
    }

    /// Creates a new `FileLock`.
    pub fn new<T>(file: T) -> Result<FileLock, FileLockError>
    where
        T: AsRef<Path>,
    {
        let file = file.as_ref();
        let file_cstr = ffi_utils::as_ref_path_to_c_string(file)?;
        log::debug!(
            "FileLock::new creating a new `FileLock` for file {:?}",
            file
        );

        // The second argument of mnt_new_lock should hold a `pid_t` value but is ignored by
        // `libmount`. Putting i32::MIN should trigger an error if `libmount`'s API changes in
        // future versions.
        let mut ptr = MaybeUninit::<*mut libmount::libmnt_lock>::zeroed();
        unsafe { ptr.write(libmount::mnt_new_lock(file_cstr.as_ptr(), i32::MIN)) };

        match unsafe { ptr.assume_init() } {
            ptr if ptr.is_null() => {
                let err_msg = format!("failed to create a new `FileLock` for file {:?}", file);
                log::debug!(
                    "FileLock::new {}. libmount::mnt_new_lock returned a NULL pointer",
                    err_msg
                );

                Err(FileLockError::Creation(err_msg))
            }
            ptr => {
                log::debug!("FileLock::new created a new `FileLock` for file {:?}", file);
                let lock = Self { ptr };

                Ok(lock)
            }
        }
    }

    /// Locks the associated file.
    pub fn lock(&mut self) -> Result<(), FileLockError> {
        log::debug!("FileLock::lock locking file");

        let result = unsafe { libmount::mnt_lock_file(self.ptr) };

        match result {
            0 => {
                log::debug!("FileLock::lock locked file");

                Ok(())
            }
            code => {
                let err_msg = "failed to lock file".to_owned();
                log::debug!(
                    "FileLock::lock {}. libmount::mnt_lock_file returned error code: {:?}",
                    err_msg,
                    code
                );

                Err(FileLockError::Lock(err_msg))
            }
        }
    }

    /// Releases the lock on the associated file.
    pub fn unlock(&mut self) {
        log::debug!("FileLock::unlock releasing file lock");

        unsafe { libmount::mnt_unlock_file(self.ptr) }
    }

    #[doc(hidden)]
    /// Blocks/Unblocks signals.
    fn set_signals(lock: *mut libmount::libmnt_lock, enable: bool) -> Result<(), FileLockError> {
        let op = if enable { 1 } else { 0 };
        let op_str = if enable {
            "enable".to_owned()
        } else {
            "disable".to_owned()
        };

        let result = unsafe { libmount::mnt_lock_block_signals(lock, op) };

        match result {
            0 => {
                log::debug!("FileLock::set_signals {}d signals", op_str);

                Ok(())
            }
            code => {
                let err_msg = format!("failed to {} signals", op_str);
                log::debug!("FileLock::set_signals {}. libmount::mnt_lock_block_signals returned error code: {:?}", err_msg, code);

                Err(FileLockError::Config(err_msg))
            }
        }
    }

    // List from https://www.man7.org/linux/man-pages/man7/signal.7.html#DESCRIPTION
    /// Blocks POSIX signals.
    ///
    /// Linux supports the standard signals listed below.  The second column of the table indicates
    /// which standard (if any) specified the signal: `P1990` indicates that the signal is described in
    /// the original POSIX.1-1990 standard; `P2001` indicates that the signal was added in SUSv2 and
    /// POSIX.1-2001.
    ///
    /// | Signal    | Standard | Action | Comment                                                                                                           |
    /// | -----     | ------   | ----   | ----                                                                                                              |
    /// | SIGABRT   | P1990    | Core   | Abort signal from [abort(3)](https://www.man7.org/linux/man-pages/man3/abort.3.html)                              |
    /// | SIGALRM   | P1990    | Term   | Timer signal from [alarm(2)](https://www.man7.org/linux/man-pages/man2/alarm.2.html)                              |
    /// | SIGBUS    | P2001    | Core   | Bus error (bad memory access)                                                                                     |
    /// | SIGCHLD   | P1990    | Ign    | Child stopped or terminated                                                                                       |
    /// | SIGCLD    | -        | Ign    | A synonym for SIGCHLD                                                                                             |
    /// | SIGCONT   | P1990    | Cont   | Continue if stopped                                                                                               |
    /// | SIGEMT    | -        | Term   | Emulator trap                                                                                                     |
    /// | SIGFPE    | P1990    | Core   | Floating-point exception                                                                                          |
    /// | SIGHUP    | P1990    | Term   | Hangup detected on controlling terminal or death of controlling process                                           |
    /// | SIGILL    | P1990    | Core   | Illegal Instruction                                                                                               |
    /// | SIGINFO   | -        |        | A synonym for SIGPWR                                                                                              |
    /// | SIGINT    | P1990    | Term   | Interrupt from keyboard                                                                                           |
    /// | SIGIO     | -        | Term   | I/O now possible (4.2BSD)                                                                                         |
    /// | SIGIOT    | -        | Core   | IOT trap. A synonym for SIGABRT                                                                                   |
    /// | SIGKILL   | P1990    | Term   | Kill signal                                                                                                       |
    /// | SIGLOST   | -        | Term   | File lock lost (unused)                                                                                           |
    /// | SIGPIPE   | P1990    | Term   | Broken pipe: write to pipe with no readers; see [pipe(7)](https://www.man7.org/linux/man-pages/man7/pipe.7.html)  |
    /// | SIGPOLL   | P2001    | Term   | Pollable event (Sys V); synonym for SIGIO                                                                         |
    /// | SIGPROF   | P2001    | Term   | Profiling timer expired                                                                                           |
    /// | SIGPWR    | -        | Term   | Power failure (System V)                                                                                          |
    /// | SIGQUIT   | P1990    | Core   | Quit from keyboard                                                                                                |
    /// | SIGSEGV   | P1990    | Core   | Invalid memory reference                                                                                          |
    /// | SIGSTKFLT | -        | Term   | Stack fault on coprocessor (unused)                                                                               |
    /// | SIGSTOP   | P1990    | Stop   | Stop process                                                                                                      |
    /// | SIGTSTP   | P1990    | Stop   | Stop typed at terminal                                                                                            |
    /// | SIGSYS    | P2001    | Core   | Bad system call (SVr4); see also [seccomp(2)](https://www.man7.org/linux/man-pages/man2/seccomp.2.html)           |
    /// | SIGTERM   | P1990    | Term   | Termination signal                                                                                                |
    /// | SIGTRAP   | P2001    | Core   | Trace/breakpoint trap                                                                                             |
    /// | SIGTTIN   | P1990    | Stop   | Terminal input for background process                                                                             |
    /// | SIGTTOU   | P1990    | Stop   | Terminal output for background process                                                                            |
    /// | SIGUNUSED | -        | Core   | Synonymous with SIGSYS                                                                                            |
    /// | SIGURG    | P2001    | Ign    | Urgent condition on socket (4.2BSD)                                                                               |
    /// | SIGUSR1   | P1990    | Term   | User-defined signal 1                                                                                             |
    /// | SIGUSR2   | P1990    | Term   | User-defined signal 2                                                                                             |
    /// | SIGVTALRM | P2001    | Term   | Virtual alarm clock (4.2BSD)                                                                                      |
    /// | SIGXCPU   | P2001    | Core   | CPU time limit exceeded (4.2BSD); see [setrlimit(2)](https://www.man7.org/linux/man-pages/man2/setrlimit.2.html)  |
    /// | SIGXFSZ   | P2001    | Core   | File size limit exceeded (4.2BSD); see [setrlimit(2)](https://www.man7.org/linux/man-pages/man2/setrlimit.2.html) |
    /// | SIGWINCH  | -        | Ign    | Window resize signal (4.3BSD, Sun)                                                                                |
    ///
    /// The signals `SIGKILL` and `SIGSTOP` cannot be caught, blocked, or
    /// ignored.
    ///
    /// Source: <cite>[Signal - Overview of signals](https://www.man7.org/linux/man-pages/man7/signal.7.html), §Standard signals.</cite>
    pub fn block_signals(&mut self) -> Result<(), FileLockError> {
        log::debug!("FileLock::block_signals blocking signals");

        Self::set_signals(self.ptr, true)
    }

    /// Unblocks signals.
    pub fn unblock_signals(&mut self) -> Result<(), FileLockError> {
        log::debug!("FileLock::unblock_signals unblocking signals");

        Self::set_signals(self.ptr, false)
    }
}

impl AsRef<FileLock> for FileLock {
    #[inline]
    fn as_ref(&self) -> &FileLock {
        self
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        log::debug!("FileLock::drop deallocating `FileLock` instance");
        self.unlock();

        unsafe { libmount::mnt_free_lock(self.ptr) }
    }
}

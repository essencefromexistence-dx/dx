//! Small Windows `RUSTC_WRAPPER` used by the local Dx build recipes.
//!
//! Cargo can keep twelve dependency jobs scheduled while this wrapper limits
//! the number of memory-heavy rustc processes. The named semaphore is shared
//! by all wrapper instances, including those spawned by Cargo in parallel.

use std::{env, process::Command};

#[cfg(windows)]
mod windows_gate {
    use std::ffi::c_void;

    type Handle = *mut c_void;

    const INFINITE: u32 = u32::MAX;
    const WAIT_OBJECT_0: u32 = 0;

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn CloseHandle(handle: Handle) -> i32;
        fn CreateSemaphoreW(
            attributes: *const c_void,
            initial_count: i32,
            maximum_count: i32,
            name: *const u16,
        ) -> Handle;
        fn ReleaseSemaphore(handle: Handle, release_count: i32, previous_count: *mut i32) -> i32;
        fn WaitForSingleObject(handle: Handle, milliseconds: u32) -> u32;
    }

    pub struct Semaphore(Handle);

    impl Semaphore {
        pub fn acquire(max_concurrency: u32) -> Option<Self> {
            let name = format!("Local\\DxDesktopRustcMemoryGate-v1-{max_concurrency}");
            let mut wide_name = name.encode_utf16().collect::<Vec<_>>();
            wide_name.push(0);
            let maximum_count = i32::try_from(max_concurrency).ok()?;

            // SAFETY: the string is NUL-terminated and remains alive for the
            // duration of the call; null attributes request default security.
            let handle = unsafe {
                CreateSemaphoreW(
                    std::ptr::null(),
                    maximum_count,
                    maximum_count,
                    wide_name.as_ptr(),
                )
            };
            if handle.is_null() {
                return None;
            }

            // SAFETY: the handle was returned by CreateSemaphoreW.
            let wait_result = unsafe { WaitForSingleObject(handle, INFINITE) };
            if wait_result != WAIT_OBJECT_0 {
                // SAFETY: the handle is valid and owned by this process.
                unsafe { CloseHandle(handle) };
                return None;
            }

            Some(Self(handle))
        }
    }

    impl Drop for Semaphore {
        fn drop(&mut self) {
            // SAFETY: this process owns the acquired semaphore handle.
            unsafe {
                ReleaseSemaphore(self.0, 1, std::ptr::null_mut());
                CloseHandle(self.0);
            }
        }
    }
}

fn main() {
    let mut args = env::args_os();
    let _wrapper = args.next();
    let Some(rustc) = args.next() else {
        eprintln!("rustc-memory-wrapper: missing rustc executable");
        std::process::exit(2);
    };

    let max_concurrency = env::var("DX_RUSTC_MAX_CONCURRENCY")
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(3);

    #[cfg(windows)]
    let _gate = windows_gate::Semaphore::acquire(max_concurrency);

    let status = match Command::new(rustc).args(args).status() {
        Ok(status) => status,
        Err(error) => {
            eprintln!("rustc-memory-wrapper: failed to start rustc: {error}");
            #[cfg(windows)]
            drop(_gate);
            std::process::exit(1);
        }
    };

    let code = status.code().unwrap_or(1);
    // `std::process::exit` skips destructors, so release the permit before
    // returning the compiler's exit code.
    #[cfg(windows)]
    drop(_gate);
    std::process::exit(code);
}

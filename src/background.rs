//! Screenshot and test runs must not steal the focus. On macOS, miniquad
//! activates the app on its first update; debug runs remember which app was
//! in front at launch and hand the focus back to it whenever this one takes
//! it during the first seconds.
#[cfg(target_os = "macos")]
mod mac {
    use std::ffi::{c_char, c_void, CString};
    use std::sync::atomic::{AtomicPtr, Ordering};
    type Id = *mut c_void;
    #[link(name = "objc")]
    extern "C" {
        fn objc_getClass(name: *const c_char) -> Id;
        fn sel_registerName(name: *const c_char) -> Id;
        fn objc_msgSend();
    }
    #[link(name = "AppKit", kind = "framework")]
    extern "C" {}

    static PREVIOUS: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

    fn class(n: &str) -> Id {
        unsafe { objc_getClass(CString::new(n).unwrap().as_ptr()) }
    }
    fn sel(n: &str) -> Id {
        unsafe { sel_registerName(CString::new(n).unwrap().as_ptr()) }
    }
    fn send(obj: Id, s: &str) -> Id {
        let f: extern "C" fn(Id, Id) -> Id = unsafe { std::mem::transmute(objc_msgSend as *const ()) };
        f(obj, sel(s))
    }
    fn send_u64(obj: Id, s: &str, arg: u64) -> Id {
        let f: extern "C" fn(Id, Id, u64) -> Id = unsafe { std::mem::transmute(objc_msgSend as *const ()) };
        f(obj, sel(s), arg)
    }

    pub fn remember_front() {
        let workspace = send(class("NSWorkspace"), "sharedWorkspace");
        let front = send(workspace, "frontmostApplication");
        let me = send(class("NSRunningApplication"), "currentApplication");
        if !front.is_null() && front != me {
            send(front, "retain");
            PREVIOUS.store(front, Ordering::Relaxed);
        }
        let app = send(class("NSApplication"), "sharedApplication");
        // Accessory: no Dock icon.
        send_u64(app, "setActivationPolicy:", 1);
    }

    pub fn give_back_focus() {
        let previous = PREVIOUS.load(Ordering::Relaxed);
        if previous.is_null() {
            return;
        }
        let app = send(class("NSApplication"), "sharedApplication");
        let active: extern "C" fn(Id, Id) -> bool = unsafe { std::mem::transmute(objc_msgSend as *const ()) };
        if active(app, sel("isActive")) {
            // NSApplicationActivateIgnoringOtherApps.
            send_u64(previous, "activateWithOptions:", 2);
            // And keep the window behind the others.
            let windows = send(app, "windows");
            let window = send_u64(windows, "objectAtIndex:", 0);
            if !window.is_null() {
                send_u64(window, "orderBack:", 0);
            }
        }
    }
}

/// Call once at launch in debug runs.
pub fn stay_in_background() {
    #[cfg(target_os = "macos")]
    mac::remember_front();
}

/// Call every frame in debug runs: returns the focus if the window took it.
pub fn keep_focus_away() {
    #[cfg(target_os = "macos")]
    mac::give_back_focus();
}

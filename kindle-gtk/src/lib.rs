//! Minimal safe GTK+ 2 bindings for Kindle applications.
//!
//! Raw GTK and GLib declarations are kept private. The public API exposes typed
//! widget handles, closure-based signal handlers, and a channel for delivering
//! worker-thread messages on the GTK main thread.

#![deny(unsafe_op_in_unsafe_fn)]

mod channel;
mod ffi;
mod signal;
mod util;
mod widget;

use std::ptr;

pub use channel::{ui_channel, UiSender, UiSource};
#[doc(hidden)]
pub use widget::Widget;
pub use widget::{
    BoxLayout, Button, Entry, Frame, Label, Notebook, ProgressBar, ScrolledWindow, WidgetExt,
    Window,
};

pub fn init() {
    let mut argc = 0;
    let mut argv = ptr::null_mut();
    unsafe { ffi::gtk_init(&mut argc, &mut argv) }
}

pub fn run() {
    unsafe { ffi::gtk_main() }
}

pub fn quit() {
    unsafe { ffi::gtk_main_quit() }
}

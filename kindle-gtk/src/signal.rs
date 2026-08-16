use std::ffi::{c_int, c_uint, c_void};
use std::mem;

use crate::ffi::{self, GCallback, GClosure, GtkWidget, FALSE, G_CONNECT_AFTER};
use crate::util::{abort_on_panic, c_string};
use crate::widget::Widget;

type SignalCallback = unsafe extern "C" fn(*mut GtkWidget, *mut c_void);
type EventSignalCallback = unsafe extern "C" fn(*mut GtkWidget, *mut c_void, *mut c_void) -> c_int;
type SwitchPageCallback = unsafe extern "C" fn(*mut GtkWidget, *mut GtkWidget, c_uint, *mut c_void);

struct SignalData {
    callback: Box<dyn Fn()>,
}

struct SwitchPageData {
    callback: Box<dyn Fn(u32)>,
}

pub(crate) fn connect<F: Fn() + 'static>(widget: Widget, signal: &str, callback: F) {
    let data = Box::new(SignalData {
        callback: Box::new(callback),
    });
    unsafe {
        ffi::g_signal_connect_data(
            widget.pointer(),
            c_string(signal).as_ptr(),
            Some(mem::transmute::<SignalCallback, GCallback>(
                signal_trampoline,
            )),
            Box::into_raw(data).cast(),
            Some(drop_signal_data),
            0,
        );
    }
}

pub(crate) fn connect_event<F: Fn() + 'static>(widget: Widget, signal: &str, callback: F) {
    let data = Box::new(SignalData {
        callback: Box::new(callback),
    });
    unsafe {
        ffi::g_signal_connect_data(
            widget.pointer(),
            c_string(signal).as_ptr(),
            Some(mem::transmute::<EventSignalCallback, GCallback>(
                event_signal_trampoline,
            )),
            Box::into_raw(data).cast(),
            Some(drop_signal_data),
            0,
        );
    }
}

pub(crate) fn connect_switch_page<F: Fn(u32) + 'static>(widget: Widget, callback: F) {
    let data = Box::new(SwitchPageData {
        callback: Box::new(callback),
    });
    unsafe {
        ffi::g_signal_connect_data(
            widget.pointer(),
            c_string("switch-page").as_ptr(),
            Some(mem::transmute::<SwitchPageCallback, GCallback>(
                switch_page_trampoline,
            )),
            Box::into_raw(data).cast(),
            Some(drop_switch_page_data),
            G_CONNECT_AFTER,
        );
    }
}

unsafe extern "C" fn signal_trampoline(_widget: *mut GtkWidget, data: *mut c_void) {
    let data = unsafe { &*data.cast::<SignalData>() };
    abort_on_panic(|| (data.callback)());
}

unsafe extern "C" fn event_signal_trampoline(
    _widget: *mut GtkWidget,
    _event: *mut c_void,
    data: *mut c_void,
) -> c_int {
    let data = unsafe { &*data.cast::<SignalData>() };
    abort_on_panic(|| (data.callback)());
    FALSE
}

unsafe extern "C" fn switch_page_trampoline(
    _notebook: *mut GtkWidget,
    _page: *mut GtkWidget,
    page_number: c_uint,
    data: *mut c_void,
) {
    let data = unsafe { &*data.cast::<SwitchPageData>() };
    abort_on_panic(|| (data.callback)(page_number));
}

unsafe extern "C" fn drop_signal_data(data: *mut c_void, _closure: *mut GClosure) {
    drop(unsafe { Box::from_raw(data.cast::<SignalData>()) });
}

unsafe extern "C" fn drop_switch_page_data(data: *mut c_void, _closure: *mut GClosure) {
    drop(unsafe { Box::from_raw(data.cast::<SwitchPageData>()) });
}

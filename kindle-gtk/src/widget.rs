use std::ffi::{c_int, c_void, CStr};
use std::marker::PhantomData;
use std::ptr::{self, NonNull};
use std::rc::Rc;

use crate::ffi::{self, GtkWidget, FALSE, GTK_POLICY_AUTOMATIC, GTK_WINDOW_TOPLEVEL};
use crate::signal::{connect, connect_event, connect_switch_page};
use crate::util::{bool_value, c_string};

#[doc(hidden)]
#[derive(Clone, Copy)]
pub struct Widget {
    pointer: NonNull<GtkWidget>,
    not_send: PhantomData<Rc<()>>,
}

impl Widget {
    fn new(pointer: *mut GtkWidget) -> Self {
        Self {
            pointer: NonNull::new(pointer).expect("GTK returned a null widget"),
            not_send: PhantomData,
        }
    }

    pub(crate) fn pointer(self) -> *mut GtkWidget {
        self.pointer.as_ptr()
    }
}

mod sealed {
    use super::Widget;

    pub trait Sealed {
        fn widget(&self) -> Widget;
    }
}

pub trait WidgetExt: sealed::Sealed {
    fn set_size_request(&self, width: i32, height: i32) {
        unsafe { ffi::gtk_widget_set_size_request(self.widget().pointer(), width, height) }
    }

    fn set_sensitive(&self, sensitive: bool) {
        unsafe {
            ffi::gtk_widget_set_sensitive(self.widget().pointer(), bool_value(sensitive));
        }
    }

    fn show_all(&self) {
        unsafe { ffi::gtk_widget_show_all(self.widget().pointer()) }
    }

    fn hide(&self) {
        unsafe { ffi::gtk_widget_hide(self.widget().pointer()) }
    }
}

impl<T: sealed::Sealed> WidgetExt for T {}

macro_rules! widget_type {
    ($name:ident) => {
        #[derive(Clone, Copy)]
        pub struct $name(Widget);

        impl sealed::Sealed for $name {
            fn widget(&self) -> Widget {
                self.0
            }
        }
    };
}

widget_type!(Window);
widget_type!(BoxLayout);
widget_type!(Button);
widget_type!(Label);
widget_type!(Entry);
widget_type!(ScrolledWindow);
widget_type!(Notebook);
widget_type!(Frame);
widget_type!(ProgressBar);

impl Window {
    pub fn new() -> Self {
        Self(Widget::new(unsafe {
            ffi::gtk_window_new(GTK_WINDOW_TOPLEVEL)
        }))
    }

    pub fn set_title(&self, title: &str) {
        unsafe { ffi::gtk_window_set_title(self.0.pointer(), c_string(title).as_ptr()) }
    }

    pub fn set_default_size(&self, width: i32, height: i32) {
        unsafe { ffi::gtk_window_set_default_size(self.0.pointer(), width, height) }
    }

    pub fn add<W: WidgetExt>(&self, child: &W) {
        unsafe { ffi::gtk_container_add(self.0.pointer(), child.widget().pointer()) }
    }

    pub fn redraw(&self) {
        unsafe { ffi::gtk_widget_queue_draw(self.0.pointer()) }
    }

    pub fn on_destroy<F: Fn() + 'static>(&self, callback: F) {
        connect(self.0, "destroy", callback)
    }
}

impl Default for Window {
    fn default() -> Self {
        Self::new()
    }
}

impl BoxLayout {
    pub fn vertical(spacing: i32) -> Self {
        Self(Widget::new(unsafe { ffi::gtk_vbox_new(FALSE, spacing) }))
    }

    pub fn horizontal(spacing: i32) -> Self {
        Self(Widget::new(unsafe { ffi::gtk_hbox_new(FALSE, spacing) }))
    }

    pub fn set_border_width(&self, width: u32) {
        unsafe { ffi::gtk_container_set_border_width(self.0.pointer(), width) }
    }

    pub fn pack_start<W: WidgetExt>(&self, child: &W, expand: bool, fill: bool, padding: u32) {
        unsafe {
            ffi::gtk_box_pack_start(
                self.0.pointer(),
                child.widget().pointer(),
                bool_value(expand),
                bool_value(fill),
                padding,
            )
        }
    }

    pub fn pack_end<W: WidgetExt>(&self, child: &W, expand: bool, fill: bool, padding: u32) {
        unsafe {
            ffi::gtk_box_pack_end(
                self.0.pointer(),
                child.widget().pointer(),
                bool_value(expand),
                bool_value(fill),
                padding,
            )
        }
    }

    pub fn clear(&self) {
        unsafe {
            ffi::gtk_container_foreach(self.0.pointer(), Some(destroy_child), ptr::null_mut());
        }
    }
}

impl Button {
    pub fn new(label: &str) -> Self {
        Self(Widget::new(unsafe {
            ffi::gtk_button_new_with_label(c_string(label).as_ptr())
        }))
    }

    pub fn on_clicked<F: Fn() + 'static>(&self, callback: F) {
        connect(self.0, "clicked", callback)
    }
}

impl Label {
    pub fn new(text: &str) -> Self {
        Self(Widget::new(unsafe {
            ffi::gtk_label_new(c_string(text).as_ptr())
        }))
    }

    pub fn set_text(&self, text: &str) {
        unsafe { ffi::gtk_label_set_text(self.0.pointer(), c_string(text).as_ptr()) }
    }

    pub fn set_line_wrap(&self, wrap: bool) {
        unsafe { ffi::gtk_label_set_line_wrap(self.0.pointer(), bool_value(wrap)) }
    }

    pub fn set_alignment(&self, horizontal: f32, vertical: f32) {
        unsafe { ffi::gtk_misc_set_alignment(self.0.pointer(), horizontal, vertical) }
    }
}

impl Entry {
    pub fn new() -> Self {
        Self(Widget::new(unsafe { ffi::gtk_entry_new() }))
    }

    pub fn text(&self) -> String {
        let text = unsafe { ffi::gtk_entry_get_text(self.0.pointer()) };
        if text.is_null() {
            String::new()
        } else {
            unsafe { CStr::from_ptr(text) }
                .to_string_lossy()
                .into_owned()
        }
    }

    pub fn set_text(&self, text: &str) {
        unsafe { ffi::gtk_entry_set_text(self.0.pointer(), c_string(text).as_ptr()) }
    }

    pub fn insert_text(&self, text: &str) {
        let text = c_string(text);
        let mut position = unsafe { ffi::gtk_editable_get_position(self.0.pointer()) };
        unsafe {
            ffi::gtk_editable_insert_text(
                self.0.pointer(),
                text.as_ptr(),
                text.as_bytes().len() as c_int,
                &mut position,
            );
            ffi::gtk_editable_set_position(self.0.pointer(), position);
        }
    }

    pub fn backspace(&self) {
        let position = unsafe { ffi::gtk_editable_get_position(self.0.pointer()) };
        if position > 0 {
            unsafe {
                ffi::gtk_editable_delete_text(self.0.pointer(), position - 1, position);
                ffi::gtk_editable_set_position(self.0.pointer(), position - 1);
            }
        }
    }

    pub fn on_activate<F: Fn() + 'static>(&self, callback: F) {
        connect(self.0, "activate", callback)
    }

    pub fn on_changed<F: Fn() + 'static>(&self, callback: F) {
        connect(self.0, "changed", callback)
    }

    pub fn on_focus_in<F: Fn() + 'static>(&self, callback: F) {
        connect_event(self.0, "focus-in-event", callback)
    }

    pub fn on_focus_out<F: Fn() + 'static>(&self, callback: F) {
        connect_event(self.0, "focus-out-event", callback)
    }

    pub fn on_button_press<F: Fn() + 'static>(&self, callback: F) {
        connect_event(self.0, "button-press-event", callback)
    }
}

impl Default for Entry {
    fn default() -> Self {
        Self::new()
    }
}

impl ScrolledWindow {
    pub fn new() -> Self {
        let widget = Self(Widget::new(unsafe {
            ffi::gtk_scrolled_window_new(ptr::null_mut(), ptr::null_mut())
        }));
        unsafe {
            ffi::gtk_scrolled_window_set_policy(
                widget.0.pointer(),
                GTK_POLICY_AUTOMATIC,
                GTK_POLICY_AUTOMATIC,
            );
        }
        widget
    }

    pub fn add_with_viewport<W: WidgetExt>(&self, child: &W) {
        unsafe {
            ffi::gtk_scrolled_window_add_with_viewport(self.0.pointer(), child.widget().pointer())
        }
    }
}

impl Default for ScrolledWindow {
    fn default() -> Self {
        Self::new()
    }
}

impl Notebook {
    pub fn new() -> Self {
        Self(Widget::new(unsafe { ffi::gtk_notebook_new() }))
    }

    pub fn append_page<W: WidgetExt>(&self, child: &W, title: &str) {
        let tab = Label::new(title);
        unsafe {
            ffi::gtk_notebook_append_page(
                self.0.pointer(),
                child.widget().pointer(),
                tab.0.pointer(),
            );
        }
    }

    pub fn on_switch_page<F: Fn(u32) + 'static>(&self, callback: F) {
        connect_switch_page(self.0, callback)
    }
}

impl Default for Notebook {
    fn default() -> Self {
        Self::new()
    }
}

impl Frame {
    pub fn new() -> Self {
        Self(Widget::new(unsafe { ffi::gtk_frame_new(ptr::null()) }))
    }

    pub fn add<W: WidgetExt>(&self, child: &W) {
        unsafe { ffi::gtk_container_add(self.0.pointer(), child.widget().pointer()) }
    }
}

impl Default for Frame {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgressBar {
    pub fn new() -> Self {
        Self(Widget::new(unsafe { ffi::gtk_progress_bar_new() }))
    }

    pub fn set_fraction(&self, fraction: f64) {
        unsafe { ffi::gtk_progress_bar_set_fraction(self.0.pointer(), fraction.clamp(0.0, 1.0)) }
    }

    pub fn set_pulse_step(&self, fraction: f64) {
        unsafe { ffi::gtk_progress_bar_set_pulse_step(self.0.pointer(), fraction.clamp(0.0, 1.0)) }
    }

    pub fn set_text(&self, text: &str) {
        unsafe { ffi::gtk_progress_bar_set_text(self.0.pointer(), c_string(text).as_ptr()) }
    }

    pub fn pulse(&self) {
        unsafe { ffi::gtk_progress_bar_pulse(self.0.pointer()) }
    }
}

impl Default for ProgressBar {
    fn default() -> Self {
        Self::new()
    }
}

unsafe extern "C" fn destroy_child(widget: *mut GtkWidget, _data: *mut c_void) {
    unsafe { ffi::gtk_widget_destroy(widget) };
}

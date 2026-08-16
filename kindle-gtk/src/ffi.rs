use std::ffi::{c_char, c_int, c_uint, c_ulong, c_void};

pub(crate) const FALSE: c_int = 0;
pub(crate) const TRUE: c_int = 1;
pub(crate) const GTK_WINDOW_TOPLEVEL: c_int = 0;
pub(crate) const GTK_POLICY_AUTOMATIC: c_int = 1;
pub(crate) const G_PRIORITY_DEFAULT: c_int = 0;
pub(crate) const G_CONNECT_AFTER: c_int = 1;

#[repr(C)]
pub(crate) struct GtkWidget {
    _private: [u8; 0],
}

#[repr(C)]
pub(crate) struct GClosure {
    _private: [u8; 0],
}

pub(crate) type GCallback = unsafe extern "C" fn();
pub(crate) type ClosureNotify = unsafe extern "C" fn(*mut c_void, *mut GClosure);
pub(crate) type SourceCallback = unsafe extern "C" fn(*mut c_void) -> c_int;
pub(crate) type DestroyNotify = unsafe extern "C" fn(*mut c_void);
pub(crate) type ContainerCallback = unsafe extern "C" fn(*mut GtkWidget, *mut c_void);

#[link(name = "gtk-x11-2.0")]
#[link(name = "gdk-x11-2.0")]
#[link(name = "gobject-2.0")]
#[link(name = "glib-2.0")]
extern "C" {
    pub(crate) fn gtk_init(argc: *mut c_int, argv: *mut *mut *mut c_char);
    pub(crate) fn gtk_main();
    pub(crate) fn gtk_main_quit();
    pub(crate) fn gtk_window_new(window_type: c_int) -> *mut GtkWidget;
    pub(crate) fn gtk_window_set_title(window: *mut GtkWidget, title: *const c_char);
    pub(crate) fn gtk_window_set_default_size(window: *mut GtkWidget, width: c_int, height: c_int);
    pub(crate) fn gtk_container_add(container: *mut GtkWidget, widget: *mut GtkWidget);
    pub(crate) fn gtk_container_set_border_width(container: *mut GtkWidget, border_width: c_uint);
    pub(crate) fn gtk_container_foreach(
        container: *mut GtkWidget,
        callback: Option<ContainerCallback>,
        callback_data: *mut c_void,
    );
    pub(crate) fn gtk_vbox_new(homogeneous: c_int, spacing: c_int) -> *mut GtkWidget;
    pub(crate) fn gtk_hbox_new(homogeneous: c_int, spacing: c_int) -> *mut GtkWidget;
    pub(crate) fn gtk_box_pack_start(
        container: *mut GtkWidget,
        child: *mut GtkWidget,
        expand: c_int,
        fill: c_int,
        padding: c_uint,
    );
    pub(crate) fn gtk_box_pack_end(
        container: *mut GtkWidget,
        child: *mut GtkWidget,
        expand: c_int,
        fill: c_int,
        padding: c_uint,
    );
    pub(crate) fn gtk_button_new_with_label(label: *const c_char) -> *mut GtkWidget;
    pub(crate) fn gtk_label_new(label: *const c_char) -> *mut GtkWidget;
    pub(crate) fn gtk_label_set_text(label: *mut GtkWidget, text: *const c_char);
    pub(crate) fn gtk_label_set_line_wrap(label: *mut GtkWidget, wrap: c_int);
    pub(crate) fn gtk_misc_set_alignment(misc: *mut GtkWidget, xalign: f32, yalign: f32);
    pub(crate) fn gtk_entry_new() -> *mut GtkWidget;
    pub(crate) fn gtk_entry_get_text(entry: *mut GtkWidget) -> *const c_char;
    pub(crate) fn gtk_entry_set_text(entry: *mut GtkWidget, text: *const c_char);
    pub(crate) fn gtk_editable_get_position(editable: *mut GtkWidget) -> c_int;
    pub(crate) fn gtk_editable_set_position(editable: *mut GtkWidget, position: c_int);
    pub(crate) fn gtk_editable_insert_text(
        editable: *mut GtkWidget,
        text: *const c_char,
        length: c_int,
        position: *mut c_int,
    );
    pub(crate) fn gtk_editable_delete_text(editable: *mut GtkWidget, start: c_int, end: c_int);
    pub(crate) fn gtk_scrolled_window_new(
        horizontal_adjustment: *mut c_void,
        vertical_adjustment: *mut c_void,
    ) -> *mut GtkWidget;
    pub(crate) fn gtk_scrolled_window_set_policy(
        scrolled_window: *mut GtkWidget,
        horizontal_policy: c_int,
        vertical_policy: c_int,
    );
    pub(crate) fn gtk_scrolled_window_add_with_viewport(
        scrolled_window: *mut GtkWidget,
        child: *mut GtkWidget,
    );
    pub(crate) fn gtk_notebook_new() -> *mut GtkWidget;
    pub(crate) fn gtk_notebook_append_page(
        notebook: *mut GtkWidget,
        child: *mut GtkWidget,
        tab_label: *mut GtkWidget,
    ) -> c_int;
    pub(crate) fn gtk_frame_new(label: *const c_char) -> *mut GtkWidget;
    pub(crate) fn gtk_progress_bar_new() -> *mut GtkWidget;
    pub(crate) fn gtk_progress_bar_set_fraction(progress_bar: *mut GtkWidget, fraction: f64);
    pub(crate) fn gtk_progress_bar_set_pulse_step(progress_bar: *mut GtkWidget, fraction: f64);
    pub(crate) fn gtk_progress_bar_set_text(progress_bar: *mut GtkWidget, text: *const c_char);
    pub(crate) fn gtk_progress_bar_pulse(progress_bar: *mut GtkWidget);
    pub(crate) fn gtk_widget_set_size_request(widget: *mut GtkWidget, width: c_int, height: c_int);
    pub(crate) fn gtk_widget_set_sensitive(widget: *mut GtkWidget, sensitive: c_int);
    pub(crate) fn gtk_widget_show_all(widget: *mut GtkWidget);
    pub(crate) fn gtk_widget_hide(widget: *mut GtkWidget);
    pub(crate) fn gtk_widget_queue_draw(widget: *mut GtkWidget);
    pub(crate) fn gtk_widget_destroy(widget: *mut GtkWidget);
    pub(crate) fn g_signal_connect_data(
        instance: *mut GtkWidget,
        detailed_signal: *const c_char,
        callback: Option<GCallback>,
        data: *mut c_void,
        destroy_data: Option<ClosureNotify>,
        connect_flags: c_int,
    ) -> c_ulong;
    pub(crate) fn g_timeout_add_full(
        priority: c_int,
        interval: c_uint,
        function: Option<SourceCallback>,
        data: *mut c_void,
        notify: Option<DestroyNotify>,
    ) -> c_uint;
    pub(crate) fn g_source_remove(tag: c_uint) -> c_int;
}

use kindle_gtk::{BoxLayout, Button, Notebook, ScrolledWindow, WidgetExt};

pub(super) fn button(text: &str) -> Button {
    let button = Button::new(text);
    button.set_size_request(-1, 48);
    button
}

pub(super) fn list_page(notebook: &Notebook, title: &str) -> BoxLayout {
    let (scroller, list) = scrollable_list();
    notebook.append_page(&scroller, title);
    list
}

pub(super) fn scrollable_list() -> (ScrolledWindow, BoxLayout) {
    let scroller = ScrolledWindow::new();
    let list = BoxLayout::vertical(10);
    list.set_border_width(8);
    scroller.add_with_viewport(&list);
    (scroller, list)
}

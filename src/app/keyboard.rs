//! Dirty hack to get a keyboard working since the build in keyboard somehow keeps crashing
//! on my kindle colorsoft when i press the back key
//!
//! TODO: Figure out why and fix it in the future
use std::cell::Cell;
use std::rc::Rc;

use kindle_gtk::{BoxLayout, Entry, WidgetExt};

use super::widget::button;

pub(super) fn build_keyboard(active_entry: Rc<Cell<Option<Entry>>>) -> BoxLayout {
    let keyboard = BoxLayout::vertical(5);
    keyboard.set_border_width(12);
    let shift = Rc::new(Cell::new(false));

    let rows: &[&[&str]] = &[
        &["1", "2", "3", "4", "5", "6", "7", "8", "9", "0"],
        &["q", "w", "e", "r", "t", "y", "u", "i", "o", "p"],
        &["a", "s", "d", "f", "g", "h", "j", "k", "l"],
    ];
    for keys in rows {
        let row = BoxLayout::horizontal(4);
        for key in *keys {
            let key_button = button(key);
            connect_key(&key_button, active_entry.clone(), shift.clone(), key);
            row.pack_start(&key_button, true, true, 0);
        }
        keyboard.pack_start(&row, false, false, 0);
    }

    let letters = BoxLayout::horizontal(4);
    let shift_button = button("Shift");
    let shift_state = shift.clone();
    shift_button.on_clicked(move || shift_state.set(!shift_state.get()));
    letters.pack_start(&shift_button, true, true, 0);
    for key in ["z", "x", "c", "v", "b", "n", "m"] {
        let key_button = button(key);
        connect_key(&key_button, active_entry.clone(), shift.clone(), key);
        letters.pack_start(&key_button, true, true, 0);
    }

    let backspace = button("Backspace");
    let backspace_entry = active_entry.clone();
    backspace.on_clicked(move || {
        if let Some(entry) = backspace_entry.get() {
            entry.backspace();
        }
    });
    letters.pack_start(&backspace, true, true, 0);
    keyboard.pack_start(&letters, false, false, 0);

    let controls = BoxLayout::horizontal(4);
    for key in ["-", "_", ".", ":", "/", "@"] {
        let key_button = button(key);
        let active_entry = active_entry.clone();
        key_button.on_clicked(move || {
            if let Some(entry) = active_entry.get() {
                entry.insert_text(key);
            }
        });
        controls.pack_start(&key_button, true, true, 0);
    }

    let space = button("Space");
    let space_entry = active_entry.clone();
    space.on_clicked(move || {
        if let Some(entry) = space_entry.get() {
            entry.insert_text(" ");
        }
    });
    controls.pack_start(&space, true, true, 0);

    let clear = button("Clear");
    let clear_entry = active_entry.clone();
    clear.on_clicked(move || {
        if let Some(entry) = clear_entry.get() {
            entry.set_text("");
        }
    });
    controls.pack_start(&clear, true, true, 0);

    let close = button("Close");
    let close_entry = active_entry;
    close.on_clicked(move || {
        close_entry.set(None);
        keyboard.hide();
    });
    controls.pack_start(&close, true, true, 0);
    keyboard.pack_start(&controls, false, false, 0);
    keyboard
}

pub(super) fn connect_keyboard(
    entry: &Entry,
    name: &'static str,
    active_entry: &Rc<Cell<Option<Entry>>>,
    keyboard: BoxLayout,
) {
    let selected_entry = *entry;
    let active_entry = active_entry.clone();
    entry.on_button_press(move || {
        eprintln!("[kpm-ui] {name} entry button press");
        active_entry.set(Some(selected_entry));
        keyboard.show_all();
    });
    entry.on_focus_in(move || eprintln!("[kpm-ui] {name} entry focus in"));
    entry.on_focus_out(move || eprintln!("[kpm-ui] {name} entry focus out"));
    let entry = *entry;
    entry.on_changed(move || {
        eprintln!(
            "[kpm-ui] {name} entry changed chars={}",
            entry.text().chars().count()
        );
    });
}

fn connect_key(
    button: &kindle_gtk::Button,
    active_entry: Rc<Cell<Option<Entry>>>,
    shift: Rc<Cell<bool>>,
    key: &str,
) {
    let key = key.to_string();
    button.on_clicked(move || {
        if let Some(entry) = active_entry.get() {
            let text = if shift.replace(false) {
                key.to_uppercase()
            } else {
                key.clone()
            };
            entry.insert_text(&text);
        }
    });
}

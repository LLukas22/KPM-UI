use kindle_gtk::{Button, WidgetExt};

use super::operation::{start_operation, Operation};
use super::{set_status, SharedApp};

pub(super) enum Action {
    Refresh,
    AddRepositoryEntry,
    Operation(Operation, Option<String>),
}

pub(super) fn connect_action(button: &Button, app: &SharedApp, action: Action) {
    let app = app.clone();
    button.on_clicked(move || perform_action(&app, &action));
}

pub(super) fn perform_action(app: &SharedApp, action: &Action) {
    if app.borrow().busy {
        let app = app.borrow();
        app.progress.pulse();
        set_status(&app, "Please wait for the current operation to finish");
        return;
    }

    match action {
        Action::Refresh => start_operation(app, Operation::Refresh, String::new()),
        Action::AddRepositoryEntry => {
            let url = app.borrow().repository_entry.text().trim().to_string();
            if url.is_empty() {
                set_status(&app.borrow(), "Enter a repository manifest URL");
                return;
            }
            start_operation(app, Operation::AddRepository, url);
        }
        Action::Operation(operation, package_id) => {
            let value = package_id.clone().unwrap_or_default();
            let confirmation = match operation {
                Operation::Uninstall => Some(format!("Uninstall '{value}'?")),
                Operation::RemoveRepository => Some(format!("Remove repository '{value}'?")),
                _ => None,
            };
            if let Some(message) = confirmation {
                request_confirmation(app, *operation, value, &message);
            } else {
                start_operation(app, *operation, value);
            }
        }
    }
}

fn request_confirmation(app: &SharedApp, operation: Operation, value: String, message: &str) {
    let mut app = app.borrow_mut();
    app.pending_confirmation = Some((operation, value));
    app.confirmation_label.set_text(message);
    app.confirmation_bar.show_all();
    app.window.redraw();
}

pub(super) fn cancel_confirmation(app: &SharedApp) {
    let mut app = app.borrow_mut();
    app.pending_confirmation = None;
    app.confirmation_bar.hide();
    set_status(&app, "Cancelled");
}

pub(super) fn confirm_pending(app: &SharedApp) {
    let pending = {
        let mut app = app.borrow_mut();
        let pending = app.pending_confirmation.take();
        app.confirmation_bar.hide();
        pending
    };
    if let Some((operation, value)) = pending {
        start_operation(app, operation, value);
    }
}

use kindle_gtk::{BoxLayout, Frame, Label, WidgetExt};
use kpm_ui::{kpm::KpmClient, Repository};

use super::action::{connect_action, Action};
use super::operation::Operation;
use super::widget::button;
use super::SharedApp;

pub(super) fn render_repositories(app: &SharedApp, list: BoxLayout, repositories: Vec<Repository>) {
    list.clear();
    if repositories.is_empty() {
        let empty = Label::new("No repositories configured.");
        empty.set_alignment(0.0, 0.5);
        list.pack_start(&empty, false, false, 8);
    }

    for repository in repositories {
        let frame = Frame::new();
        let row = BoxLayout::vertical(5);
        row.set_border_width(10);
        frame.add(&row);

        let name = Label::new(&repository.name);
        name.set_alignment(0.0, 0.5);
        row.pack_start(&name, false, false, 0);

        let id = Label::new(&repository.id);
        id.set_alignment(0.0, 0.5);
        row.pack_start(&id, false, false, 0);

        let url = Label::new(&repository.url);
        url.set_line_wrap(true);
        url.set_alignment(0.0, 0.0);
        row.pack_start(&url, false, false, 2);

        if repository.id == KpmClient::DEFAULT_REPOSITORY_ID {
            let default = Label::new("Default repository");
            default.set_alignment(1.0, 0.5);
            row.pack_end(&default, false, false, 0);
        } else {
            let remove = button("Remove");
            remove.set_size_request(130, 48);
            row.pack_end(&remove, false, false, 0);
            connect_action(
                &remove,
                app,
                Action::Operation(Operation::RemoveRepository, Some(repository.id)),
            );
        }
        list.pack_start(&frame, false, false, 0);
    }
    list.show_all();
}

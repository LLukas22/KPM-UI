use std::collections::{HashMap, HashSet};
use std::time::Instant;

use kindle_gtk::{BoxLayout, Frame, Label, WidgetExt};
use kpm_ui::Package;

use super::action::{connect_action, Action};
use super::operation::Operation;
use super::widget::button;
use super::{set_status, App, SharedApp};

pub(super) fn package_name(app: &App, id: &str) -> String {
    app.installed_packages
        .iter()
        .chain(&app.available_packages)
        .find(|package| package.id == id)
        .map(|package| package.name.clone())
        .unwrap_or_else(|| id.to_string())
}

pub(super) fn render_packages(app: &SharedApp, packages: Vec<Package>, installed: bool) {
    let started = Instant::now();
    let package_count = packages.len();
    let (list, installed_ids, available_versions) = package_context(app, installed);

    list.clear();
    if packages.is_empty() {
        let empty = Label::new(if installed {
            "No installed packages found."
        } else {
            "No packages found. Update the index or try another search."
        });
        empty.set_alignment(0.0, 0.5);
        list.pack_start(&empty, false, false, 8);
    }

    for package in packages {
        let frame = Frame::new();
        let row = BoxLayout::vertical(5);
        row.set_border_width(10);
        frame.add(&row);

        let newer_version = package.version.and_then(|installed_version| {
            available_versions
                .get(&package.id)
                .copied()
                .filter(|available_version| *available_version > installed_version)
        });
        let title = if installed && newer_version.is_none() && package.version.is_some() {
            format!("{}  v{}", package.name, package.version_text())
        } else {
            package.name.clone()
        };
        let title_label = Label::new(&title);
        title_label.set_alignment(0.0, 0.5);
        row.pack_start(&title_label, false, false, 0);

        let id_label = Label::new(&package.id);
        id_label.set_alignment(0.0, 0.5);
        row.pack_start(&id_label, false, false, 0);

        if let (Some(installed_version), Some(available_version)) = (package.version, newer_version)
        {
            let versions = Label::new(&format!(
                "Installed: {}.{}.{}    New: {}.{}.{}",
                installed_version[0],
                installed_version[1],
                installed_version[2],
                available_version[0],
                available_version[1],
                available_version[2]
            ));
            versions.set_alignment(0.0, 0.5);
            row.pack_start(&versions, false, false, 2);
        }

        if !package.description.is_empty() {
            let description = Label::new(&package.description);
            description.set_line_wrap(true);
            description.set_alignment(0.0, 0.0);
            row.pack_start(&description, false, false, 2);
        }

        add_package_action(
            app,
            &row,
            package,
            installed,
            newer_version.is_some(),
            &installed_ids,
        );
        list.pack_start(&frame, false, false, 0);
    }
    list.show_all();
    eprintln!(
        "[kpm-ui] rendered {package_count} {} package rows in {} ms",
        if installed { "installed" } else { "available" },
        started.elapsed().as_millis()
    );
}

pub(super) fn filter_available_packages(app: &SharedApp) {
    let started = Instant::now();
    let (packages, total, query) = {
        let app = app.borrow();
        let query = app.search_entry.text().trim().to_lowercase();
        let terms: Vec<_> = query.split_whitespace().collect();
        let packages = app
            .available_packages
            .iter()
            .filter(|package| {
                let searchable = format!(
                    "{} {} {} {}",
                    package.id, package.name, package.author, package.description
                )
                .to_lowercase();
                terms.iter().all(|term| searchable.contains(term))
            })
            .cloned()
            .collect::<Vec<_>>();
        (packages, app.available_packages.len(), query)
    };
    let count = packages.len();
    render_packages(app, packages, false);
    eprintln!(
        "[kpm-ui] filtered query_chars={} matches={count} total={total} in {} ms",
        query.chars().count(),
        started.elapsed().as_millis()
    );
    if !app.borrow().busy {
        let message = if query.is_empty() {
            format!("{total} package(s) available")
        } else {
            format!("{count} of {total} package(s) match")
        };
        set_status(&app.borrow(), &message);
    }
}

fn package_context(
    app: &SharedApp,
    installed: bool,
) -> (BoxLayout, HashSet<String>, HashMap<String, [u64; 3]>) {
    let app = app.borrow();
    let list = if installed {
        app.installed_list
    } else {
        app.available_list
    };
    let installed_ids = app
        .installed_packages
        .iter()
        .map(|package| package.id.clone())
        .collect();
    let available_versions = app
        .available_packages
        .iter()
        .filter_map(|package| package.version.map(|version| (package.id.clone(), version)))
        .collect();
    (list, installed_ids, available_versions)
}

fn add_package_action(
    app: &SharedApp,
    row: &BoxLayout,
    package: Package,
    installed: bool,
    update_available: bool,
    installed_ids: &HashSet<String>,
) {
    if installed {
        if update_available {
            let update = button("Update");
            update.set_size_request(130, 48);
            row.pack_end(&update, false, false, 0);
            connect_action(
                &update,
                app,
                Action::Operation(Operation::UpdatePackage, Some(package.id.clone())),
            );
        }

        let uninstall = button("Uninstall");
        uninstall.set_size_request(130, 48);
        row.pack_end(&uninstall, false, false, 0);
        connect_action(
            &uninstall,
            app,
            Action::Operation(Operation::Uninstall, Some(package.id)),
        );
    } else if installed_ids.contains(&package.id) {
        let installed_label = Label::new("Installed");
        installed_label.set_alignment(1.0, 0.5);
        row.pack_end(&installed_label, false, false, 8);
    } else {
        let install = button("Install");
        install.set_size_request(130, 48);
        row.pack_end(&install, false, false, 0);
        connect_action(
            &install,
            app,
            Action::Operation(Operation::Install, Some(package.id)),
        );
    }
}

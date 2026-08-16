use std::thread;

use kindle_gtk::WidgetExt;
use kpm_ui::{kpm::KpmClient, Package, Repository};

use super::package::{filter_available_packages, package_name, render_packages};
use super::repository::render_repositories;
use super::{set_status, App, SharedApp};

#[derive(Clone, Copy)]
pub(super) enum Operation {
    Refresh,
    Install,
    UpdatePackage,
    Uninstall,
    Update,
    Upgrade,
    LoadAvailable,
    AddRepository,
    RemoveRepository,
}

pub(super) enum UiEvent {
    Progress(String),
    Complete(Completion),
}

pub(super) struct Completion {
    operation: Operation,
    value: String,
    result: Result<OperationResult, String>,
}

enum OperationResult {
    AvailablePackages(Vec<Package>),
    PackageChange {
        output: String,
        installed: Vec<Package>,
    },
    RepositoryChange {
        output: String,
        repositories: Vec<Repository>,
    },
    Refresh {
        installed: Vec<Package>,
        repositories: Vec<Repository>,
        available: Vec<Package>,
    },
    Message(String),
}

pub(super) fn start_operation(app: &SharedApp, operation: Operation, value: String) {
    eprintln!(
        "[kpm-ui] operation started: {} value={value:?}",
        operation.progress_message()
    );
    let (kpm, sender) = {
        let mut app = app.borrow_mut();
        app.pending_confirmation = None;
        app.confirmation_bar.hide();
        set_busy(&mut app, true, operation.progress_message());
        (
            app.kpm.clone(),
            app.events
                .as_ref()
                .expect("UI event channel is initialized")
                .clone(),
        )
    };

    thread::spawn(move || {
        let progress_sender = sender.clone();
        let mut progress = move |message: &str| {
            let _ = progress_sender.send(UiEvent::Progress(message.to_string()));
        };
        let result = execute(&kpm, operation, &value, &mut progress);
        let _ = sender.send(UiEvent::Complete(Completion {
            operation,
            value,
            result,
        }));
    });
}

pub(super) fn handle_ui_event(app: &SharedApp, event: UiEvent) {
    match event {
        UiEvent::Progress(message) => update_progress(&app.borrow(), &message),
        UiEvent::Complete(completion) => complete_operation(app, completion),
    }
}

fn execute(
    kpm: &KpmClient,
    operation: Operation,
    value: &str,
    progress: &mut dyn FnMut(&str),
) -> Result<OperationResult, String> {
    match operation {
        Operation::Refresh => {
            progress("Reading installed packages...");
            let installed = kpm.installed_packages()?;
            progress("Reading configured repositories...");
            let repositories = kpm.repositories(progress)?;
            progress("Loading available packages...");
            let available = kpm.search("", progress)?;
            Ok(OperationResult::Refresh {
                installed,
                repositories,
                available,
            })
        }
        Operation::Install | Operation::UpdatePackage | Operation::Uninstall => {
            let output = match operation {
                Operation::Install | Operation::UpdatePackage => kpm.install(value, progress)?,
                Operation::Uninstall => kpm.uninstall(value, progress)?,
                _ => unreachable!(),
            };
            progress("Refreshing installed packages...");
            let installed = kpm.installed_packages()?;
            Ok(OperationResult::PackageChange { output, installed })
        }
        Operation::Update => kpm.update(progress).map(OperationResult::Message),
        Operation::Upgrade => {
            let output = kpm.upgrade(progress)?;
            progress("Refreshing installed packages...");
            let installed = kpm.installed_packages()?;
            Ok(OperationResult::PackageChange { output, installed })
        }
        Operation::LoadAvailable => kpm
            .search("", progress)
            .map(OperationResult::AvailablePackages),
        Operation::AddRepository | Operation::RemoveRepository => {
            let output = match operation {
                Operation::AddRepository => kpm.add_repository(value, progress)?,
                Operation::RemoveRepository => kpm.remove_repository(value, progress)?,
                _ => unreachable!(),
            };
            progress("Refreshing configured repositories...");
            let repositories = kpm.repositories(progress)?;
            Ok(OperationResult::RepositoryChange {
                output,
                repositories,
            })
        }
    }
}

fn complete_operation(app: &SharedApp, completion: Completion) {
    match completion.result {
        Ok(OperationResult::AvailablePackages(packages)) => {
            app.borrow_mut().available_packages = packages;
            finish_operation(app, "Available packages updated");
            let installed = app.borrow().installed_packages.clone();
            render_packages(app, installed, true);
            filter_available_packages(app);
        }
        Ok(OperationResult::PackageChange { output, installed }) => {
            let package_name = package_name(&app.borrow(), &completion.value);
            app.borrow_mut().installed_packages = installed.clone();
            render_packages(app, installed, true);
            filter_available_packages(app);
            let message = match completion.operation {
                Operation::Install => format!("Installed {package_name}"),
                Operation::UpdatePackage => format!("Updated {package_name}"),
                Operation::Uninstall => format!("Uninstalled {package_name}"),
                _ => KpmClient::summarize_output(&output),
            };
            finish_operation(app, &message);
        }
        Ok(OperationResult::RepositoryChange {
            output,
            repositories,
        }) => {
            app.borrow_mut().repositories = repositories.clone();
            render_repositories(app, app.borrow().repository_list, repositories);
            if matches!(completion.operation, Operation::AddRepository) {
                app.borrow().repository_entry.set_text("");
            }
            finish_operation(app, &KpmClient::summarize_output(&output));
            start_operation(app, Operation::LoadAvailable, String::new());
        }
        Ok(OperationResult::Refresh {
            installed,
            repositories,
            available,
        }) => {
            let package_count = available.len();
            {
                let mut app = app.borrow_mut();
                app.available_packages = available;
                app.installed_packages = installed.clone();
                app.repositories = repositories.clone();
            }
            render_packages(app, installed, true);
            render_repositories(app, app.borrow().repository_list, repositories);
            filter_available_packages(app);
            finish_operation(
                app,
                &format!("Ready - {package_count} package(s) available"),
            );
        }
        Ok(OperationResult::Message(output)) => {
            finish_operation(app, &KpmClient::summarize_output(&output));
            if matches!(completion.operation, Operation::Update) {
                start_operation(app, Operation::LoadAvailable, String::new());
            }
        }
        Err(error) => {
            eprintln!("[kpm-ui] operation failed: {error}");
            finish_operation(app, &format!("Error: {error}"));
        }
    }
}

fn set_busy(app: &mut App, busy: bool, message: &str) {
    app.busy = busy;
    app.notebook.set_sensitive(!busy);
    if busy {
        app.keyboard.hide();
        app.progress.set_fraction(0.0);
        app.progress.set_text("Working...");
        app.progress.show_all();
    } else {
        app.progress.hide();
    }
    set_status(app, message);
    app.window.redraw();
}

fn finish_operation(app: &SharedApp, message: &str) {
    set_busy(&mut app.borrow_mut(), false, message);
}

fn update_progress(app: &App, message: &str) {
    if !app.busy {
        return;
    }
    if let Some(percent) = KpmClient::progress_percent(message) {
        app.progress.set_fraction(percent / 100.0);
        app.progress.set_text(&format!("{percent:.0}%"));
    } else {
        app.progress.pulse();
        app.progress.set_text("Working...");
    }
    if !message.is_empty() {
        set_status(app, message);
    }
}

impl Operation {
    fn progress_message(self) -> &'static str {
        match self {
            Self::Refresh => "Refreshing package information...",
            Self::Install => "Installing package...",
            Self::UpdatePackage => "Updating package...",
            Self::Uninstall => "Uninstalling package...",
            Self::Update => "Updating repository index...",
            Self::Upgrade => "Upgrading installed packages...",
            Self::LoadAvailable => "Loading available packages...",
            Self::AddRepository => "Adding repository and updating index...",
            Self::RemoveRepository => "Removing repository and updating index...",
        }
    }
}

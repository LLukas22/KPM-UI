mod action;
mod keyboard;
mod operation;
mod package;
mod repository;
mod widget;

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use action::{connect_action, perform_action, Action};
use keyboard::{build_keyboard, connect_keyboard};
use kindle_gtk::{
    self as gtk, BoxLayout, Entry, Label, Notebook, ProgressBar, UiSender, UiSource, WidgetExt,
    Window,
};
use kpm_ui::{kpm::KpmClient, Package, Repository};
use operation::{handle_ui_event, start_operation, Operation, UiEvent};
use package::filter_available_packages;
use repository::render_repositories;
use widget::{button, list_page, scrollable_list};

const WINDOW_TITLE: &str = "L:A_N:application_PC:T_ID:org.kindlemodding.kpm-ui";

type SharedApp = Rc<RefCell<App>>;

struct App {
    kpm: KpmClient,
    window: Window,
    notebook: Notebook,
    installed_list: BoxLayout,
    available_list: BoxLayout,
    repository_list: BoxLayout,
    search_entry: Entry,
    repository_entry: Entry,
    keyboard: BoxLayout,
    confirmation_bar: BoxLayout,
    confirmation_label: Label,
    status: Label,
    progress: ProgressBar,
    available_packages: Vec<Package>,
    installed_packages: Vec<Package>,
    repositories: Vec<Repository>,
    busy: bool,
    pending_confirmation: Option<(Operation, String)>,
    events: Option<UiSender<UiEvent>>,
    completion_source: Option<UiSource>,
}

pub fn run() {
    gtk::init();
    let active_entry = Rc::new(Cell::new(None::<Entry>));

    let window = Window::new();
    window.set_title(WINDOW_TITLE);
    window.set_default_size(600, 800);

    let root = BoxLayout::vertical(10);
    root.set_border_width(12);
    window.add(&root);

    let toolbar = BoxLayout::horizontal(8);
    let refresh_button = button("Refresh");
    let update_button = button("Update index");
    let upgrade_button = button("Upgrade all");
    let exit_button = button("Exit");
    toolbar.pack_start(&refresh_button, false, false, 0);
    toolbar.pack_start(&update_button, false, false, 0);
    toolbar.pack_start(&upgrade_button, false, false, 0);
    toolbar.pack_end(&exit_button, false, false, 0);
    root.pack_start(&toolbar, false, false, 0);

    let notebook = Notebook::new();
    let installed_list = list_page(&notebook, "Installed");

    let available_page = BoxLayout::vertical(8);
    let search_bar = BoxLayout::horizontal(8);
    let search_entry = Entry::new();
    search_bar.pack_start(&Label::new("Search:"), false, false, 0);
    search_bar.pack_start(&search_entry, true, true, 0);
    available_page.pack_start(&search_bar, false, false, 0);
    let (available_scroller, available_list) = scrollable_list();
    available_page.pack_start(&available_scroller, true, true, 0);
    notebook.append_page(&available_page, "Available");

    let repository_page = BoxLayout::vertical(8);
    let repository_bar = BoxLayout::horizontal(8);
    let repository_entry = Entry::new();
    let add_repository_button = button("Add repository");
    repository_bar.pack_start(&Label::new("Repository URL:"), false, false, 0);
    repository_bar.pack_start(&repository_entry, true, true, 0);
    repository_bar.pack_start(&add_repository_button, false, false, 0);
    repository_page.pack_start(&repository_bar, false, false, 0);
    let (repository_scroller, repository_list) = scrollable_list();
    repository_page.pack_start(&repository_scroller, true, true, 0);
    notebook.append_page(&repository_page, "Repositories");
    root.pack_start(&notebook, true, true, 0);

    let keyboard = build_keyboard(active_entry.clone());
    root.pack_end(&keyboard, false, false, 0);

    let confirmation_bar = BoxLayout::horizontal(8);
    let confirmation_label = Label::new("");
    confirmation_label.set_line_wrap(true);
    confirmation_label.set_alignment(0.0, 0.5);
    let cancel_button = button("Cancel");
    let confirm_button = button("Confirm");
    confirmation_bar.pack_start(&confirmation_label, true, true, 0);
    confirmation_bar.pack_end(&confirm_button, false, false, 0);
    confirmation_bar.pack_end(&cancel_button, false, false, 0);
    root.pack_end(&confirmation_bar, false, false, 0);

    let status = Label::new("Reading installed packages...");
    status.set_line_wrap(true);
    status.set_alignment(0.0, 0.5);
    root.pack_end(&status, false, false, 0);

    let progress = ProgressBar::new();
    progress.set_size_request(-1, 30);
    progress.set_pulse_step(0.08);
    root.pack_end(&progress, false, false, 0);

    let app = Rc::new(RefCell::new(App {
        kpm: KpmClient::from_env(),
        window,
        notebook,
        installed_list,
        available_list,
        repository_list,
        search_entry,
        repository_entry,
        keyboard,
        confirmation_bar,
        confirmation_label,
        status,
        progress,
        available_packages: Vec::new(),
        installed_packages: Vec::new(),
        repositories: Vec::new(),
        busy: false,
        pending_confirmation: None,
        events: None,
        completion_source: None,
    }));

    let completion_app = app.clone();
    let (completions, completion_source) =
        gtk::ui_channel(move |event| handle_ui_event(&completion_app, event));
    {
        let mut app = app.borrow_mut();
        app.events = Some(completions);
        app.completion_source = Some(completion_source);
    }

    window.on_destroy(|| {
        eprintln!("[kpm-ui] window destroy signal");
        gtk::quit();
    });
    connect_action(&refresh_button, &app, Action::Refresh);
    connect_action(
        &update_button,
        &app,
        Action::Operation(Operation::Update, None),
    );
    connect_action(
        &upgrade_button,
        &app,
        Action::Operation(Operation::Upgrade, None),
    );
    connect_action(&add_repository_button, &app, Action::AddRepositoryEntry);

    let exit_app = app.clone();
    exit_button.on_clicked(move || {
        let app = exit_app.borrow();
        if app.busy {
            app.progress.pulse();
            set_status(&app, "Please wait for the current operation to finish");
        } else {
            gtk::quit();
        }
    });

    let cancel_app = app.clone();
    cancel_button.on_clicked(move || action::cancel_confirmation(&cancel_app));
    let confirm_app = app.clone();
    confirm_button.on_clicked(move || action::confirm_pending(&confirm_app));

    connect_keyboard(&search_entry, "search", &active_entry, keyboard);
    connect_keyboard(&repository_entry, "repository", &active_entry, keyboard);
    let search_app = app.clone();
    search_entry.on_changed(move || filter_available_packages(&search_app));

    let page_app = app.clone();
    let page_active_entry = active_entry.clone();
    notebook.on_switch_page(move |page| {
        page_active_entry.set(None);
        page_app.borrow().keyboard.hide();
        if page == 1 {
            filter_available_packages(&page_app);
        } else if page == 2 {
            let (list, repositories) = {
                let app = page_app.borrow();
                (app.repository_list, app.repositories.clone())
            };
            render_repositories(&page_app, list, repositories);
        }
    });

    let repository_app = app.clone();
    repository_entry
        .on_activate(move || perform_action(&repository_app, &Action::AddRepositoryEntry));

    window.show_all();
    keyboard.hide();
    confirmation_bar.hide();
    progress.hide();
    window.redraw();
    start_operation(&app, Operation::Refresh, String::new());
    gtk::run();
}

fn set_status(app: &App, message: &str) {
    app.status.set_text(message);
}

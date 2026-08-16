mod app;

fn main() {
    std::panic::set_hook(Box::new(|panic| eprintln!("[kpm-ui] panic: {panic}")));
    eprintln!("[kpm-ui] starting pid={}", std::process::id());
    app::run();
    eprintln!("[kpm-ui] GTK main loop exited");
}

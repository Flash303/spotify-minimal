mod blocker;
mod config;
#[cfg(target_os = "windows")]
mod mixer;
mod plan_b;
mod webview;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();
    if std::env::args().any(|arg| arg == "--help" || arg == "-h") {
        println!(
            "{}: minimal Spotify player (wry + adblock)",
            config::APP_TITLE
        );
        println!("usage: spotify-minimal [--plan-b]");
        return;
    }
    if std::env::args().any(|arg| arg == "--plan-b") {
        if let Err(error) = plan_b::launch() {
            eprintln!("plan B failed: {error}");
            std::process::exit(1);
        }
        return;
    }
    #[cfg(target_os = "windows")]
    mixer::ensure_labels();
    let Err(error) = webview::run();
    eprintln!("{} : {error}", config::APP_TITLE);
    std::process::exit(1);
}

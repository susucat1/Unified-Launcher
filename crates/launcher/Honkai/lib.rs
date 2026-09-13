use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::LazyLock;

use anime_launcher_sdk::anime_game_core::honkai::prelude::*;
use anime_launcher_sdk::anime_game_core::prelude::*;
use anime_launcher_sdk::config::ConfigExt;
use anime_launcher_sdk::honkai::config::{Config, Schema};
use anime_launcher_sdk::honkai::consts::*;
use anime_launcher_sdk::honkai::sessions::Sessions;
use anime_launcher_sdk::honkai::states::LauncherState;
use anime_launcher_sdk::sessions::SessionsExt;
use anyhow::{Context, Result};
use iced::widget::{container, text, Column};
use iced::{window, Application, Command, Element, Length, Settings, Theme};
use tracing_subscriber::filter::*;
use tracing_subscriber::prelude::*;

pub mod background;
pub mod i18n;
pub mod move_files;
pub mod ui;

pub const APP_ID: &str = "unified-launcher";
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const APP_DEBUG: bool = cfg!(debug_assertions);

pub static READY: AtomicBool = AtomicBool::new(false);

#[inline(always)]
pub fn is_ready() -> bool {
    READY.load(Ordering::Relaxed)
}

// ==========================================
// THAY THẾ LAZY_STATIC BẰNG STD::SYNC::LAZYLOCK
// ==========================================
pub static CONFIG: LazyLock<Schema> =
    LazyLock::new(|| Config::get().expect("Failed to load config"));

pub static GAME: LazyLock<Game> = LazyLock::new(|| {
    Game::new(
        CONFIG.game.path.for_edition(CONFIG.launcher.edition),
        CONFIG.launcher.edition,
    )
});

pub static LAUNCHER_FOLDER: LazyLock<PathBuf> =
    LazyLock::new(|| launcher_dir().expect("Failed to get launcher folder"));

pub static CACHE_FOLDER: LazyLock<PathBuf> =
    LazyLock::new(|| cache_dir().expect("Failed to get launcher's cache folder"));

pub static DEBUG_FILE: LazyLock<PathBuf> = LazyLock::new(|| LAUNCHER_FOLDER.join("debug.log"));
pub static BACKGROUND_FILE: LazyLock<PathBuf> = LazyLock::new(|| LAUNCHER_FOLDER.join("background"));
pub static PROCESSED_BACKGROUND_FILE: LazyLock<PathBuf> =
    LazyLock::new(|| CACHE_FOLDER.join("background"));
pub static KEEP_BACKGROUND_FILE: LazyLock<PathBuf> =
    LazyLock::new(|| LAUNCHER_FOLDER.join(".keep-background"));
pub static FIRST_RUN_FILE: LazyLock<PathBuf> =
    LazyLock::new(|| LAUNCHER_FOLDER.join(".first-run"));

// ==========================================
// CLI & TRACING
// ==========================================
fn print_help() {
    println!(
        r#"
Unified Launcher {APP_VERSION}

Usage:
  unified-launcher [OPTION...]

Options:
  -h, --help            Show this help message and exit
  --debug               Force debug output in stdout (repeat for trace)
  --no-verbose-tracing  Disable verbose tracing output in stdout
  --run-game            Launch the game right away if it's ready
  --just-run-game       Same as --run-game, but also pre-downloads
  --session <NAME>      Switch to the given session before starting
    "#
    );
}

#[derive(Default)]
struct CliOptions {
    force_debug: u8,
    run_game: bool,
    just_run_game: bool,
    no_verbose_tracing: bool,
    session: Option<String>,
}

fn parse_args() -> CliOptions {
    let mut opts = CliOptions::default();
    let mut args = std::env::args().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                print_help();
                std::process::exit(0);
            }
            "--debug" => opts.force_debug += 1,
            "--run-game" => opts.run_game = true,
            "--just-run-game" => opts.just_run_game = true,
            "--no-verbose-tracing" => opts.no_verbose_tracing = true,
            "--session" => {
                if let Some(session) = args.next() {
                    opts.session = Some(session);
                }
            }
            _ => {}
        }
    }
    opts
}

// ==========================================
// ICED LAUNCHER APP
// ==========================================
pub enum Screen {
    FirstRun(ui::first_run::main::FirstRunState),
    Main(ui::main::MainState),
}

#[derive(Debug, Clone)]
pub enum Message {
    FirstRunMsg(ui::first_run::main::FirstRunMessage),
    MainMsg(ui::main::MainMessage),
    SwitchToMain,
}

pub struct HonkersApp {
    screen: Screen,
}

impl Application for HonkersApp {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let is_first_run = FIRST_RUN_FILE.exists();

        let (screen, cmd) = if is_first_run {
            let (state, cmd) = ui::first_run::main::FirstRunState::new();
            (Screen::FirstRun(state), cmd.map(Message::FirstRunMsg))
        } else {
            let (state, cmd) = ui::main::MainState::new();
            (Screen::Main(state), cmd.map(Message::MainMsg))
        };

        READY.store(true, Ordering::Relaxed);
        (Self { screen }, cmd)
    }

    fn title(&self) -> String {
        format!("Honkers Launcher v{APP_VERSION}")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match (&mut self.screen, message) {
            (Screen::FirstRun(first_run), Message::FirstRunMsg(msg)) => {
                first_run.update(msg).map(Message::FirstRunMsg)
            }
            (Screen::Main(main_state), Message::MainMsg(msg)) => {
                main_state.update(msg).map(Message::MainMsg)
            }

            (_, Message::SwitchToMain) => {
                let (main_state, cmd) = ui::main::MainState::new();
                self.screen = Screen::Main(main_state);
                cmd.map(Message::MainMsg)
            }
            _ => Command::none(),
        }
    }

    fn view(&self) -> Element<Message> {
        match &self.screen {
            Screen::FirstRun(first_run) => first_run.view().map(Message::FirstRunMsg),
            Screen::Main(main_state) => main_state.view().map(Message::MainMsg),
        }
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}

// ==========================================
// MAIN ENTRYPOINT
// ==========================================
fn main() -> Result<()> {
    human_panic::setup_panic!(human_panic::metadata!());

    let cli = parse_args();

    if !LAUNCHER_FOLDER.exists() {
        if LAUNCHER_FOLDER.is_symlink() {
            anyhow::bail!("Launcher folder is a broken symlink: {}", LAUNCHER_FOLDER.display());
        }
        std::fs::create_dir_all(LAUNCHER_FOLDER.as_path())
            .context("Failed to create launcher folder")?;

        let _ = std::fs::write(FIRST_RUN_FILE.as_path(), "");

        if let Ok(mut config) = Config::get() {
            config.launcher.language = i18n::format_lang(i18n::get_default_lang());
            let _ = Config::update_raw(config);
        }
    }
  
    if !CACHE_FOLDER.exists() {
        if CACHE_FOLDER.is_symlink() {
            anyhow::bail!("Cache folder is a broken symlink: {}", CACHE_FOLDER.display());
        }
        std::fs::create_dir_all(CACHE_FOLDER.as_path())
            .context("Failed to create cache folder")?;
    }

    if let Some(session) = cli.session {
        Sessions::set_current(session)?;
    }

    let filter_noisy_crates = |target: &str| {
        !target.contains("rustls")
            && !target.contains("reqwest")
            && !target.contains("h2")
            && !target.contains("hyper")
            && !target.contains("wgpu")
    };

    let log_level = match cli.force_debug {
        0 if APP_DEBUG => LevelFilter::DEBUG,
        0 => LevelFilter::WARN,
        1 => LevelFilter::DEBUG,
        _ => LevelFilter::TRACE,
    };

    let stdout_layer = tracing_subscriber::fmt::layer()
        .compact()
        .with_filter(log_level)
        .with_filter(filter_fn(move |meta| {
            filter_noisy_crates(meta.target()) && !cli.no_verbose_tracing
        }));

    let file = std::fs::File::create(DEBUG_FILE.as_path())
        .context("Failed to create debug.log file")?;

    let file_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .with_writer(std::sync::Arc::new(file))
        .with_filter(log_level)
        .with_filter(filter_fn(move |meta| filter_noisy_crates(meta.target())));

    tracing_subscriber::registry()
        .with(stdout_layer)
        .with(file_layer)
        .init();

    tracing::info!("Starting Honkai Launcher ({APP_VERSION})");

    let lang = CONFIG
        .launcher
        .language
        .parse()
        .context("Wrong language format used in config")?;
    i18n::set_lang(lang).context("Failed to set launcher language")?;

    if cli.run_game || cli.just_run_game {
        if let Ok(state) = LauncherState::get_from_config(|_| {}) {
            match state {
                LauncherState::Launch => {
                    anime_launcher_sdk::honkai::game::run()?;
                    return Ok(());
                }
                LauncherState::PredownloadAvailable { .. } if cli.just_run_game => {
                    anime_launcher_sdk::honkai::game::run()?;
                    return Ok(());
                }
                _ => {}
            }
        }
    }

    let settings = Settings {
        window: window::Settings {
            size: iced::Size::new(1024.0, 600.0),
            min_size: Some(iced::Size::new(800.0, 500.0)),
            resizable: true,
            decorations: true,
            transparent: false,
            ..Default::default()
        },
        flags: (),
        default_font: iced::Font::DEFAULT,
        default_text_size: iced::Pixels(14.0),
        antialiasing: true,
        id: Some(APP_ID.to_string()),
    };

    HonkaiApp::run(settings).map_err(|e| anyhow::anyhow!("Iced error: {e}"))
}

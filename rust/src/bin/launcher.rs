use std::io::{self, IsTerminal, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::terminal;

type Res<T> = Result<T, Box<dyn std::error::Error>>;

// ─── colours ────────────────────────────────────────────────────────────────
const RST: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const RED: &str = "\x1b[91m";
const GRN: &str = "\x1b[92m";
const YLW: &str = "\x1b[93m";
const BLU: &str = "\x1b[94m";
const CYN: &str = "\x1b[96m";

// ─── demo registry ───────────────────────────────────────────────────────────
struct Demo {
    key: char,
    label: &'static str,
    col: &'static str,
    note: Option<&'static str>,
    // (program, args, subdir-relative-to-project-root)
    build: Option<(&'static str, &'static [&'static str], &'static str)>,
    run: (&'static str, &'static [&'static str], &'static str),
}

const DEMOS: &[Demo] = &[
    Demo {
        key: '1',
        label: "C          ·  Part 1 — Array access horrors",
        col: RED,
        note: None,
        build: Some(("make", &["main"], "c")),
        run: ("./main", &[], "c"),
    },
    Demo {
        key: '2',
        label: "C          ·  Part 2 — Pointer lifetime horrors",
        col: RED,
        note: None,
        build: Some(("make", &["part2"], "c")),
        run: ("./part2", &[], "c"),
    },
    Demo {
        key: '3',
        label: "C          ·  Part 3 — Performance & verbosity",
        col: RED,
        note: None,
        build: Some(("make", &["part3"], "c")),
        run: ("./part3", &[], "c"),
    },
    Demo {
        key: '4',
        label: "Rust       ·  Part 1 — Array safety",
        col: GRN,
        note: None,
        build: Some(("cargo", &["build", "--bin", "array-horror-rs"], "rust")),
        run: ("./target/debug/array-horror-rs", &[], "rust"),
    },
    Demo {
        key: '5',
        label: "Rust       ·  Part 2 — Ownership / lifetimes",
        col: GRN,
        note: None,
        build: Some(("cargo", &["build", "--bin", "part2"], "rust")),
        run: ("./target/debug/part2", &[], "rust"),
    },
    Demo {
        key: '6',
        label: "Rust       ·  Part 3 — Performance & verbosity",
        col: GRN,
        note: Some("--release"),
        build: Some(("cargo", &["build", "--release", "--bin", "part3"], "rust")),
        run: ("./target/release/part3", &[], "rust"),
    },
    Demo {
        key: '7',
        label: "TypeScript ·  Part 1 — Array safety",
        col: BLU,
        note: None,
        build: None,
        run: ("bun", &["run", "src/main.ts"], "typescript"),
    },
    Demo {
        key: '8',
        label: "TypeScript ·  Part 2 — GC / null safety",
        col: BLU,
        note: None,
        build: None,
        run: ("bun", &["run", "src/part2.ts"], "typescript"),
    },
    Demo {
        key: '9',
        label: "TypeScript ·  Part 3 — Performance & verbosity",
        col: BLU,
        note: None,
        build: None,
        run: ("bun", &["run", "src/part3.ts"], "typescript"),
    },
    Demo {
        key: 'b',
        label: "C          ·  Bonus — The Cryptic Signature",
        col: RED,
        note: Some("☠"),
        build: Some(("make", &["bonus"], "c")),
        run: ("./bonus", &[], "c"),
    },
];

// ─── paths ───────────────────────────────────────────────────────────────────

// CARGO_MANIFEST_DIR is `rust/` — parent is the project root.
// This is compile-time constant, suitable for a local dev tool.
fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("CARGO_MANIFEST_DIR has no parent")
        .to_path_buf()
}

fn demo_dir(subdir: &str) -> PathBuf {
    project_root().join(subdir)
}

// ─── raw-mode RAII guard ──────────────────────────────────────────────────────
struct RawMode;

impl RawMode {
    fn enable() -> Res<Self> {
        terminal::enable_raw_mode()?;
        Ok(Self)
    }
}

impl Drop for RawMode {
    fn drop(&mut self) {
        terminal::disable_raw_mode().ok();
    }
}

// ─── SIGINT RAII guard ────────────────────────────────────────────────────────
// Stores and restores the previous SIGINT disposition around child execution.
// Using a no-op handler (not SIG_IGN) so the child does not inherit a
// suppressed signal — the child's pre_exec resets it to SIG_DFL.
extern "C" fn noop_sigint(_: libc::c_int) {}

struct SigintGuard(libc::sighandler_t);

impl SigintGuard {
    fn noop() -> Self {
        let prev =
            unsafe { libc::signal(libc::SIGINT, noop_sigint as *const () as libc::sighandler_t) };
        Self(prev)
    }
}

impl Drop for SigintGuard {
    fn drop(&mut self) {
        unsafe { libc::signal(libc::SIGINT, self.0) };
    }
}

// ─── terminal helpers ─────────────────────────────────────────────────────────
fn clear_screen() {
    print!("\x1b[2J\x1b[H");
    io::stdout().flush().ok();
}

const CTHULHU: &str = r#"
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣀⣤⣴⣶⣾⣿⣷⣾⣿⣷⣶⣦⣄⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣠⣤⣤⣀⠀⠀⠀⢀⣠⣶⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣷⣦⣄⠀⠀⠀⠀⣀⣤⣤⣄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢠⡾⠟⠻⣿⣿⣆⠀⢠⣾⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣷⡀⠀⣼⣿⣿⠟⠻⢷⡄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠀⠀⠀⣿⣿⡿⢀⣾⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣧⠀⣿⣿⣿⠀⠀⠀⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⢀⣀⣀⣀⣀⠀⠀⢀⣴⣿⣿⠃⣼⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣇⠘⣿⣿⣆⠀⠀⠀⣀⣀⣀⣀⡀⠀⠀⠀⠀⠀
⠀⠀⠀⢠⣶⣿⣿⣿⣿⣿⣷⣄⢿⣿⠟⠁⢰⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡄⠈⢻⣿⣷⣴⣾⣿⣿⣿⣿⣿⣦⡀⠀⠀⠀
⠀⠀⢰⡿⠋⠁⠀⢀⣨⣝⣿⣿⣷⡁⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣷⠀⠀⢩⣾⣿⣿⣫⣄⠀⠀⠈⠙⣿⡆⠀⠀
⠀⠀⢾⠃⠀⢀⣴⣿⣿⡿⠚⣿⣿⣷⠀⠘⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⣿⣿⣿⠱⢿⣿⣿⣦⡀⠀⠸⡇⠀⠀
⠀⠀⠀⢀⣴⣿⣿⡿⠋⠀⠀⣿⣿⣿⠀⠀⠈⢻⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠟⠁⠀⠀⣿⣿⣿⠀⠀⠙⢿⣿⣿⣦⡀⠀⠀⠀
⠀⠀⣰⣿⣿⡿⠋⠀⠀⠀⠀⣿⣿⡿⠀⠀⠀⠀⠙⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡿⠋⠀⠀⠀⠀⣿⣿⣿⠀⠀⠀⠀⠙⢿⣿⣿⣄⠀⠀
⠀⣼⣿⣿⠏⠀⠀⠀⠀⠀⣰⣿⣿⠇⣠⣾⡇⠀⠀⣮⡻⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⢟⣴⠀⠀⢸⣷⡄⢹⣿⣿⡄⠀⠀⠀⠀⠀⠻⣿⣿⣦⠀
⣸⣿⣿⠏⠀⠀⠀⠀⠀⢠⣿⣿⡟⢰⣿⣿⣿⣄⣀⣹⣿⣞⢿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡿⣵⣿⣇⣀⣤⣿⣿⣿⡄⢻⣿⣿⡄⠀⠀⠀⠀⠀⢹⣿⣿⡆
⣿⣿⣿⠀⠀⠀⠀⠀⣰⣿⣿⠟⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣷⡻⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⢏⣾⣿⣿⣿⣿⣿⣿⣿⣿⣷⠀⢻⣿⣿⣆⠀⠀⠀⠀⠀⣿⣿⣿
⣿⣿⣿⡀⠀⠀⢀⣼⣿⣿⠏⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣇⠀⠁⠘⢿⣿⣿⣿⣿⣿⣿⣿⣿⡿⠁⠉⠀⣹⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠻⣿⣿⣧⡀⠀⠀⢠⣿⣿⣿
⢹⣿⣿⣧⠀⣠⣾⣿⡿⠃⠀⠀⠀⠹⣿⣿⠟⠛⠛⣿⣿⣿⣦⣀⠀⠀⠻⣿⣿⣿⣿⣿⣿⠏⠀⠀⣀⣼⣿⣿⣿⠛⠛⠻⣿⣿⠃⠀⠀⠀⠙⢿⣿⣷⡄⢀⣾⣿⣿⠇
⠀⠻⣿⣿⣷⣽⣻⠟⠁⠀⠀⠀⠀⠀⠈⠁⠀⢀⣠⣿⣿⣿⣿⣿⣿⣶⣤⡙⣿⡿⢿⣿⢃⣤⣶⣿⣿⣿⣿⣿⣷⣄⡀⠀⠈⠁⠀⠀⠀⠀⠀⠈⠿⣟⣿⣿⣿⣿⠟⠀
⠀⠀⠈⣻⢿⣿⣿⣿⣷⣶⣶⣶⣶⣶⣶⣶⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡎⠁⠈⣱⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣷⣶⣶⣶⣶⣶⣶⣶⣾⣿⣿⣿⡿⣛⠁⠀⠀
⠀⠀⢸⣿⣷⡏⠙⠛⠿⠿⢿⣿⣿⣿⣿⣿⠿⠿⠿⠿⢿⣿⠿⣿⣿⣿⣿⣿⣿⠀⢀⣿⣿⣿⣿⣿⣿⠿⣿⠿⠿⠿⠿⢿⣿⣿⣿⣿⣿⡿⠿⠿⠛⠋⢵⣿⣿⡆⠀⠀
⠀⠀⣿⣿⣿⠃⠀⠀⢀⣀⣀⣀⡀⠀⠀⠀⠀⣠⣿⣿⣿⣿⣿⣷⡝⣿⣿⣿⣿⡇⢸⣿⣿⣿⣿⢫⣾⣿⣿⣿⣿⣿⣄⠀⠀⠀⠀⣀⣀⣀⣀⡀⠀⠀⠸⣿⣿⣷⠀⠀
⠀⠀⣿⣿⣿⠀⠀⢰⣿⡿⠿⣿⣿⣷⢄⣴⣿⣿⣿⣿⠟⠉⣵⣶⣶⡸⣿⣿⣿⡇⢸⣿⣿⣿⢃⣶⣶⡎⠙⠻⣿⣿⣿⣷⣤⡠⣾⣿⡿⠿⢿⣿⡄⠀⠀⣿⣿⣿⠀⠀
⠀⠀⢿⣿⣿⡀⠀⠸⡿⠀⠀⠀⢙⣵⣿⣿⣿⠟⠋⠀⠀⠀⣾⣿⣿⡇⣿⣿⣿⡇⢸⣿⣿⣿⢸⣿⣿⣧⠀⠀⠀⠙⠿⣿⣿⣿⣮⡋⠀⠀⠀⣿⠇⠀⢠⣿⣿⡟⠀⠀
⠀⠀⠸⣿⣿⣧⡀⠀⠀⠀⢀⣴⣿⣿⡿⣿⡅⠀⠀⠀⢀⣼⣿⣿⣿⠇⣿⣿⣿⠇⢸⣿⣿⣿⢸⣿⣿⣿⣆⠀⠀⠀⠀⣨⣻⣿⣿⣿⣦⠀⠀⠀⠀⢀⣾⣿⣿⠃⠀⠀
⠀⠀⠀⠹⣿⣿⣷⣤⡀⢠⣿⣿⣿⠟⢿⣿⣿⠀⣀⣴⣿⣿⣿⠟⠁⠀⣿⣿⣿⠀⠈⣿⣿⣿⠀⠈⠻⣿⣿⣷⣦⣀⠀⣿⣿⡎⠻⣿⣿⣷⡄⢀⣴⣿⣿⣿⠃⠀⠀⠀
⠀⠀⠀⠀⠈⠻⣿⣿⣿⣷⣾⣿⣥⣤⢸⣿⣿⣸⣿⣿⡿⠋⠁⠀⠀⢀⣿⣿⡟⠀⠀⣿⣿⣿⠀⠀⠀⠈⠻⢿⣿⣿⣷⣿⣿⣧⣤⣭⣿⣷⣿⣿⣿⡿⠟⠁⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠉⢛⣻⣿⣿⣿⣿⢼⣿⣿⡟⠛⠉⠀⢀⣀⣤⣄⣸⣿⣿⠇⠀⠀⢸⣿⣿⡇⣤⣤⣀⡀⠀⠉⠛⢻⣿⣿⣿⣿⣿⣿⣿⣟⡛⠉⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠸⣿⣿⣷⡀⠀⠸⣿⣿⡇⣀⣴⣾⣿⣿⣿⣏⣿⣿⡿⠀⠀⠀⠀⣿⣿⣿⢻⣿⣿⣿⣷⣦⡀⢸⣿⣿⠁⠀⢀⣾⣿⣿⠇⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠻⣿⣿⣿⣦⣤⣿⣿⣾⣿⣿⠿⠋⠁⢀⣾⣿⣿⢳⣷⠀⢀⣾⡼⣿⣿⣧⡀⠈⠙⠿⣿⣿⣷⣯⣯⣤⣴⣿⣿⣿⠏⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠙⠿⣿⣿⣿⣿⣿⣟⣯⣅⣀⣠⣴⣿⣿⣿⠃⢻⣿⠆⢸⣿⡏⠹⣿⣿⣿⣦⣄⣀⣨⣽⣻⣿⣿⣿⣿⣿⠟⠃⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠙⢿⣿⣿⣿⣿⣿⣿⠟⠁⠀⣼⡿⠀⠈⣿⣇⠀⠈⠻⣿⣿⣿⣿⣿⣿⡿⠃⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠉⠙⠉⠉⠀⠀⣠⣾⠿⠁⠀⠀⠘⠿⣷⡄⠀⠀⠉⠉⠋⠉⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
"#;

fn print_splash() {
    clear_screen();
    println!("{RED}{DIM}{CTHULHU}{RST}");
    println!("{DIM}        Ph'nglui mglw'nafh Cthulhu R'lyeh wgah'nagl fhtagn.");
    println!("              In his house at R'lyeh, dead Cthulhu waits.");
    println!("                  And so does your segmentation fault.{RST}");
    println!("\n{DIM}  Press any key to begin…{RST}");
    io::stdout().flush().ok();
}

fn print_menu() {
    clear_screen();
    println!();
    println!("{RED}{BOLD}  ╔═════════════════════════════════════════════════╗");
    println!("  ║             ☠   pointer-horror   ☠              ║");
    println!("  ╚═════════════════════════════════════════════════╝{RST}");

    let mut prev_col = "";
    for d in DEMOS {
        if d.col != prev_col {
            if !prev_col.is_empty() {
                println!();
            }
            prev_col = d.col;
        }
        let note = d
            .note
            .map(|n| format!("  {DIM}({n}){RST}"))
            .unwrap_or_default();
        println!(
            "    {CYN}{key}){RST}  {col}{label}{RST}{note}",
            key = d.key,
            col = d.col,
            label = d.label
        );
    }

    println!("\n    {DIM}q) quit{RST}\n");
    print!("    {BOLD}> {RST}");
    io::stdout().flush().ok();
}

// ─── key reading ─────────────────────────────────────────────────────────────
fn read_key() -> Res<char> {
    let _raw = RawMode::enable()?;
    loop {
        if let Event::Key(KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            ..
        }) = event::read()?
        {
            return Ok(match code {
                KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => '\x03',
                KeyCode::Char(c) => c,
                KeyCode::Esc => '\x1b',
                KeyCode::Enter => '\n',
                _ => continue,
            });
        }
    }
}

fn press_any_key() {
    print!("\n  {DIM}Done. Press any key to return to the menu…{RST} ");
    io::stdout().flush().ok();
    let _raw = RawMode::enable().ok();
    loop {
        match event::read() {
            Ok(Event::Key(KeyEvent {
                kind: KeyEventKind::Press,
                ..
            })) => break,
            Ok(_) => continue, // ignore mouse / resize events
            Err(_) => break,
        }
    }
    println!();
}

// ─── precompile ───────────────────────────────────────────────────────────────
// Build all C demos at startup so the first run of each is instant.
fn precompile_c() {
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    let spinner = thread::spawn(move || {
        let mut i = 0usize;
        while running_clone.load(Ordering::Relaxed) {
            print!(
                "\r  {DIM}{}{RST}  {DIM}Compiling C demos…{RST}   ",
                SPIN_FRAMES[i % SPIN_FRAMES.len()]
            );
            io::stdout().flush().ok();
            i += 1;
            thread::sleep(Duration::from_millis(80));
        }
    });

    let result = Command::new("make")
        .arg("all")
        .current_dir(demo_dir("c"))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .output();

    running.store(false, Ordering::Relaxed);
    let _ = spinner.join();

    match result {
        Ok(out) if out.status.success() => {
            println!("\r  {GRN}✓{RST}  C demos ready.                           ");
        }
        _ => {
            // Non-fatal: on-demand build in the menu loop will surface the error.
            println!("\r  {YLW}⚠{RST}  C pre-build failed — will retry on demand.");
        }
    }
}

const SPIN_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

fn build_demo(demo: &Demo) -> bool {
    let Some((prog, args, subdir)) = demo.build else {
        return true;
    };

    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    let spinner = thread::spawn(move || {
        let mut i = 0usize;
        while running_clone.load(Ordering::Relaxed) {
            print!(
                "\r  {GRN}{}{RST}  {DIM}Building…{RST}   ",
                SPIN_FRAMES[i % SPIN_FRAMES.len()]
            );
            io::stdout().flush().ok();
            i += 1;
            thread::sleep(Duration::from_millis(80));
        }
    });

    let result = Command::new(prog)
        .args(args)
        .current_dir(demo_dir(subdir))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output();

    running.store(false, Ordering::Relaxed);
    let _ = spinner.join();

    match result {
        Ok(out) if out.status.success() => {
            println!("\r  {GRN}✓{RST}  Built.                              ");
            true
        }
        Ok(out) => {
            println!("\r  {RED}✗{RST}  Build failed.                       \n");
            io::stderr().write_all(&out.stderr).ok();
            println!();
            false
        }
        Err(e) => {
            println!("\r  {RED}✗{RST}  Build error: {e}");
            false
        }
    }
}

// ─── run ──────────────────────────────────────────────────────────────────────
fn run_demo(demo: &Demo) {
    clear_screen();
    println!(
        "\n  {DIM}▶  {col}{label}{RST}{DIM}…{RST}\n",
        col = demo.col,
        label = demo.label
    );
    println!("{DIM}{}{RST}\n", "─".repeat(60));

    let (prog, args, subdir) = demo.run;

    // Install a no-op SIGINT handler in the parent so Ctrl-C during the demo
    // doesn't kill the launcher.  The guard restores the previous disposition
    // when it drops.  pre_exec resets SIGINT to SIG_DFL inside the child
    // before exec, so the child inherits the default (not our no-op).
    let _sig = SigintGuard::noop();

    use std::os::unix::process::CommandExt;
    let status = unsafe {
        Command::new(prog)
            .args(args)
            .current_dir(demo_dir(subdir))
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .pre_exec(|| {
                libc::signal(libc::SIGINT, libc::SIG_DFL);
                Ok(())
            })
            .spawn()
    }
    .and_then(|mut c| c.wait());

    // _sig drops here, restoring the previous SIGINT disposition.

    println!("\n{DIM}{}{RST}", "─".repeat(60));
    if let Err(e) = status {
        println!("  {RED}Failed to run: {e}{RST}");
    }
}

// ─── main ─────────────────────────────────────────────────────────────────────
fn main() {
    if !io::stdin().is_terminal() {
        eprintln!(
            "{RED}Error:{RST} stdin must be a TTY. \
             Run this binary in an interactive terminal."
        );
        std::process::exit(1);
    }

    print_splash();
    let _ = read_key();
    precompile_c();

    loop {
        print_menu();

        let key = match read_key() {
            Ok(k) => k,
            Err(e) => {
                eprintln!("{RED}Error reading input: {e}{RST}");
                break;
            }
        };

        // Echo the key, then newline.
        if key != '\r' && key != '\n' {
            println!("{key}\n");
        } else {
            println!();
        }

        if key == '\x03' || key == 'q' || key == 'Q' {
            println!("\n  {DIM}Goodbye.\n{RST}");
            break;
        }

        if key == '\x1b' {
            continue;
        }

        let Some(demo) = DEMOS.iter().find(|d| d.key == key) else {
            continue;
        };

        println!(
            "\n  {YLW}→{RST}  {col}{label}{RST}\n",
            col = demo.col,
            label = demo.label
        );

        if build_demo(demo) {
            println!("Running demo");
            run_demo(demo);
        }
        press_any_key();
    }
}

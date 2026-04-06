use std::{
    env,
    io::{self, Write},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use crossterm::{
    event::{poll, read, Event, KeyCode},
    style::Color,
    terminal, Result as CTResult,
};
use getopts::Options;
use nanorand::{Rng, WyRand};
use signal_hook::{consts::TERM_SIGNALS, flag::register};

mod matrix;
mod screen;
use matrix::*;
use screen::RawScreen;

static BOW_COLORS: [Color; 6] = [
    Color::Green,
    Color::Red,
    Color::Blue,
    Color::Cyan,
    Color::Yellow,
    Color::Magenta,
];
const ANSI_RESET: &str = "\x1b[0m";

fn main() -> CTResult<()> {
    let term_flag = Arc::new(AtomicBool::new(false));
    for &sig in TERM_SIGNALS {
        register(sig, term_flag.clone()).unwrap();
    }

    let args: Vec<_> = env::args().collect();
    let mut opts = Options::new();
    opts.optopt("s", "speed", "character drop speed [default 4](0-9)", "");
    opts.optopt(
        "c",
        "color",
        "main character color [default Green](Green,Red,Blue,Cyan,Yellow,Magenta)",
        "",
    );
    opts.optflag("r", "rainbow", "rainbow color matrix");
    opts.optflag("h", "help", "show this help");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(e) => panic!("{}", e.to_string()),
    };
    if matches.opt_present("h") {
        print_usage(opts);
        return Ok(());
    }
    let mut speed: u64 = matches
        .opt_str("s")
        .unwrap_or("4".to_string())
        .parse()
        .unwrap();
    let mut rainbow = matches.opt_present("r");
    let color = matches.opt_str("c").unwrap_or("Green".to_string());
    let mut color = if color.eq_ignore_ascii_case("Red") {
        Color::Red
    } else if color.eq_ignore_ascii_case("Blue") {
        Color::Blue
    } else if color.eq_ignore_ascii_case("Cyan") {
        Color::Cyan
    } else if color.eq_ignore_ascii_case("Magenta") {
        Color::Magenta
    } else {
        Color::Green
    };

    let (cols, rows) = terminal::size()?;
    let mut normal = String::new();
    let mut frame_buf = String::new();
    let mut matrix = Matrix::new(cols as usize, rows as usize);
    let (mut char_val, mut next);

    let _screen = RawScreen::new()?;
    let mut stdout = io::stdout();
    let mut rng = WyRand::new();
    let mut rainbow_bits = 0u64;
    let mut rainbow_bits_left = 0u8;
    while !term_flag.load(Ordering::Relaxed) {
        next = matrix.next();
        normal.clear();
        frame_buf.clear();
        frame_buf.push_str("\x1b[H");
        let mut active_color: Option<Color> = None;
        for item in next {
            char_val = item.val as char;
            if !rainbow {
                if item.val == EMPTY || item.val == BLANK {
                    normal.push(' ');
                } else if item.is_head() {
                    push_run_with_color(&mut frame_buf, &mut active_color, color, &normal);
                    normal.clear();
                    reset_color(&mut frame_buf, &mut active_color);
                    frame_buf.push(char_val);
                } else {
                    normal.push(char_val);
                }
            } else {
                color = BOW_COLORS
                    [next_rainbow_color_idx(&mut rng, &mut rainbow_bits, &mut rainbow_bits_left)];
                if item.val == EMPTY || item.val == BLANK {
                    reset_color(&mut frame_buf, &mut active_color);
                    frame_buf.push(' ');
                } else if item.is_head() {
                    reset_color(&mut frame_buf, &mut active_color);
                    frame_buf.push(char_val);
                } else {
                    set_color(&mut frame_buf, &mut active_color, color);
                    frame_buf.push(char_val);
                }
            }
        }
        if !rainbow {
            push_run_with_color(&mut frame_buf, &mut active_color, color, &normal);
        }
        reset_color(&mut frame_buf, &mut active_color);
        stdout.write_all(frame_buf.as_bytes())?;
        stdout.flush()?;

        let delay_ms = (10 * speed).max(1);
        if poll(Duration::from_millis(delay_ms))? {
            match read()? {
                Event::Key(e) => match e.code {
                    KeyCode::Char('q' | 'Q') => break,
                    KeyCode::Char('0'..='9') => {
                        if let KeyCode::Char(ch) = e.code {
                            speed = (ch as u8 - b'0') as u64;
                        }
                    }
                    KeyCode::Char('g' | 'G') => {
                        color = Color::Green;
                        rainbow = false;
                    }
                    KeyCode::Char('r' | 'R') => {
                        color = Color::Red;
                        rainbow = false;
                    }
                    KeyCode::Char('b' | 'B') => {
                        color = Color::Blue;
                        rainbow = false;
                    }
                    KeyCode::Char('c' | 'C') => {
                        color = Color::Cyan;
                        rainbow = false;
                    }
                    KeyCode::Char('y' | 'Y') => {
                        color = Color::Yellow;
                        rainbow = false;
                    }
                    KeyCode::Char('m' | 'M') => {
                        color = Color::Magenta;
                        rainbow = false;
                    }
                    KeyCode::Char('w' | 'W') => rainbow = true,
                    KeyCode::Char('d' | 'D') => {
                        color = Color::Green;
                        speed = 4;
                        rainbow = false;
                    }
                    _ => {}
                },
                Event::Resize(cols, rows) => matrix = Matrix::new(cols as usize, rows as usize),
                _ => {}
            }
        }
    }
    Ok(())
}

#[inline]
fn push_run_with_color(
    frame_buf: &mut String,
    active_color: &mut Option<Color>,
    color: Color,
    text: &str,
) {
    if text.is_empty() {
        return;
    }
    set_color(frame_buf, active_color, color);
    frame_buf.push_str(text);
}

#[inline]
fn set_color(frame_buf: &mut String, active_color: &mut Option<Color>, color: Color) {
    if *active_color == Some(color) {
        return;
    }
    frame_buf.push_str(color_ansi(color));
    *active_color = Some(color);
}

#[inline]
fn reset_color(frame_buf: &mut String, active_color: &mut Option<Color>) {
    if active_color.is_some() {
        frame_buf.push_str(ANSI_RESET);
        *active_color = None;
    }
}

#[inline]
fn color_ansi(color: Color) -> &'static str {
    match color {
        Color::Green => "\x1b[32m",
        Color::Red => "\x1b[31m",
        Color::Blue => "\x1b[34m",
        Color::Cyan => "\x1b[36m",
        Color::Yellow => "\x1b[33m",
        Color::Magenta => "\x1b[35m",
        _ => "\x1b[32m",
    }
}

#[inline]
fn next_rainbow_color_idx(rng: &mut WyRand, bit_pool: &mut u64, bits_left: &mut u8) -> usize {
    loop {
        if *bits_left < 3 {
            *bit_pool = rng.generate::<u64>();
            *bits_left = 64;
        }
        let idx = (*bit_pool & 0b111) as usize;
        *bit_pool >>= 3;
        *bits_left -= 3;
        if idx < BOW_COLORS.len() {
            return idx;
        }
    }
}

fn print_usage(opts: Options) {
    let brief = format!(
        "{} {}\nUsage: {} [options]\n{}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_NAME"),
        format!(
            "{}\n{}\n{}\n{}\n{}",
            "Press 'q' or 'Q' to quit.",
            "Press the initial to change the color (like 'g' to green).",
            "Press number to change speed.",
            "Press 'w' or 'W' to set rainbow mode.",
            "Press 'd' back to default."
        )
    );
    println!("{}", opts.usage(&brief));
}

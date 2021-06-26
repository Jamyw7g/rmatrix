use std::{env, io, panic::set_hook, time::Duration};

use crossterm::{
    cursor::{Hide, MoveToColumn, MoveToRow, Show},
    event::{poll, read, Event, KeyCode},
    execute,
    style::{Color, Stylize},
    terminal::{
        self, disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
    },
    Result as CTResult,
};
use getopts::Options;
use rand::Rng;

mod matrix;
use matrix::*;

static BOW_COLORS: [Color; 6] = [
    Color::Green,
    Color::Red,
    Color::Blue,
    Color::Cyan,
    Color::Yellow,
    Color::Magenta,
];

fn main() -> CTResult<()> {
    set_hook(Box::new(|info| {
        reset().unwrap();

        if let Some(location) = info.location() {
            print!(
                "Panic in File `{}` at Ln {}, Col {}: ",
                location.file(),
                location.line(),
                location.column()
            );
        }

        if let Some(s) = info.payload().downcast_ref::<String>() {
            println!("{}", s);
        } else {
            println!();
        }
    }));

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
    let mut matrix = Matrix::new(cols as usize, rows as usize);
    let (mut char_val, mut next);

    init()?;
    let mut stdout = io::stdout();
    let mut rng = rand::thread_rng();
    loop {
        next = matrix.next();
        normal.clear();
        execute!(stdout, MoveToRow(0), MoveToColumn(0))?;
        for item in next {
            char_val = item.val as u8 as char;
            if !rainbow {
                if item.val == -1 || item.val == BLANK {
                    normal.push(' ');
                } else if item.is_head {
                    print!("{}{}", normal.clone().with(color), char_val);
                    normal.clear();
                } else {
                    normal.push(char_val);
                }
            } else {
                color = BOW_COLORS[rng.gen_range(0..6)];
                if item.val == -1 || item.val == BLANK {
                    print!(" ");
                } else if item.is_head {
                    print!("{}", char_val);
                } else {
                    print!("{}", char_val.with(color));
                }
            }
        }
        if !rainbow {
            print!("{}", normal.clone().with(color));
        }

        if poll(Duration::from_millis(10 * speed))? {
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

    reset()
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

fn init() -> CTResult<()> {
    execute!(io::stdout(), EnterAlternateScreen, Hide)?;
    enable_raw_mode()
}

fn reset() -> CTResult<()> {
    execute!(io::stdout(), Show, LeaveAlternateScreen)?;
    disable_raw_mode()
}

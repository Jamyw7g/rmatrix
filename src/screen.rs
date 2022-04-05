use std::io;
use crossterm::execute;
use crossterm::terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::cursor::{Show, Hide};

pub struct RawScreen;

impl RawScreen {
    pub fn new() -> crossterm::Result<Self> {
        execute!(io::stdout(), EnterAlternateScreen, Hide)?;
        enable_raw_mode()?;
        Ok(Self)
    } 
}

impl Drop for RawScreen {
    fn drop(&mut self) {
        disable_raw_mode().unwrap();
        execute!(io::stdout(), Show, LeaveAlternateScreen).unwrap();
    }
}
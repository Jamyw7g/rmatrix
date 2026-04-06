use crossterm::cursor::{Hide, Show};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use std::io;

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

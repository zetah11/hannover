mod poll;

pub use poll::{InputPoller, WavetablePoller};

use std::io::{stdout, Stdout, Write};
use std::time::Duration;

use bresenham::Bresenham;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::style::{self, Color};
use crossterm::terminal;
use crossterm::ExecutableCommand;
use crossterm::{cursor, QueueableCommand};
use itertools::Itertools;
use single_value_channel::Updater;

pub const WT_VIZ_WIDTH: usize = 48;
pub const WT_VIZ_HEIGHT: usize = 8;
pub const WT_LETTERS: [char; 16] = [
    ' ', '.', '.', '_', '\'', '|', '/', 'j', '\'', '\\', '|', 'L', '^', '\\', '/', '#',
];

#[derive(Debug)]
pub enum GuiError {
    /// The program was interrupted with Ctrl-C.
    Interrupted,
    Crossterm(crossterm::ErrorKind),
}

impl From<crossterm::ErrorKind> for GuiError {
    fn from(value: crossterm::ErrorKind) -> Self {
        Self::Crossterm(value)
    }
}

impl std::fmt::Display for GuiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Interrupted => write!(f, "interrupted by user"),
            Self::Crossterm(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for GuiError {}

pub struct Gui {
    text: String,
    cursor: usize,
    max_cursor: usize,
    send: Updater<String>,
    recv: WavetablePoller,

    wt: [[char; WT_VIZ_WIDTH]; WT_VIZ_HEIGHT],
}

impl Gui {
    pub fn run(send: Updater<String>, recv: WavetablePoller) -> Result<(), GuiError> {
        terminal::enable_raw_mode()?;
        let result = Self::event_loop(send, recv);
        terminal::disable_raw_mode()?;

        result
    }

    fn event_loop(send: Updater<String>, recv: WavetablePoller) -> Result<(), GuiError> {
        let mut this = Self {
            text: String::new(),
            cursor: 0,
            max_cursor: 0,
            send,
            recv,

            wt: [[' '; WT_VIZ_WIDTH]; WT_VIZ_HEIGHT],
        };

        let mut stdout = stdout();

        stdout
            .execute(style::SetForegroundColor(Color::Green))?
            .execute(style::Print("> "))?
            .execute(style::ResetColor)?;

        loop {
            match this.handle_event() {
                Ok(true) => break,
                Ok(false) => {}
                Err(GuiError::Interrupted) => {
                    this.unrender(&mut stdout)?;
                    stdout.flush()?;

                    return Err(GuiError::Interrupted);
                }
                Err(e) => return Err(e),
            }

            this.max_cursor = this.max_cursor.max(this.cursor);
            this.update()?;
            this.render(&mut stdout)?;
            if this.send.update(this.text.clone()).is_err() {
                break;
            }
        }

        this.unrender(&mut stdout)?;
        stdout.flush()?;
        Ok(())
    }

    fn handle_event(&mut self) -> Result<bool, GuiError> {
        if !event::poll(Duration::from_millis(1000 / 15))? {
            return Ok(false);
        }

        match event::read()? {
            Event::Key(KeyEvent {
                code, modifiers, ..
            }) => match code {
                KeyCode::Backspace => {
                    if let Some((index, _)) = self
                        .text
                        .char_indices()
                        .filter(|(index, _)| *index < self.cursor)
                        .last()
                    {
                        self.text.remove(index);
                        self.cursor = index;
                    }
                }

                KeyCode::Enter => {
                    return Ok(true);
                }

                KeyCode::Left => {
                    let c = self
                        .text
                        .get(..self.cursor)
                        .and_then(|s| s.chars().last())
                        .map(|c| c.len_utf8())
                        .unwrap_or(0);
                    self.cursor = self.cursor.wrapping_sub(c);
                }

                KeyCode::Right => {
                    let c = self
                        .text
                        .get(self.cursor..)
                        .and_then(|s| s.chars().next())
                        .map(|c| c.len_utf8())
                        .unwrap_or(0);
                    self.cursor = self.cursor.wrapping_add(c)
                }

                KeyCode::Home => self.cursor = 0,
                KeyCode::End => self.cursor = self.text.len(),

                KeyCode::Tab => {
                    self.text.insert(self.cursor, '\t');
                    self.cursor += '\t'.len_utf8();
                }

                KeyCode::Delete => {
                    if self.cursor < self.text.len() {
                        self.text.remove(self.cursor);
                    }
                }

                KeyCode::Char(c) => {
                    let c: String = if modifiers.contains(KeyModifiers::SHIFT) {
                        c.to_uppercase().collect()
                    } else if modifiers.contains(KeyModifiers::CONTROL) && c == 'c' {
                        return Err(GuiError::Interrupted);
                    } else {
                        String::from(c)
                    };

                    self.text.insert_str(self.cursor, &c);
                    self.cursor += c.len();
                }

                KeyCode::Esc => {}
                KeyCode::F(_) => {}
                KeyCode::BackTab | KeyCode::Insert => {}
                KeyCode::CapsLock | KeyCode::ScrollLock | KeyCode::NumLock => {}
                KeyCode::Up | KeyCode::Down | KeyCode::PageUp | KeyCode::PageDown => {}
                KeyCode::Null | KeyCode::PrintScreen | KeyCode::Pause | KeyCode::Menu => {}
                KeyCode::KeypadBegin | KeyCode::Media(_) | KeyCode::Modifier(_) => {}
            },

            Event::Paste(data) => {
                self.text.push_str(&data);
            }

            _ => {}
        }

        Ok(false)
    }

    fn unrender(&self, stdout: &mut Stdout) -> Result<(), GuiError> {
        let clear: String = (0..self.max_cursor).map(|_| ' ').collect();
        stdout.queue(style::Print(clear))?;

        let clear: String = (0..WT_VIZ_WIDTH).map(|_| ' ').collect();
        for _ in 0..WT_VIZ_HEIGHT {
            stdout
                .queue(cursor::MoveToNextLine(1))?
                .queue(style::Print(&clear))?;
        }

        stdout
            .queue(cursor::MoveToPreviousLine(WT_VIZ_HEIGHT as u16))?
            .flush()?;

        Ok(())
    }

    fn render(&self, stdout: &mut Stdout) -> Result<(), GuiError> {
        // draw text
        stdout
            .queue(cursor::MoveToColumn(2))?
            .queue(style::Print(&self.text))?;

        for _ in self.cursor..self.max_cursor {
            stdout.queue(style::Print(" "))?;
        }

        let mut newlines = 1;
        stdout.queue(cursor::MoveToNextLine(1))?;

        // draw wavetable
        for line in self.wt {
            newlines += 1;
            stdout
                .queue(style::Print(line.into_iter().collect::<String>()))?
                .queue(cursor::MoveToNextLine(1))?;
        }

        // reset cursor
        let column = 2 + self
            .text
            .char_indices()
            .enumerate()
            .filter(|(_, (byte, _))| *byte < self.cursor)
            .map(|(index, _)| index + 1)
            .last()
            .unwrap_or(0);

        stdout
            .queue(cursor::MoveToPreviousLine(newlines))?
            .queue(cursor::MoveToColumn(column as u16))?
            .flush()?;

        Ok(())
    }

    fn update(&mut self) -> Result<(), GuiError> {
        if let Some(wt) = self.recv.poll() {
            // create a "high-res" image, and downsample to appropriate letters.
            let wt = draw_wavetable(wt);
            for (y, row) in self.wt.iter_mut().enumerate() {
                for (x, v) in row.iter_mut().enumerate() {
                    let a = wt[2 * y][2 * x];
                    let b = wt[2 * y][2 * x + 1];
                    let c = wt[2 * y + 1][2 * x];
                    let d = wt[2 * y + 1][2 * x + 1];

                    let i = (a as u8) << 3 | (b as u8) << 2 | (c as u8) << 1 | (d as u8);
                    *v = WT_LETTERS[i as usize];
                }
            }
        }

        Ok(())
    }
}

fn draw_wavetable(wt: &[u8]) -> [[bool; 2 * WT_VIZ_WIDTH]; 2 * WT_VIZ_HEIGHT] {
    let n = wt.len() as f64;
    let width = (2 * WT_VIZ_WIDTH) as f64;
    let height = (2 * WT_VIZ_HEIGHT) as f64;

    let mut res = [[false; 2 * WT_VIZ_WIDTH]; 2 * WT_VIZ_HEIGHT];

    for ((start_ndx, start), (end_ndx, end)) in wt.iter().enumerate().tuple_windows() {
        let start_x = ((start_ndx as f64 / n) * width) as isize;
        let end_x = (end_ndx as f64 / n * width) as isize;

        let start_y = (((255 - *start) as f64 / 256.0) * height) as isize;
        let end_y = (((255 - *end) as f64 / 256.0) * height) as isize;

        for (x, y) in Bresenham::new((start_x, start_y), (end_x, end_y)) {
            res[y as usize][x as usize] = true;
        }
    }

    res
}

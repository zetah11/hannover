use std::io::{stdout, Stdout, Write};

use anyhow::anyhow;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::style::{self, Color};
use crossterm::terminal;
use crossterm::ExecutableCommand;
use crossterm::{cursor, QueueableCommand};

pub struct Gui {
    text: String,
    cursor: usize,
    max_cursor: usize,
}

impl Gui {
    pub fn run() -> anyhow::Result<String> {
        terminal::enable_raw_mode()?;
        let result = Self::event_loop();
        terminal::disable_raw_mode()?;

        result
    }

    fn event_loop() -> anyhow::Result<String> {
        let mut this = Self {
            text: String::new(),
            cursor: 0,
            max_cursor: 0,
        };

        let mut stdout = stdout();

        stdout
            .execute(style::SetForegroundColor(Color::Green))?
            .execute(style::Print("> "))?
            .execute(style::ResetColor)?;

        loop {
            match this.handle_event(&mut stdout) {
                Ok(true) => break,
                Ok(false) => {}
                Err(e) => {
                    terminal::disable_raw_mode();
                    return Err(e);
                }
            }

            this.max_cursor = this.max_cursor.max(this.cursor);

            stdout
                .queue(cursor::MoveToColumn(2))?
                .queue(style::Print(&this.text))?;

            for _ in this.cursor..this.max_cursor {
                stdout.queue(style::Print(" "))?;
            }

            stdout
                .queue(cursor::MoveToColumn(2 + this.cursor as u16))?
                .flush()?;
        }

        stdout.flush()?;
        Ok(this.text)
    }

    fn handle_event(&mut self, stdout: &mut Stdout) -> anyhow::Result<bool> {
        match event::read()? {
            Event::Key(KeyEvent {
                code, modifiers, ..
            }) => match code {
                KeyCode::Backspace => {
                    if let Some((index, _)) = self
                        .text
                        .char_indices()
                        .filter(|(index, c)| *index < self.cursor)
                        .last()
                    {
                        self.text.remove(index);
                        self.cursor = index;
                    }
                }

                KeyCode::Enter => {
                    stdout.execute(cursor::MoveToNextLine(1))?;
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
                        return Err(anyhow!("Interrupted"));
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
}

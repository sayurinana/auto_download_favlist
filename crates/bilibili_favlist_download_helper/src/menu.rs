use std::io::{stdout, Write};
use std::time::Duration;

use anyhow::Result;
use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{poll, read, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::terminal::{self, ClearType};
use crossterm::{execute, QueueableCommand};

pub enum MenuOutcome {
    Selected(usize),
    Esc,
}

pub fn select_from_menu(title: &str, options: &[String]) -> Result<MenuOutcome> {
    if options.is_empty() {
        return Ok(MenuOutcome::Esc);
    }
    terminal::enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, Hide)?;
    clear_pending_events()?;
    let mut index = 0usize;

    loop {
        redraw(&mut stdout, title, options, index)?;
        if let Event::Key(key) = read()? {
            match normalize_key(key) {
                Some(NormalizedKey::Up) => {
                    if index == 0 {
                        index = options.len() - 1;
                    } else {
                        index -= 1;
                    }
                }
                Some(NormalizedKey::Down) => {
                    index = (index + 1) % options.len();
                }
                Some(NormalizedKey::Confirm) => {
                    cleanup(stdout)?;
                    return Ok(MenuOutcome::Selected(index));
                }
                Some(NormalizedKey::Esc) => {
                    cleanup(stdout)?;
                    return Ok(MenuOutcome::Esc);
                }
                None => {}
            }
        }
    }
}

fn redraw(
    stdout: &mut std::io::Stdout,
    title: &str,
    options: &[String],
    index: usize,
) -> Result<()> {
    stdout.queue(MoveTo(0, 0))?;
    stdout.queue(terminal::Clear(ClearType::All))?;
    writeln!(stdout, "{}", title)?;
    writeln!(stdout)?;
    for (i, option) in options.iter().enumerate() {
        if i == index {
            writeln!(stdout, "> {}", option)?;
        } else {
            writeln!(stdout, "  {}", option)?;
        }
    }
    stdout.flush()?;
    Ok(())
}

fn cleanup(mut stdout: std::io::Stdout) -> Result<()> {
    execute!(stdout, Show)?;
    terminal::disable_raw_mode()?;
    Ok(())
}

enum NormalizedKey {
    Up,
    Down,
    Confirm,
    Esc,
}

fn normalize_key(key: KeyEvent) -> Option<NormalizedKey> {
    if key.kind != KeyEventKind::Press {
        return None;
    }
    if key.modifiers != KeyModifiers::NONE {
        return None;
    }
    match key.code {
        KeyCode::Up | KeyCode::Left => Some(NormalizedKey::Up),
        KeyCode::Down | KeyCode::Right => Some(NormalizedKey::Down),
        KeyCode::Enter | KeyCode::Char(' ') => Some(NormalizedKey::Confirm),
        KeyCode::Esc => Some(NormalizedKey::Esc),
        KeyCode::Char(c) => match c {
            'w' | 'W' => Some(NormalizedKey::Up),
            's' | 'S' => Some(NormalizedKey::Down),
            'a' | 'A' => Some(NormalizedKey::Up),
            'd' | 'D' => Some(NormalizedKey::Down),
            _ => None,
        },
        _ => None,
    }
}

fn clear_pending_events() -> Result<()> {
    while poll(Duration::from_millis(0))? {
        let _ = read()?;
    }
    Ok(())
}

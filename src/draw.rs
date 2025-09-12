use crate::{
    chain::{Chain, ChainLink},
    element::{BasicElement, ElementType},
    error::EASHError,
    evaluate::{TokenType, tokenize},
    misc_types::{Alignment, Width},
};

use crossterm::{
    cursor::MoveToColumn,
    queue,
    style::{
        Color as ctColor, Print, PrintStyledContent, ResetColor, SetBackgroundColor,
        SetForegroundColor, Stylize,
    },
    terminal::{Clear, ClearType},
};

use std::{io::Write, sync::MutexGuard};

// returns string with padding, content start & content end
pub fn pad_string(original: String, size: u16, aligment: &Alignment) -> (String, usize, usize) {
    let mut s = original;
    if s.len() >= size as usize {
        let len = s.len();
        return (s, 0, len);
    }

    let difference = size as usize - s.len();
    let start;
    let end;
    match aligment {
        Alignment::Left => {
            start = 0;
            end = s.len();
            s = format!("{}{}", s, " ".repeat(difference as usize));
        }
        Alignment::Center => {
            let l = difference / 2;
            let r = difference - l;
            s = format!("{}{}{}", " ".repeat(l as usize), s, "".repeat(r as usize));
            start = l;
            end = s.len() - r;
        }
        Alignment::Right => {
            s = format!("{}{}", " ".repeat(difference as usize), s);
            start = difference;
            end = s.len();
        }
    };

    (s, start, end)
}

pub fn draw_flat_basic_element<W: Write>(
    w: &mut W,
    item: &ChainLink,
    e: &BasicElement,
    content: String,
) -> Result<(), EASHError> {
    let mut print = &content[0..];
    // cut the string off if its behind the first terminal character
    if item.mass.position.round() < 0.0 {
        let difference = item.mass.position.round().abs() as u16;
        // don't print the string
        if difference >= print.len() as u16 {
            return Ok(());
        }

        print = &content[difference as usize..];
    }

    let styled = print
        .stylize()
        .with(e.visual_state.color.to_flat_color()?)
        .on(e.visual_state.bg_color.to_flat_color()?);
    queue!(w, PrintStyledContent(styled))?;
    return Ok(());
}

// we need it to be mutable to set the width property on mass
// TODO)) split this function up
pub fn draw<'a, W: Write + Send>(
    w: &mut W,
    elements: &mut MutexGuard<Chain>,
) -> Result<(), EASHError> {
    _ = queue!(w, MoveToColumn(0), Clear(ClearType::CurrentLine));

    let mut cursor_position = 0;
    for item in elements.links.iter_mut() {
        let position = item.mass.position.round() as u16;
        queue!(w, MoveToColumn(position))?;

        // draw each element based on its enum 😨😨😨
        match &item.element {
            ElementType::BasicElement(e) => {
                // add spacing
                let mut print = format!(
                    "{}{}{}",
                    " ".repeat(e.visual_state.padding as usize),
                    e.content,
                    " ".repeat(e.visual_state.padding as usize)
                );

                // pad string if too small, cut it if its too big.
                let (mut start, mut end);
                match e.visual_state.width {
                    Width::Minimum(m) => {
                        (print, start, end) = pad_string(print, m as u16, &e.visual_state.align);
                    }
                    Width::Units(u) => {
                        (print, start, end) = pad_string(print, u as u16, &e.visual_state.align);
                    }
                }

                start += e.visual_state.padding as usize;
                end -= e.visual_state.padding as usize;

                item.mass.width = print.len() as u16;

                // style & print element as required (character at a time if its a gradient)
                if e.visual_state.bg_color.is_gradient() || e.visual_state.color.is_gradient() {
                    let fg = e.visual_state.color.to_color_for_char(0.0);
                    let bg = e.visual_state.bg_color.to_color_for_char(0.0);
                    queue!(w, SetBackgroundColor(bg), SetForegroundColor(fg))?;

                    for (i, character) in print.chars().enumerate() {
                        let mut character = character.to_string().stylize();
                        if i >= start && i <= end && e.visual_state.color.is_gradient() {
                            let color = e
                                .visual_state
                                .color
                                .to_color_for_char((i as usize - start / end) as f32);
                            character = character.with(color)
                        }

                        if e.visual_state.bg_color.is_gradient() {
                            character = character.on(e
                                .visual_state
                                .color
                                .to_color_for_char((i as usize / end) as f32))
                        }

                        queue!(w, PrintStyledContent(character))?;
                    }
                } else {
                    draw_flat_basic_element(w, item, e, print)?;
                }
            }
            ElementType::Prompt(pm) => {
                let lock_result = pm.try_lock(); // idk how to convert a mutex error to an eash error
                let lock;
                // TODO)) make this not quit drawing prompt when the mutex is locked?
                if let Ok(l) = lock_result {
                    lock = l;
                } else {
                    continue;
                }
                cursor_position = item.mass.position.round() as u16 + lock.cursor_position.clone();
                queue!(w, ResetColor)?;

                let tokens = tokenize(&lock.prompt);
                if tokens.is_empty() {
                    continue;
                }

                // Oh my Performance Bruh
                let mut colors = Vec::new();
                for token in tokens {
                    // temporary logic....
                    let color = match token.contents {
                        TokenType::Value(_) => ctColor::Black,
                        TokenType::Flag(_) => ctColor::Red,
                        TokenType::Directory(_) => ctColor::Yellow,
                        TokenType::String(_) => ctColor::Green,
                        TokenType::AndThen => ctColor::Magenta,
                        TokenType::Pipe => ctColor::Cyan,
                        TokenType::Nonsense(_) => ctColor::DarkRed,
                    };
                    colors.push((token.start, color));
                }

                let mut color_index = 0;
                let (_, first_color) = colors.get(0).unwrap();
                queue!(w, SetForegroundColor(*first_color))?;
                for (position, character) in lock.prompt.chars().enumerate() {
                    let color = colors.get(color_index + 1);
                    if let Some((ni, nc)) = color {
                        if *ni == position {
                            queue!(w, SetForegroundColor(*nc))?;
                            color_index += 1;
                        }
                    }

                    queue!(w, Print(character))?;
                }

                item.mass.width = lock.prompt.len() as u16;
            }
        }
        w.flush()?;
    }
    // if theres no cursor position then set it
    queue!(w, MoveToColumn(cursor_position))?;
    w.flush()?;

    Ok(())
}

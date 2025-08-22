use crossterm::style::{
    Print, PrintStyledContent, ResetColor, SetBackgroundColor, SetForegroundColor, Stylize,
};
use eash::chain::{Chain, ChainLink, ChainMass, step_links};
use eash::element::{BasicElement, ElementType};
use eash::error::EASHError;
use eash::misc_types::{Alignment, Color, Direction, HexColor, VisualState, Width};
use eash::prompt::{self, Prompt};

use crossterm::{
    cursor::MoveToColumn,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    queue,
    terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode},
};

use std::fmt::format;
use std::{
    io::{Stdout, Write},
    panic::{set_hook, take_hook},
    process::exit,
    sync::{Arc, Mutex, MutexGuard},
    thread,
    time::{Duration, Instant},
};

fn init_panic_hook() {
    let original_hook = take_hook();
    set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        original_hook(info);
    }));
}

// returns string, content start, content end
fn pad_string(original: String, size: u16, aligment: &Alignment) -> (String, usize, usize) {
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

// we need it to be mutable to set the size
fn render<'a, W: Write + Send>(
    w: &mut W,
    elements: &mut MutexGuard<Chain>,
) -> Result<(), EASHError> {
    _ = queue!(w, MoveToColumn(0), Clear(ClearType::CurrentLine));

    let mut cursor_position = 0;
    for item in elements.links.iter_mut() {
        let position = item.mass.position.round() as u16;
        queue!(w, MoveToColumn(position))?;

        // render each element based on it's enum 😨😨😨
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
                    let styled = e
                        .content
                        .clone()
                        .stylize()
                        .with(e.visual_state.color.to_flat_color()?)
                        .on(e.visual_state.bg_color.to_flat_color()?);
                    queue!(w, PrintStyledContent(styled))?;
                }
            }
            ElementType::Prompt(pm) => {
                let lock = pm.lock().unwrap(); // idk how to convert a mutex error to an eash error
                cursor_position = lock.cursor_position.clone();
                queue!(w, ResetColor)?;
                queue!(w, Print(lock.prompt.as_str()))?;

                item.mass.width = lock.prompt.len() as u16;
            }
        }
        w.flush()?;
    }
    // if theres no cursor position then set it
    queue!(w, MoveToColumn(cursor_position))?;

    Ok(())
}

// TODO)) right now this just kind turns the result into an option... probably don't need this.
fn read_ct_keypress_event(event_result: std::io::Result<Event>) -> Option<KeyEvent> {
    // if we recieved an error just return.
    if event_result.is_err() {
        // Golang lookin ass error handling
        println!(
            "Got error while reading input: {}",
            event_result.unwrap_err()
        );
        return None;
    }

    return event_result.unwrap().as_key_event();
}

fn render_thread<W: Write + Send + 'static>(element_mutex: Arc<Mutex<Chain>>, w: W) {
    let mut w = w;
    thread::Builder::new()
        .name("Rendering".to_string())
        .spawn(move || {
            let mut instant = Instant::now();
            loop {
                thread::sleep(Duration::from_millis(1000 / 60)); // TODO)) make configurable
                let mut lock = element_mutex.lock().unwrap();
                step_links(&mut lock, instant.elapsed().as_nanos() as f32 * 1e-9);
                render(&mut w, &mut lock).expect("render esploded 💥💥💥");
                instant = Instant::now();
            }
        })
        .expect("erm.... what the thread?");
}

fn main() {
    // just disables raw mode when we panic
    init_panic_hook();

    // i feel like arc<mutex<T>>'s are sheltering me from a cruel and inhuman data management fact
    let prompt = Arc::new(Mutex::new(Prompt {
        cursor_position: 0,
        prompt: "".to_string(),
        selection_start: None,
    }));

    // TODO)) make this more bearable with a builder and some macros or something
    let elements: Arc<Mutex<Chain>> = Arc::new(Mutex::new(Chain {
        spacing: 3,
        links: vec![
            ChainLink {
                mass: ChainMass {
                    mass: 0.5,
                    position: -30.0,
                    velocity: 0.0,
                    width: 1,
                },
                element: ElementType::BasicElement(BasicElement {
                    content: "wung".to_string(),
                    visual_state: VisualState {
                        align: Alignment::Left,
                        width: Width::Minimum(30),
                        padding: 2,
                        bg_color: Color::Solid(HexColor {
                            r: 30,
                            g: 30,
                            b: 30,
                        }),
                        color: Color::Solid(HexColor {
                            r: 222,
                            g: 222,
                            b: 222,
                        }),
                    },
                }),
            },
            ChainLink {
                mass: ChainMass {
                    mass: 1.0,
                    position: 0.0,
                    velocity: 0.0,
                    width: 1,
                },
                element: ElementType::BasicElement(BasicElement {
                    content: "wung".to_string(),
                    visual_state: VisualState {
                        align: Alignment::Left,
                        width: Width::Minimum(30),
                        padding: 2,
                        bg_color: Color::Solid(HexColor {
                            r: 30,
                            g: 30,
                            b: 30,
                        }),
                        color: Color::Solid(HexColor {
                            r: 222,
                            g: 222,
                            b: 222,
                        }),
                    },
                }),
            },
            ChainLink {
                mass: ChainMass {
                    mass: 1.0,
                    position: 0.0,
                    velocity: 0.0,
                    width: 1,
                },
                element: ElementType::Prompt(prompt.clone()),
            },
        ],
    }));

    enable_raw_mode().expect("Oh mah gawd.");
    render_thread(elements.clone(), std::io::stdout());

    fn bump(elements: &Arc<Mutex<Chain>>, velocity: f32, direction: Direction) {
        let mut lock = elements.lock().unwrap();
        // TODO)) make it so we don't have to iterate through the entire chain each time we bump
        for v in lock.links.iter_mut() {
            match v.element {
                ElementType::Prompt(_) => {
                    v.mass.velocity += match direction {
                        Direction::Left => velocity * -1.0,
                        Direction::Right => velocity,
                    };
                    break;
                }
                _ => {}
            }
        }
    }

    loop {
        let keypress_event = read_ct_keypress_event(event::read());
        if keypress_event == None {
            continue;
        };
        let keypress_event = keypress_event.unwrap();

        let mut lock = prompt.lock().unwrap();
        match keypress_event.code {
            KeyCode::Char(c) => {
                if c == 'c' {
                    if keypress_event.modifiers.contains(KeyModifiers::CONTROL) {
                        disable_raw_mode().expect("Oh mah gawd.");
                        exit(0);
                    };
                }

                if (c == 'w' || c == 'h' || c == '7')
                    && keypress_event.modifiers.contains(KeyModifiers::CONTROL)
                {
                    if lock.ctrl_backspace() {
                        bump(&elements, 50.0, Direction::Left);
                    }
                    continue;
                }

                lock.insert_character(c);
            }
            KeyCode::Backspace => {
                // run the backspace function, bump it harder if it returns true
                if lock.backspace() {
                    bump(&elements, 30.0, Direction::Left);
                } else {
                    bump(&elements, 10.0, Direction::Left);
                }
            }
            KeyCode::Left => {
                let shift = keypress_event.modifiers.contains(KeyModifiers::SHIFT);
                let ctrl = keypress_event.modifiers.contains(KeyModifiers::CONTROL);
                // run the arrow function, bump it harder if it returns true
                if lock.horiziontal_arrow(Direction::Left, shift, ctrl) {
                    bump(&elements, 30.0, Direction::Left);
                } else {
                    bump(&elements, 10.0, Direction::Left);
                }
            }
            KeyCode::Right => {
                let shift = keypress_event.modifiers.contains(KeyModifiers::SHIFT);
                let ctrl = keypress_event.modifiers.contains(KeyModifiers::CONTROL);
                // run the arrow function, bump it harder if it returns true
                if lock.horiziontal_arrow(Direction::Right, shift, ctrl) {
                    bump(&elements, 30.0, Direction::Right);
                } else {
                    bump(&elements, 10.0, Direction::Right);
                }
            }
            _ => {}
        }
    }
}

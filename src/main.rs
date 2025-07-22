use crossterm::{
    cursor::MoveToColumn,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    queue,
    style::{
        Color as ctColor, PrintStyledContent, ResetColor, SetBackgroundColor, SetForegroundColor,
        StyledContent, Stylize,
    },
    terminal::{disable_raw_mode, enable_raw_mode},
};

use std::{
    process::exit,
    sync::{
        Arc, Mutex, MutexGuard,
        mpsc::{Receiver, channel},
    },
    thread,
    time::Duration,
};

struct HexColor {
    r: u8,
    g: u8,
    b: u8,
}

enum Color {
    Transparent,
    Solid(HexColor),
    Gradient(HexColor, HexColor),
}

impl Color {
    fn to_color_for_char(&self, distance: f32) -> ctColor {
        match self {
            Self::Transparent => ctColor::Reset,
            Self::Solid(c) => ctColor::Rgb {
                r: c.r,
                g: c.g,
                b: c.b,
            },
            Self::Gradient(f, t) => ctColor::Rgb {
                r: (f.r as f32 + ((t.r as i16 - f.r as i16) as f32 * distance)) as u8,
                g: (f.g as f32 + ((t.g as i16 - f.g as i16) as f32 * distance)) as u8,
                b: (f.b as f32 + ((t.b as i16 - f.b as i16) as f32 * distance)) as u8,
            },
        }
    }
}

enum Alignment {
    Left,
    Right,
}

enum Width {
    Units(u32),
    Minimum(u32),
}

struct VisualState {
    align: Alignment,
    width: Width,
    padding: u32,
    bg_color: Color,
    color: Color,
}

impl Default for VisualState {
    fn default() -> Self {
        return Self {
            align: Alignment::Left,
            color: Color::Solid(HexColor {
                r: 255,
                g: 255,
                b: 255,
            }),
            padding: 1,
            bg_color: Color::Transparent,
            width: Width::Minimum(0),
        };
    }
}

struct Element {
    state: VisualState,
    content: String,
}

impl Element {
    fn get_width(&self) -> u32 {
        return match self.state.width {
            Width::Units(u) => u,
            Width::Minimum(m) => {
                self.content.len() as u32 + (self.state.padding * 2).clamp(m, u32::MAX)
            }
        };
    }
}

struct ChainSpring {
    position: f32,
    goal_position: f32,
    dampening: f32,
    velocity: f32,
}

struct ChainLink {
    spring: ChainSpring,
    element: Element,
}

type Chain = Vec<ChainLink>;

fn step_links(chain: &mut Chain) {
    let mut i = 0;

    // collisions
    let velocities: Vec<f32> = vec![];

    for link in chain.iter() {
        let could = if link.spring.velocity > 0 as f32 {
            chain.get(i + 1)
        } else {
            chain.get(i - 1)
        };

        let neighbour = match could {
            Some(n) => n,
            None => continue,
        };

        if link.spring.velocity > 0 as f32 {
            link.spring.
        } else {

        }

        i = i + 1;
    }
}

fn process_print_width_as_unit(
    print_content: &str,
    u: u32,
    padding: u32,
    align: &Alignment,
) -> String {
    let next_print_content;
    // same thing again lol
    if (u as usize) > print_content.len() + (padding as usize * 2) {
        let spacing = " ".repeat((print_content.len() + (padding as usize * 2)) - u as usize);
        match align {
            Alignment::Left => next_print_content = format!("{}{}", print_content, spacing),
            Alignment::Right => next_print_content = format!("{}{}", spacing, print_content),
        }
    } else if (u as usize) < print_content.len() + (padding as usize * 2) {
        // if its too long, cut it down!!!
        // currently really easy to make crash and just kinda bad
        next_print_content = format!(
            "{}{}",
            (&print_content[0..(u as usize - 1) - (padding as usize * 2) - 3]).to_string(),
            "..."
        );
    } else {
        next_print_content = print_content.to_string()
    }

    return next_print_content;
}

fn process_print_width_as_min(
    print_content: &str,
    m: u32,
    padding: u32,
    align: &Alignment,
) -> String {
    // if its smaller add some spacing
    if (m as usize) > print_content.len() + (padding as usize * 2) {
        let spacing = " ".repeat(m as usize - (print_content.len() + (padding as usize * 2)));
        match align {
            Alignment::Left => return format!("{}{}", print_content, spacing),
            Alignment::Right => return format!("{}{}", spacing, print_content),
        }
    }
    return print_content.to_string();
}

fn render<'a, W>(w: &mut W, elements: &MutexGuard<Vec<Element>>)
where
    W: std::io::Write,
{
    _ = queue!(w, MoveToColumn(0));

    let mut e = 1;
    for element in elements.iter() {
        if e != 1 {
            _ = queue!(w, ResetColor);
            print!(" ");
        }
        let vs = &element.state;
        let mut print_content = element.content.clone();

        // width checking shennanigans
        print_content = match vs.width {
            Width::Minimum(m) => {
                process_print_width_as_min(&print_content, m, vs.padding, &vs.align)
            }
            Width::Units(u) => {
                process_print_width_as_unit(&print_content, u, vs.padding, &vs.align)
            }
        };

        // padding
        let padding = " ".repeat(vs.padding as usize);
        print_content = format!("{}{}{}", padding, print_content, padding);

        // We shouldn't NEED to do this if both color and background_color are solid, which they usually are.
        // TODO: just print everything at once if both colors are solid
        let mut i = 1;
        for char in print_content.chars() {
            let distance = i as f32 / print_content.len() as f32;
            let styled_character = char
                .to_string()
                .with(vs.color.to_color_for_char(distance))
                .on(vs.bg_color.to_color_for_char(distance));

            _ = queue!(w, PrintStyledContent(styled_character));
            i = i + 1;
        }
        _ = w.flush();

        e = e + 1;
    }
}

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

    // for now, we aren't going to handle key holding, mouse movements and the like.
    // maybe soon.
    return event_result.unwrap().as_key_event();
}

fn render_thread(element_mutex: Arc<Mutex<Vec<Element>>>) {
    thread::spawn(move || {
        let mut stdout = std::io::stdout();

        loop {
            thread::sleep(Duration::from_millis(1000 / 60)); // i hate this
            let lock = element_mutex.lock().unwrap();
            render(&mut stdout, &lock);
        }
    });
}

fn main() {
    let elements = Arc::new(Mutex::new(vec![
        Element {
            content: "Garish ass gradients".to_string(),
            state: VisualState {
                align: Alignment::Right,
                width: Width::Minimum(0),
                padding: 2,
                bg_color: Color::Gradient(
                    HexColor {
                        r: 111,
                        g: 30,
                        b: 70,
                    },
                    HexColor {
                        r: 46,
                        g: 39,
                        b: 98,
                    },
                ),
                color: Color::Gradient(
                    HexColor {
                        r: 244,
                        g: 209,
                        b: 76,
                    },
                    HexColor {
                        r: 109,
                        g: 234,
                        b: 98,
                    },
                ),
            },
        },
        Element {
            content: "textiticus".to_string(),
            state: VisualState {
                align: Alignment::Left,
                width: Width::Minimum(0),
                padding: 2,
                bg_color: Color::Transparent,
                color: Color::Solid(HexColor {
                    r: 255,
                    g: 255,
                    b: 255,
                }),
            },
        },
    ]));

    render_thread(elements.clone());

    let mut prompt_text = String::new();

    enable_raw_mode().expect("Oh mah gawd.");

    loop {
        let keypress_event = read_ct_keypress_event(event::read());
        if keypress_event == None {
            continue;
        };
        let keypress_event = keypress_event.unwrap();

        match keypress_event.code {
            KeyCode::Char(c) => {
                if c == 'c' {
                    if keypress_event.modifiers.contains(KeyModifiers::CONTROL) {
                        disable_raw_mode().expect("Oh mah gawd.");
                        exit(0);
                    };
                }

                prompt_text = prompt_text + &c.to_string()[0..];
                let mut lock = elements.lock().unwrap();
                lock.get_mut(1).unwrap().content = prompt_text.clone();
            }
            KeyCode::Backspace if !prompt_text.is_empty() => {
                prompt_text = (&prompt_text[0..prompt_text.len() - 1]).to_string();
                let mut lock = elements.lock().unwrap();
                lock.get_mut(1).unwrap().content = prompt_text.clone();
            }
            _ => {}
        }
    }
}

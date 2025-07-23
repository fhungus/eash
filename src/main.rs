use crossterm::{
    cursor::MoveToColumn,
    event::{self, DisableBracketedPaste, Event, KeyCode, KeyEvent, KeyModifiers},
    queue,
    style::{
        Color as ctColor, PrintStyledContent, ResetColor, SetBackgroundColor, SetForegroundColor,
        StyledContent, Stylize,
    },
    terminal::{self, Clear, ClearType, disable_raw_mode, enable_raw_mode},
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

struct ChainMass {
    mass: f32,
    position: f32,
    velocity: f32,
}

struct ChainLink {
    mass: ChainMass,
    element: Element,
}

fn calculate_spring_distance(l: &ChainLink, r: &ChainLink) -> u32 {
    // spring distance should be size of both of its neighbours / 2 + the spacing between them
    const SPACING: u32 = 2;
    return ((l.element.get_width() + r.element.get_width()) / 2) + SPACING;
}

type Chain = Vec<ChainLink>;

// THERE IS SOMETHING HORRIFICALLY WRONG WITH MY SPRING SIMULATION
const SPRING_CONSTANT: f32 = 0.3;
const SPRING_DAMPING: f32 = 0.8;

fn calculate_force(chain: &Chain, link_index: usize) -> f32 {
    let link = &chain[link_index];
    let mut force: f32 = 0.0;

    // left spring
    // we could DRY this out, right?
    if link_index != 0 {
        if let Some(left_neighbour) = chain.get(link_index - 1) {
            let natural_distance = calculate_spring_distance(left_neighbour, link);
            let displacement = link.mass.position - left_neighbour.mass.position;
            force += -SPRING_CONSTANT * (displacement - (natural_distance as f32));
        }
    } else {
        // nudge the starting element to zero, so we can anchor to something
        let natural_distance = link.element.get_width() / 2 + 2;
        let displacement = link.mass.position; // goal position is ZERO!
        force += -SPRING_CONSTANT * (displacement - (natural_distance as f32))
    }

    // right spring
    if let Some(right_neighbour) = chain.get(link_index + 1) {
        let natural_distance = calculate_spring_distance(link, right_neighbour);
        let displacement = right_neighbour.mass.position - link.mass.position;
        force += SPRING_CONSTANT * (displacement - (natural_distance as f32));
    }

    force -= SPRING_DAMPING * link.mass.velocity;

    return force;
}

fn step_links(chain: &mut MutexGuard<Chain>, dt: f32) {
    // I DONT KNOW WHAT VERLET INTEGRATION IS
    let n = chain.len();

    // calculate extra forces and such
    let mut extra_forces = vec![0.0; n];
    for i in 0..n {
        extra_forces[i] = calculate_force(chain, i) / chain.get(i).unwrap().mass.mass;
    }

    for i in 0..n {
        let link = chain.get_mut(i).unwrap();

        link.mass.position =
            link.mass.position + (link.mass.velocity * dt + 0.5 * extra_forces[i] * dt * dt);

        link.mass.velocity += extra_forces[i];
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

fn render<'a, W>(w: &mut W, elements: &MutexGuard<Chain>)
where
    W: std::io::Write,
{
    _ = queue!(w, MoveToColumn(0), Clear(ClearType::CurrentLine));

    for item in elements.iter() {
        let element = &item.element;

        let start_position: i32 =
            item.mass.position.round() as i32 - (element.get_width() as i32 / 2);

        let terminal_position = if start_position >= 0 {
            start_position
        } else {
            0
        };
        _ = queue!(w, ResetColor, MoveToColumn(terminal_position as u16));

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
        let mut i = 0;
        for char in print_content.chars() {
            i = i + 1;
            if i + start_position < 0 {
                continue;
            }

            let distance = i as f32 / print_content.len() as f32;
            let styled_character = char
                .to_string()
                .with(vs.color.to_color_for_char(distance))
                .on(vs.bg_color.to_color_for_char(distance));

            _ = queue!(w, PrintStyledContent(styled_character));
        }
        _ = w.flush();
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

fn render_thread(element_mutex: Arc<Mutex<Chain>>) {
    thread::Builder::new()
        .name("Rendering".to_string())
        .spawn(move || {
            let mut stdout = std::io::stdout();

            loop {
                thread::sleep(Duration::from_millis(1000 / 60)); // i hate this
                let mut lock = element_mutex.lock().unwrap();
                step_links(&mut lock, 1.0);
                render(&mut stdout, &lock);
            }
        })
        .expect("erm.... what the thread?");
}

fn main() {
    let elements = Arc::new(Mutex::new(vec![
        ChainLink {
            mass: ChainMass {
                mass: 0.5,
                position: 0.0,
                velocity: 0.0,
            },
            element: Element {
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
        },
        ChainLink {
            mass: ChainMass {
                mass: 1.0,
                position: 0.0,
                velocity: 0.0,
            },
            element: Element {
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
                lock.get_mut(1).unwrap().element.content = prompt_text.clone();
            }
            KeyCode::Backspace if !prompt_text.is_empty() => {
                prompt_text = (&prompt_text[0..prompt_text.len() - 1]).to_string();
                let mut lock = elements.lock().unwrap();
                lock.get_mut(1).unwrap().element.content = prompt_text.clone();
            }
            _ => {}
        }
    }
}

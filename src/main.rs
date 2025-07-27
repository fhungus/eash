use crossterm::{
    cursor::{self, MoveToColumn},
    event::{self, DisableBracketedPaste, Event, KeyCode, KeyEvent, KeyModifiers},
    queue,
    style::{
        Color as ctColor, PrintStyledContent, ResetColor, SetBackgroundColor, SetForegroundColor,
        StyledContent, Stylize,
    },
    terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode},
};

use std::{
    io::{Stdout, Write},
    process::exit,
    sync::{Arc, Mutex, MutexGuard},
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

trait Element {
    fn get_width(&self) -> u32;

    fn render(&self, start_position: i32, w: &mut Stdout);
}

struct StandardElement {
    state: VisualState,
    content: String,
}

impl Element for StandardElement {
    fn get_width(&self) -> u32 {
        return match self.state.width {
            Width::Units(u) => u,
            Width::Minimum(m) => {
                (self.content.len() as u32 + (self.state.padding * 2)).clamp(m, u32::MAX)
            }
        };
    }

    fn render(&self, start_position: i32, w: &mut Stdout) {
        let terminal_position = if start_position >= 0 {
            start_position
        } else {
            0
        };
        _ = queue!(w, ResetColor, MoveToColumn(terminal_position as u16));

        let vs = &self.state;
        let mut print_content = self.content.clone();

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

        w.flush();
    }
}

struct ChainMass {
    mass: f32,
    position: f32,
    velocity: f32,
}

struct ChainLink {
    mass: ChainMass,
    element: Box<dyn Element + Send + Sync>,
}

// ignoring the r until we have to anchor shit to the right
fn calculate_spring_distance(l: &ChainLink, r: &ChainLink /* uhm */) -> u32 {
    // spring distance should be size of both of its neighbours / 2 + the spacing between them
    const SPACING: u32 = 2;
    return l.element.get_width() + SPACING;
}

type Chain = Vec<ChainLink>;

// THERE IS SOMETHING HORRIFICALLY WRONG WITH MY SPRING SIMULATION
const SPRING_CONSTANT: f32 = 0.8;
const SPRING_DAMPING: f32 = 0.1;

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
        // will be removed when i figure out how to do this more cleanly
        let natural_distance = 2;
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

fn render<'a>(w: &mut Stdout, elements: &MutexGuard<Chain>) {
    _ = queue!(w, MoveToColumn(0), Clear(ClearType::CurrentLine));

    for item in elements.iter() {
        item.element.render(item.mass.position.round() as i32, w);
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
                step_links(&mut lock, 1.0 / 60.0);
                render(&mut stdout, &lock);
            }
        })
        .expect("erm.... what the thread?");
}

struct Prompt {
    cursor_position: u16,
    prompt: String,
    selection_start: Option<u16>, // if None, then there is no selection
}

struct Selection {
    start: u16,
    end: u16,
}

enum Direction {
    Left,
    Right,
}

impl Prompt {
    fn start_selection(&mut self) {
        self.selection_start = Some(self.cursor_position);
    }

    fn find_skippable_in_direction(&self, direction: Direction) -> u16 {
        let increment = match direction {
            Direction::Left => -1,
            Direction::Right => 1,
        };

        let mut i = self.cursor_position as i16;
        loop {
            i = i + increment;
            if i <= 1 {
                return 1;
            }

            // check if THIS character is "skippable", if it is, set cursor_pos and return here
            // TODO: make this part support utf16, mostly just in case i need it in the future
            // TODO: (also) do this just generally better
            let bytes = self.prompt.as_bytes()[i as usize];
            if ' ' as u8 == bytes || '/' as u8 == bytes || '=' as u8 == bytes {
                return i as u16;
            }
        }
    }

    fn jump_in_direction(&mut self, direction: Direction) {
        let jump_to = self.find_skippable_in_direction(direction);
        self.cursor_position = jump_to;
    }

    fn ctrl_backspace(&mut self) {
        let cut_position = self.find_skippable_in_direction(Direction::Left);
        if cut_position == self.cursor_position {
            return;
        };

        let left_side = &self.prompt[0..cut_position as usize];
        let right_side = &self.prompt[self.cursor_position as usize..];

        self.prompt = format!("{}{}", left_side, right_side);
        self.cursor_position = cut_position;
    }

    fn move_cursor(&mut self, space: u32, direction: Direction) {
        // would it be cursed if we could cast direction into a i16?
        let neg = match direction {
            Direction::Left => -1,
            Direction::Right => 1,
        };

        let new_position = (self.cursor_position as i16 + (space as i16 * neg)) as u16;
        if new_position <= 1 {
            self.cursor_position = 1;
            return;
        } else if new_position >= self.prompt.len() as u16 {
            self.cursor_position = self.prompt.len() as u16;
        }
    }
}

fn main() {
    let elements = Arc::new(Mutex::new(vec![
        ChainLink {
            mass: ChainMass {
                mass: 0.5,
                position: 0.0,
                velocity: 0.0,
            },
            element: Box::new(StandardElement {
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
            }),
        },
        ChainLink {
            mass: ChainMass {
                mass: 1.0,
                position: 0.0,
                velocity: 0.0,
            },
            element: Box::new(StandardElement {
                content: "sporticus".to_string(),
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
            }),
        },
    ]));

    render_thread(elements.clone());

    let mut prompt_text = String::new();

    enable_raw_mode().expect("Oh mah gawd.");

    let mut prompt = Prompt {
        cursor_position: 1,
        prompt: "".to_string(),
        selection_start: None,
    };
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

                // prompt_text = prompt_text + &c.to_string()[0..];
                // let mut lock = elements.lock().unwrap();
                // lock.get_mut(1).unwrap().mass.velocity += 2.0;
                // lock.get_mut(1).unwrap().element.content = prompt_text.clone();
            }
            KeyCode::Backspace => {
                // let mut lock = elements.lock().unwrap();
                // if !prompt_text.is_empty() {
                //     prompt_text = (&prompt_text[0..prompt_text.len() - 1]).to_string();
                //     lock.get_mut(1).unwrap().element.content = prompt_text.clone();
                // } else {
                //     lock.get_mut(1).unwrap().mass.velocity -= 5.0;
                // }
            }
            _ => {}
        }
    }
}

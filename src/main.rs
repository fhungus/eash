use eash::{
    chain::{Chain, ChainLink, ChainMass, step_links},
    draw::draw,
    element::{BasicElement, ElementType},
    misc_types::{Alignment, Color, Direction, HexColor, VisualState, Width},
    prompt::Prompt,
};

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode},
};

use std::{
    io::Write,
    panic::{set_hook, take_hook},
    process::exit,
    sync::{Arc, Mutex},
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

fn init_draw_thread<W: Write + Send + 'static>(element_mutex: Arc<Mutex<Chain>>, w: W) {
    let mut w = w;
    thread::Builder::new()
        .name("Rendering".to_string())
        .spawn(move || {
            let mut instant = Instant::now();
            loop {
                thread::sleep(Duration::from_millis(1000 / 60)); // TODO)) make configurable
                let lock_result = element_mutex.try_lock();
                if lock_result.is_err() {
                    continue;
                };

                let mut lock = lock_result.unwrap();
                step_links(&mut lock, instant.elapsed().as_nanos() as f32 * 1e-9);
                draw(&mut w, &mut lock).expect("render esploded 💥💥💥");
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
                    position: -10.0,
                    velocity: 0.0,
                    width: 1,
                },
                element: ElementType::BasicElement(BasicElement {
                    content: "home/bitchass".to_string(),
                    visual_state: VisualState {
                        align: Alignment::Left,
                        width: Width::Minimum(2),
                        padding: 2,
                        bg_color: Color::Solid(HexColor {
                            r: 100,
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
                    position: -5.0,
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
                            r: 255,
                            g: 222,
                            b: 222,
                        }),
                        color: Color::Solid(HexColor { r: 2, g: 2, b: 2 }),
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
    init_draw_thread(elements.clone(), std::io::stdout());

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

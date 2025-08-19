use eash::chain::{Chain, ChainLink, ChainMass, step_links};
use eash::element::Element;
use eash::misc_types::{Alignment, Color, Direction, HexColor, VisualState, Width};
use eash::new_element;
use eash::prompt::Prompt;

use crossterm::{
    cursor::MoveToColumn,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    queue,
    terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode},
};

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

fn render<'a, W: Write + Send>(w: &mut W, elements: &MutexGuard<Chain>) {
    _ = queue!(w, MoveToColumn(0), Clear(ClearType::CurrentLine));

    for item in elements.links.iter() {
        
    }
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
                render(&mut w, &lock);
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

    let elements = Arc::new(Mutex::new(vec![
        ChainLink {
            mass: ChainMass {
                mass: 0.5,
                position: -30.0,
                velocity: 0.0,
            },
            element: new_element!(Stdout, "wung", render_thread, render_thread),
        },
        ChainLink {
            mass: ChainMass {
                mass: 1.0,
                position: -30.0,
                velocity: 0.0,
            },
            element: Box::new(Element {
                prompt: prompt.clone(),
            }),
        },
    ]));

    enable_raw_mode().expect("Oh mah gawd.");
    render_thread(elements.clone(), std::io::stdout());

    fn bump<W: Write + Send>(elements: &Arc<Mutex<Chain<W>>>, velocity: f32, direction: Direction) {
        let mut lock = elements.lock().unwrap();
        lock[1].mass.velocity += match direction {
            Direction::Left => velocity * -1.0,
            Direction::Right => velocity,
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

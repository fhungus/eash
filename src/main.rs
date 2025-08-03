use eash::chain::{Chain, ChainLink, ChainMass, step_links};
use eash::elements::{prompt_element::PromptElement, standard_element::StandardElement};
use eash::misc_types::{Alignment, Color, HexColor, VisualState, Width};
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
    time::Duration,
};

fn init_panic_hook() {
    let original_hook = take_hook();
    set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        original_hook(info);
    }));
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
                position: -30.0,
                velocity: 0.0,
            },
            element: Box::new(PromptElement {
                prompt: prompt.clone(),
            }),
        },
    ]));

    enable_raw_mode().expect("Oh mah gawd.");
    render_thread(elements.clone());

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

                let mut lock = prompt.lock().unwrap();
                lock.insert_character(c);
            }
            KeyCode::Backspace => {
                let mut lock = prompt.lock().unwrap();
                lock.backspace(keypress_event.modifiers.contains(KeyModifiers::CONTROL));
            }
            KeyCode::Left => {}
            KeyCode::Right => {}
            _ => {}
        }
    }
}

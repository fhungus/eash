use eash::chain::{Chain, ChainLink, ChainMass, step_links};
use eash::elements::{Element, prompt_element::PromptElement, standard_element::StandardElement};
use eash::misc_types::{Alignment, Color, HexColor, VisualState, Width};
use eash::prompt::Prompt;

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

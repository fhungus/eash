use eash::{
    chain::{Chain, ChainLink, ChainMass, step_links},
    config::{GlyphList, file_to_config, find_config, get_elements_from_config},
    draw::draw,
    element::ElementType,
    error::EASHError,
    misc_types::Direction,
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

    event_result.unwrap().as_key_event()
}

fn init_draw_thread<W: Write + Send + 'static>(
    element_mutex: Arc<Mutex<Chain>>,
    glyphs: GlyphList,
    w: W,
) {
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
                draw(&mut w, &mut lock, &glyphs).expect("render esploded 💥💥💥");
                instant = Instant::now();
            }
        })
        .expect("erm.... what the thread?");
}

fn main() -> Result<(), EASHError> {
    let config_path = find_config()?;
    if config_path.is_none() {
        // WE'RE JUST GONNA KILL EM!!!!
        println!("Failed to find a config file!!!\n
                  Put an eash.toml file in either .config/, .config/eash, or your current folder.
                  (eventually im planning on having this generate a config file after getting permission)");
        return Ok(());
    }

    let elements;
    let spring;
    let glyphs;
    {
        // TODO)) proper handling for this
        let config_struct = file_to_config(config_path.unwrap())?;
        elements = get_elements_from_config(&config_struct)?;
        spring = config_struct.spring;
        glyphs = config_struct.glyphs;
    }

    // just disables raw mode when we panic
    init_panic_hook();

    // i feel like arc<mutex<T>>'s are sheltering me from a cruel and inhuman data management fact
    let prompt = Arc::new(Mutex::new(Prompt {
        cursor_position: 0,
        prompt: "".to_string(),
        selection_start: None,
    }));

    // TODO)) move chain propagation into its own function
    let mut l: Vec<ChainLink> = elements
        .into_iter()
        .enumerate()
        .map(|(i, e)| ChainLink {
            mass: ChainMass {
                // TODO)) chainmass should set itself intelligently 🧠🧠🧠 instead of being defined here...
                position: i as f32 - 10.0,
                mass: 1.0,
                velocity: 0.0,
                width: 0, // set by render function...
            },
            element: e,
        })
        .collect();
    l.push(ChainLink {
        // prompt gets special treatment because 💤
        mass: ChainMass {
            position: l.len() as f32 - 10.0,
            mass: 1.0,
            velocity: 0.0,
            width: 0,
        },
        element: ElementType::Prompt(prompt.clone()),
    });

    let chain = Arc::new(Mutex::new(Chain {
        spring: spring.into(),
        links: l,
    }));

    enable_raw_mode().expect("Oh mah gawd.");
    // TODO)) OH MY FUCKING GOD WE CLONING WHOLE ASS HASHMAPS NOW!??! BROTHA NOOO!!!!
    init_draw_thread(chain.clone(), glyphs, std::io::stdout());

    fn bump(chain: &Arc<Mutex<Chain>>, velocity: f32, direction: Direction) {
        let mut lock = chain.lock().unwrap();
        // TODO)) make it so we don't have to iterate through the entire chain each time we bump
        for v in lock.links.iter_mut() {
            if let ElementType::Prompt(_) = v.element {
                v.mass.velocity += match direction {
                    Direction::Left => -velocity,
                    Direction::Right => velocity,
                };
            }
        }
    }

    loop {
        let keypress_event = read_ct_keypress_event(event::read());
        if keypress_event.is_none() {
            continue;
        };
        let keypress_event = keypress_event.unwrap();

        let mut lock = prompt.lock().unwrap();
        match keypress_event.code {
            KeyCode::Char(c) => {
                if c == 'c' && keypress_event.modifiers.contains(KeyModifiers::CONTROL) {
                    disable_raw_mode().expect("Oh mah gawd.");
                    exit(0);
                }

                if (c == 'w' || c == 'h' || c == '7') // for some reason different terminals output these for ctrl backspace
                    && keypress_event.modifiers.contains(KeyModifiers::CONTROL)
                {
                    if lock.ctrl_backspace() {
                        bump(&chain, 50.0, Direction::Left);
                    }
                    continue;
                }

                lock.insert_character(c);
            }
            KeyCode::Backspace => {
                // run the backspace function, bump it harder if it returns true
                if lock.backspace() {
                    bump(&chain, 30.0, Direction::Left);
                } else {
                    bump(&chain, 10.0, Direction::Left);
                }
            }
            KeyCode::Left => {
                let shift = keypress_event.modifiers.contains(KeyModifiers::SHIFT);
                let ctrl = keypress_event.modifiers.contains(KeyModifiers::CONTROL);
                // run the arrow function, bump it harder if it returns true
                if lock.horiziontal_arrow(Direction::Left, shift, ctrl) {
                    bump(&chain, 30.0, Direction::Left);
                } else {
                    bump(&chain, 10.0, Direction::Left);
                }
            }
            KeyCode::Right => {
                let shift = keypress_event.modifiers.contains(KeyModifiers::SHIFT);
                let ctrl = keypress_event.modifiers.contains(KeyModifiers::CONTROL);
                // run the arrow function, bump it harder if it returns true
                if lock.horiziontal_arrow(Direction::Right, shift, ctrl) {
                    bump(&chain, 30.0, Direction::Right);
                } else {
                    bump(&chain, 10.0, Direction::Right);
                }
            }
            _ => {}
        }
    }
}

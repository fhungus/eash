// ELEMENT COMPOSURE:
// {
//      id: "6678438008",
//      render: eash.element.default_renderer,
//      contents: "m... yes...", -- used by render function
//      prompt_updated: function(prompt)
//
//      end
// }
//
// ELEMENT BUILDER:
// eash.element.new()
//      .renderer(eash.element.with_icon_renderer)
//      .starting_glyph(eash.glyphs.get("checkmark"))
//      .inline_separator(eash.glyphs.get("dot_separator"))
//      .on("prompt_updated", function()
//          popen("rm -rf ~/") -- trolled
//      end)
//      .build()
//
// RICH PROMPT COMPOSURE:
// {"pacman", {type: eash.prompt.argument, flag: "-S", argument: "neovim"}}
//
// Content: {
//
// }
// element.Builder: {
//      mt: {
//          _index: eash.element.Builder
//      }
// }
// eash: {
//      element:
//          size("min" || "exact", num || null)
//          elements: {element}
//          Builder
//          default_renderer() -> Content
//          with_icon_renderer() -> Content
//      - 4 later
//      glyphs:
//          current:
//
//          set_glyphs()
//      prompt:
// }

use std::sync::{Arc, Mutex, mpsc::channel};

use crate::{chain::Chain, element::Element, error::EASHError, prompt::Prompt};
use mlua::{AsChunk, Chunk, Function, Lua};

struct ElementEvents {
    on_init: Option<Function>,
    on_initial_frame: Option<Function>,
    on_new_frame: Option<Function>, // NOT the render method!!!
    on_prompt_update: Option<Function>,
}

struct LuaElement {
    id: String,
    renderer: Function,
    events: ElementEvents,
}

fn add_api(lua: &mut Lua) {}

// Five-Hundred Arc<Mutex<T>>s
fn lua_init<W: std::io::Write, P: AsChunk>(
    payload: P,
    prompt: Arc<Mutex<Prompt>>,
    chain: Arc<Mutex<Chain<W>>>,
) -> Result<(), EASHError> {
    let prompt_lock = prompt.lock().unwrap();
    let chain_lock = chain.lock().unwrap();

    let lua = Lua::new();

    lua.load(payload).set_name("eash_config").exec()?;

    return Ok(());
}

fn babysitter_thread<W: std::io::Write>(
    lua: Lua,
    prompt: Arc<Mutex<Prompt>>,
    chain: Arc<Mutex<Chain<W>>>,
) {
}

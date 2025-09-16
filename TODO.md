Features that I am going to work on/considering.
# FEATURES
- Glyphs! 
- Reactive prompt elements
- Timestamp "bar" with an expanding animation that plays when it happens
- Making it an actual semi-working shell (I ***STILLL*** HAVEN'T EVEN STARTED THIS YET!!! 😭😭😭)
- Actual shell features (piping, exports, functions, etc.)
- ~~ls but cool~~ could maybe bundle an eash-style ls alternative but probably not wise to overwrite default ls
- ~~cd & mv show something when ran, like "Changed directory to 📁 ~/Downloads.", "Moved 📄 fuckenheimer.txt -> ~/Downloads/Sexmeister/"~~ same thing
- Nice looking errors (like nushell!)
- Graceful shutdown & ~~error handling~~ (we have EASHError? that counts right)
- Overlay dialog system
- File Picker
  - we could go further.... 😈
    - Color picker / highlighting?
    - Custom shortcut pickers definable in config that use regex to know when to open?!??! (cool asf but i'd need to commit to it)
      - cat color picker "cc" -> tabby, calico, black, white, whatchu want as a demo
- Interactive Config builder for ez onboarding
- Scroll through history with up & down
- Multiple lines

# ARCHITECTURAL / PERFORMANCE
- Change the integer types for the terminal to u16 or maybe usize.
- Printing & Rendering abstraction for good vibes ✌️
- Remove config data duplication.
- Split config into a billion little pieces (punishment for being too hard to read)
- Event scheduling (so that we don't update the prompt unless we need to)
- Start benchmarking things!!!

# DONE!
- not printing characters one at a time.
- Syntax highlighting
## halfassed
- Lua based configuration with a goofy ass API
  - Spring configuration
  - Custom elements with features like polling from commands, running functions at configurable intervals,
  - Element style (powerlines & rounding and such but also like if they have icons or something)
  - Syntax customization (define symbols and what they should do?) (?)
  - Configuration Aliases & Variables
  - Dialogues and icons configurable.
  - currently the shell is configured in TOML which is a shit choice, but i don't know how to use mlua so this will have to for
    now, and only the basics of these actually exist so far.


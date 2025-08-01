Features that I am going to work on/considering.
# FEATURES
- Lua based configuration with a goofy ass API
  - Spring configuration
  - Custom elements with features like polling from commands, running functions at configurable intervals,
  - Element style (powerlines & rounding and such but also like if they have icons or something)
  - Syntax customization (define symbols and what they should do?) (?)
  - Configuration Aliases & Variables
  - Dialogues and icons configurable.
- Syntax highlighting
- Timestamp "bar" with an expanding animation that plays when it happens
- Making it an actual semi-working shell (I HAVEN'T EVEN STARTED THIS YET!!! 😭😭😭)
- Actual shell features (piping, exports, functions, etc.)
- ls but cool
- cd & mv show something when ran, like "Changed directory to 📁 ~/Downloads.", "Moved
- nice looking errors (like nushell)
- Graceful shutdown & error handling
- I NEED TO SPLIT IT INTO MULTIPLE FILES IT'S ALREADY GETTING KIND OF ANNOYING

# ARCHITECTURAL
- Printing & Rendering abstraction so i'm not printing characters one at a time.

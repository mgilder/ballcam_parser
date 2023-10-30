# Rocket League Ballcam Parser using Boxcars library

### Installation + Setup

- [Install Rust](https://doc.rust-lang.org/book/ch01-01-installation.html)
- Clone or download repo
- Create and fill-in `.env` file
TEMPLATE `.env` file:
```
TARGET_PLAYER=""  # Your player-id. should be of the format `<platform_lowercase>-<platform_id>-0`
REPLAY_DIR=""     # The directory with all of your replays (%UserProfile%/Documents/My Games/Rocket League/TAGame/Demos) usually on windows?
TEST_FILE=""      # Just a random testing thing, can be ignored.
```
- Run with `cargo run --release`



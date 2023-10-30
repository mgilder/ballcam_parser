# Rocket League Ballcam Parser using Boxcars library

## Installation + Setup

- [Install Rust](https://doc.rust-lang.org/book/ch01-01-installation.html)
- Clone or download the repository
- Create and fill-in `.env` file in the downloaded repository. Here is a TEMPLATE `.env` file:
```
TARGET_PLAYER=""  # Your player-id. should be of the format `<platform_lowercase>-<platform_id>-0`
REPLAY_DIR=""     # The directory with all of your replays (%UserProfile%/Documents/My Games/Rocket League/TAGame/Demos) usually on windows?
TEST_FILE=""      # Just a random testing thing, can be ignored.
```
- Run with `cargo run --release`
- Now you should have some output plots in the `./outputs` folder where you ran the code

If you don't know what your `TARGET_PLAYER` is, then you can add the following code to the top of `main` in `src/main.rs`, if you set `TEST_FILE` to point to one of your replays. This should print out the player-ids for all players in the match.
```rust
dbg!(replay_stats_rl::parse_replay_file(replay_file).unwrap());
return;
```

## Overview of Key Events

First there are a few key events:
- `TAGame.CameraSettingsActor_TA:bUsingSecondaryCamera`
    - The key event. This event sets the ballcam mode. True=>Ballcam on, False=> Ballcam off. Cameras have ballcam off by default when created.
- `TAGame.Default__CameraSettingsActor_TA`
    - Creation of a new CameraSettingsActor. Helps us track camera events by tracking the actor_ids for these events.
- `TAGame.CameraSettingsActor_TA:PRI`
    - Associates a CameraSettingsActor with a PlayerReplicationInfo. Allows us to figure out what player the camera events refer to.
- `Engine.PlayerReplicationInfo:UniqueId`
    - For a PlayerReplicationInfo, gives us the `UniqueId` of the player, which is useful for differentiating and tracking players within and across replays.
- `TAGame.GameEvent_TA:ReplicatedStateName`
    - The state of the game. Has 3 possible values from what I've seen. `Countdown`, `Active`, and `PostGoalScored`. Allows us to exclude counting ballcam time during replays and after goals are scored.
- `ProjectX.GRI_X:Reservations`
    - Helps us determine when people leave the game, so we can stop tracking them

## Overview of Process

- The first step is this function, which is the main API/entrypoint:
```rust
pub fn parse_replay_file(replay_file: &str) -> Result<(Metadata, HashMap<UniqueId, PlayerResult>), ()> { /* ... */ }
```
- There are a few steps performed for this function
    - Read the file specified by the argument, and parse with boxcars
    - Create the `LifetimeList` object from the replay using the `parse_lifetimes` function
    - Get the metadata (less important step to grab player/replay name and date) using the `get_metadata` function
    - Get the ballcam results using the `new_ballcam_lifetimes` function

- LifetimeList
    - This is basically a list of "Lifetimes". A "Lifetime" is basically just a list of events for a given actor_id, from creation until either deleted or overwritten.
    - TODO
- Ballcam Calculation
    - TODO
- TODO

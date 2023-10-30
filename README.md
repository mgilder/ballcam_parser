# Rocket League Ballcam Parser using Boxcars library

## Installation + Setup

- [Install Rust](https://doc.rust-lang.org/book/ch01-01-installation.html)
- Clone or download the repository
- Create and fill-in `.env` file in the downloaded repository
TEMPLATE `.env` file:
```
TARGET_PLAYER=""  # Your player-id. should be of the format `<platform_lowercase>-<platform_id>-0`
REPLAY_DIR=""     # The directory with all of your replays (%UserProfile%/Documents/My Games/Rocket League/TAGame/Demos) usually on windows?
TEST_FILE=""      # Just a random testing thing, can be ignored.
```
- Run with `cargo run --release`
- Now you should have some output plots in the `./outputs` folder where you ran the code

## Overview of Method

First there are a few key events:
- `TAGame.CameraSettingsActor_TA:bUsingSecondaryCamera`
    - The key event. This event sets the ballcam mode. True=>Ballcam on, False=> Ballcam off. Cameras have ballcam off by default when created.
- `TAGame.Default__CameraSettingsActor_TA`
    - Creation of a new CameraSettingsActor. Helps us track camera events by tracking the actor_ids for these events.
- `TAGame.CameraSettingsActor_TA:PRI`
    - Associates a CameraSettingsActor with a PlayerReplicationInfo. Allows us to figure out what player the camera events refer to.
- `Engine.PlayerReplicationInfo:UniqueId`
    - For a PlayerReplicationInfo, gives us the `UniqueId`, which is useful for tracking players within and across games.
- `TAGame.GameEvent_TA:ReplicatedStateName`
    - The state of the game. Has 3 possible values from what I've seen. `Countdown`, `Active`, and `PostGoalScored`. Allows us to exclude counting ballcam time during replays and after goals are scored.
- `ProjectX.GRI_X:Reservations`
    - Helps us determine when people leave the game, so we can stop tracking them


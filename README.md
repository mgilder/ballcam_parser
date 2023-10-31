# Rocket League Ballcam Parser using Boxcars library

## Installation + Setup

- [Install Rust](https://doc.rust-lang.org/book/ch01-01-installation.html)
- Clone or download the repository
- Create and fill-in `.env` file in the downloaded repository. Here is a TEMPLATE `.env` file:
```
TARGET_PLAYER=""  # Your player-id. should be of the format `<platform_lowercase>-<platform_id>-0`
REPLAY_DIR=""     # The directory with all of your replays (%UserProfile%/Documents/My Games/Rocket League/TAGame/Demos) usually on windows?
TEST_FILE=""      # Just a random testing thing, can be ignored.
PLOT_OTHER_PLAYERS=true    # whether to include the average of the other players in the output plots
```
- Run with `cargo run --release`
- Now you should have some output plots in the `./outputs` folder where you ran the code

If you don't know what your `TARGET_PLAYER` is, then you can uncomment the `TARGET_PLAYER` determination code in `src/main.rs` at the top of `main`. Running with `cargo run --release` will tell you the top 10 most seen player ids in your list of replays, and your player id should probably be the most seen one. Once you're done, recomment that code.
```rust
    ////////////////////////////////////////////////////////////
    // START TARGET_PLAYER DETERMINATION CODE
    // Delete or Comment out the next line to enable the check
    /*
        ...
        return;
    // */
    // END TARGET_PLAYER DETERMINATION CODE
    ////////////////////////////////////////////////////////////
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
    - This is basically a list of "Lifetimes". A "Lifetime" is basically just a list of events for a given actor_id, from it's creation until it's either deleted or overwritten.
    - Long story short, this basically is just used to figure out at any network frame or time what each actor_id corresponds to, and also easily filter for relevant events. In this case, we want to filter only to camera events, so that we can get ballcam events, and when we have `TAGame.CameraSettingsActor_TA:PRI` events, be able to easily figure out the actor_id "Lifetime" it points to at that time, and then from there search that playerreplicationinfo actor's "Lifetime" to get the `UniqueId`.
- Ballcam Calculation
    - Using the LifetimeList we can get a few different things.
        - First we use `player_id_buckets`, to limit the lifetimes to only camera objects, and also group them by the corresponding player in a hashmap.
            - We'll use this later to search through the camera events to get ballcam events.
        - Then we use `get_disconnect_players`, to get any players that disconnected during that match, and the time at which they did so.
            - We'll use this later to stop tracking a player after they disconnect the first time.
        - Then we use `get_state_changes`, to get all the game state changes, e.g. `Countdown`, `Active`, `PostGoalScored` from the `TAGame.GameEvent_TA:ReplicatedStateName` events as mentioned above.
            - We'll use this later to not count time that occurs between goals and countdown.
        - Then for each player we'll get a list of ballcam events using `get_ballcam_list`, then process them using `process_ballcam`.
- Then we return the results for each player.

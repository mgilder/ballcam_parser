use std::{time, collections::HashMap, ops::Deref};
use boxcars::UniqueId;
use chrono::NaiveDate;
use::dotenv;
use replay_stats_rl::{Metadata, PlayerResult};

fn main() {
    let start_time = time::Instant::now();
    let mut times = vec![("Start", time::Instant::now())];

    let replay_file = &dotenv::var("TEST_FILE").ok().expect("Please specify a TEST_FILE in the .env file");
    let replay_dir = &dotenv::var("REPLAY_DIR").ok().expect("Please specify a REPLAY_DIR in the .env file");
    let target_player = &dotenv::var("TARGET_PLAYER").ok().expect("Please specify a TARGET_PLAYER in the .env file");


    ////////////////////////////////////////////////////////////
    // START TARGET_PLAYER DETERMINATION CODE
    // Delete or Comment out the next line to enable the check
    /*
    let mut uid_counts: HashMap<String, i32> = HashMap::new();
    replay_stats_rl::get_replay_list(replay_dir)
        .iter()
        .filter_map(|rfile| {
            replay_stats_rl::parse_replay_file(&rfile).ok()
        })
        .for_each(|pret| {
            for uid in pret.1.keys() {
                *uid_counts.entry(replay_stats_rl::uid_to_string(uid)).or_insert(0) += 1;
            }
        });
    let mut uid_results: Vec<(String, i32)> = uid_counts.into_iter().collect();
    uid_results.sort_by_key(|item| std::cmp::Reverse(item.1));
    uid_results.iter().take(10).for_each(|(player, count)| {
        eprintln!("{:5} times we saw:    {}", count, player);
    });
    return;
    // */
    // END TARGET_PLAYER DETERMINATION CODE
    ////////////////////////////////////////////////////////////

    // dbg!(replay_stats_rl::parse_replay_file(replay_file).unwrap());
   /* 
    let mut reservation_results: HashMap<(Option<(bool, bool)>, (bool, bool)), i64> = HashMap::new();
    replay_stats_rl::get_replay_list(replay_dir).iter().for_each(|rfile| {
        eprintln!("File: {}", rfile);
        replay_stats_rl::reservation_stats(rfile, &mut reservation_results);
    });
    for key in reservation_results.keys() {
        eprintln!("{:?} -> {}", key, reservation_results.get(key).unwrap());
    }
    return;
    */

   /* 
    replay_stats_rl::get_replay_list(replay_dir).iter().take(500).for_each(|rfile| {
        eprintln!("File: {}", rfile);
        replay_stats_rl::parse_replay_file(rfile).unwrap();
    });
    return;
    
    replay_stats_rl::parse_replay_file(replay_file).unwrap();
    return;
    */
    times.push(("Configs Loaded", time::Instant::now()));

    let replays = replay_stats_rl::get_replay_list(replay_dir);
    times.push(("Got Replay List", time::Instant::now()));

    let mut ballcam_results: Vec<(Metadata, HashMap<UniqueId, replay_stats_rl::PlayerResult>)> = replays.iter().filter_map(|rfile| {
            replay_stats_rl::parse_replay_file(&rfile).ok()
    }).collect();

    times.push(("Replays Processed", time::Instant::now()));

    ballcam_results.sort_by_key(|(md, _)| {md.date});
    times.push(("Replays Sorted", time::Instant::now()));


    //TODO dbg!(&ballcam_results, ballcam_results.len());

    //let mut deduped: Vec<(Metadata, BallcamResults)> = Vec::with_capacity(ballcam_results.len());


    //TODO dbg!(&ballcam_results, ballcam_results.len());

    let after2023: Vec<(Metadata, HashMap<UniqueId, PlayerResult>)> = ballcam_results.iter()
        .filter(|r| r.0.date >= NaiveDate::from_ymd_opt(2023, 01, 01).unwrap())
        .map(|(md, bc)| {(md.clone(), bc.clone())})
        .collect();

    let ones: Vec<(Metadata, HashMap<UniqueId, PlayerResult>)> = ballcam_results.iter()
        .filter(|r| r.0.playlist == "TAGame.Replay_Soccar_TA-1")
        .map(|(md, bc)| {(md.clone(), bc.clone())})
        .collect();

    let twos: Vec<(Metadata, HashMap<UniqueId, PlayerResult>)> = ballcam_results.iter()
        .filter(|r| r.0.playlist == "TAGame.Replay_Soccar_TA-2")
        .map(|(md, bc)| {(md.clone(), bc.clone())})
        .collect();

    let threes: Vec<(Metadata, HashMap<UniqueId, PlayerResult>)> = ballcam_results.iter()
        .filter(|r| r.0.playlist == "TAGame.Replay_Soccar_TA-3")
        .map(|(md, bc)| {(md.clone(), bc.clone())})
        .collect();

    let ones2023: Vec<(Metadata, HashMap<UniqueId, PlayerResult>)> = ballcam_results.iter()
        .filter(|r| r.0.date >= NaiveDate::from_ymd_opt(2023, 01, 01).unwrap())
        .filter(|r| r.0.playlist == "TAGame.Replay_Soccar_TA-1")
        .map(|(md, bc)| {(md.clone(), bc.clone())})
        .collect();

    let twos2023: Vec<(Metadata, HashMap<UniqueId, PlayerResult>)> = ballcam_results.iter()
        .filter(|r| r.0.date >= NaiveDate::from_ymd_opt(2023, 01, 01).unwrap())
        .filter(|r| r.0.playlist == "TAGame.Replay_Soccar_TA-2")
        .map(|(md, bc)| {(md.clone(), bc.clone())})
        .collect();

    let threes2023: Vec<(Metadata, HashMap<UniqueId, PlayerResult>)> = ballcam_results.iter()
        .filter(|r| r.0.date >= NaiveDate::from_ymd_opt(2023, 01, 01).unwrap())
        .filter(|r| r.0.playlist == "TAGame.Replay_Soccar_TA-3")
        .map(|(md, bc)| {(md.clone(), bc.clone())})
        .collect();
    times.push(("Datasets Generated", time::Instant::now()));

    replay_stats_rl::plot_updated(ballcam_results, &format!("both-sides-ballcam-full-{}", false), target_player);
    replay_stats_rl::plot_updated(after2023, &format!("both-sides-ballcam-2023-later-{}", false), target_player);
    replay_stats_rl::plot_updated(ones, &format!("both-sides-ballcam-full-1s-{}", false), target_player);
    replay_stats_rl::plot_updated(twos, &format!("both-sides-ballcam-full-2s-{}", false), target_player);
    replay_stats_rl::plot_updated(threes, &format!("both-sides-ballcam-full-3s-{}", false), target_player);
    replay_stats_rl::plot_updated(ones2023, &format!("both-sides-ballcam-2023-1s-{}", false), target_player);
    replay_stats_rl::plot_updated(twos2023, &format!("both-sides-ballcam-2023-2s-{}", false), target_player);
    replay_stats_rl::plot_updated(threes2023, &format!("both-sides-ballcam-2023-3s-{}", false), target_player);

    times.push(("Plots Generated", time::Instant::now()));


    for i in 1..times.len() {
        eprintln!("{}:  {:?}", times[i].0, times[i].1 - times[i-1].1);
    }

    let main_duration = start_time.elapsed();
    eprintln!("Time elapsed is: {:?}", main_duration);

    //get_usage_stats();
}

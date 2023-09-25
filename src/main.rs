use boxcars::{ActiveActor, Frame, UpdatedAttribute, ActorId, NewActor};
use boxcars::{ParseError, Replay, Attribute, HeaderProp};
//use boxcars::{ActorId, Attribute, ObjectId, ParserBuilder, Replay};
use std::error;
use std::fs;
use std::hash::Hash;
use std::path::{Path, PathBuf};
//use std::io;
use std::collections::{HashMap, HashSet};
use chrono::{NaiveDate, Duration};
//use chrono::format::ParseError;
//
use plotters::prelude::*;
//use chrono::{Utc, TimeZone};
use::dotenv;
use std::time;

fn parse_rl(data: &[u8]) -> Result<Replay, ParseError> {
    boxcars::ParserBuilder::new(data)
        .must_parse_network_data()
        .parse()
}

fn get_object_id(replay: &Replay, name: &str) -> Option<i32> {
    replay.objects.iter().position(|f| f == name).map(|v| v as i32)
}

/*
fn get_name_id(replay: &Replay, name: &str) -> Option<i32> {
    replay.names.iter().position(|f| f == name).map(|v| v as i32)
}
*/

fn parse_file(filename: &str) -> Result<Replay, Box<dyn error::Error>> {
    let buffer = fs::read(filename)?;
    let replay = parse_rl(&buffer)?;
    Ok(replay)
}

#[derive(Debug)]
struct TimeResult {
    ballcam_time: f32,
    standard_time: f32,
    swaps: i32,
}

impl TimeResult {
    fn new() -> Self {
        Self {
            ballcam_time: 0f32,
            standard_time: 0f32,
            swaps: 0,
        }
    }
}

#[derive(Debug, Clone)]
struct Metadata {
    name: String,
    date: NaiveDate,
    game_mode: String,
}

impl Metadata {
    fn new(name: String, date: NaiveDate, game_mode: String) -> Self {
        Self {
            name,
            date,
            game_mode
        }
    }
}

#[derive(Debug)]
struct ReplayResult {
    stats: TimeResult,
    meta: Metadata,
}

impl ReplayResult {
    fn new(stats: TimeResult, meta: Metadata) -> Self{
        Self {stats, meta}
    }
}

fn get_metadata(replay: &Replay) -> Metadata {
    //let mut result = Metadata::new();
    let mut result_name = None;
    let mut result_date = None;
    let mut result_mode = None;

    let player_name_prop = replay.properties
        .iter()
        .find(|prop| prop.0 == "PlayerName").unwrap();
    if let HeaderProp::Str(pname) = &player_name_prop.1 {
        result_name = Some(pname.clone());
    }

    let game_time_prop = replay.properties
        .iter()
        .find(|prop| prop.0 == "Date").unwrap();
    if let HeaderProp::Str(gdate) = &game_time_prop.1 {
        result_date = gdate.split_whitespace().next().map(|v| NaiveDate::parse_from_str(v, "%Y-%m-%d").ok()).flatten();
    }

    let game_mode_prop = replay.properties
        .iter()
        .find(|prop| prop.0 == "TeamSize").unwrap();
    if let HeaderProp::Int(tsize) = &game_mode_prop.1 {
        result_mode = Some(format!("{}-{}", replay.game_type, tsize));
    }

    Metadata::new(result_name.unwrap(), result_date.unwrap(), result_mode.unwrap())
}

fn get_replay_list(dir: &str) -> Vec<String> {
    let path = Path::new(dir);
    fs::read_dir(path)
        .expect("Unable to list")
        .into_iter()
        .filter(|r| r.is_ok())
        .map(|r| r.unwrap().path())
        .filter(|r| r.extension().map_or(false, |ex| ex == "replay"))
        .map(|p| p.into_os_string().into_string().expect("failed to convert PathBuf to String"))
        .collect()
}

fn plot_updated(data: Vec<(Metadata, BallcamResults)>, file: &str) {
    let fname = format!("outputs/{}.png", file);
    let root_area = BitMapBackend::new(&fname, (600*2, 2*400))
        .into_drawing_area();
    root_area.fill(&WHITE).unwrap();

    //let start_date = data.first().unwrap().0.date.clone(); //Utc.ymd(2019, 10, 1);
    //let end_date = data.last().unwrap().0.date.clone(); //NaiveDate::  ymd(2023, 6, 30);

//    let min_val = -12.0 + data.iter().fold(None,|acc, &(_, f)| {
//        Some(acc.map_or(f, |v| if f < v {f} else {v}))
//    }).unwrap_or(0.0);
//    let max_val = 12.0 + data.iter().fold(None,|acc, &(_, f)| {
//        Some(acc.map_or(f, |v| if f > v {f} else {v}))
//    }).unwrap_or(101.0);
    let min_val: f32 = 0f32;
    let max_val: f32 = 100f32;

    let self_series: Vec<(NaiveDate, f32)> = data.iter().map(|(md, lst)| {
        (md.date, lst.self_percent)
    }).collect();
    let other_series: Vec<(NaiveDate, f32)> = data.iter().map(|(md, lst)| {
        (md.date, lst.other_percent)
    }).collect();

    //dbg!(&self_series, &self_series.len());
    //dbg!(&other_series);
    /*
    self_series.dedup_by_key(|(dt, _)| dt.clone());
    other_series.dedup_by_key(|(dt, _)| dt.clone());
    dbg!(&self_series, &self_series.len());
    dbg!(&other_series);
    */

    let start_date = self_series.first().unwrap().0.clone()
        .min(other_series.first().unwrap().0.clone());
    let end_date = self_series.last().unwrap().0.clone()
        .max(other_series.last().unwrap().0.clone())
        + Duration::days(14);

    let mut ctx = ChartBuilder::on(&root_area)
        .set_label_area_size(LabelAreaPosition::Left, 40)
        .set_label_area_size(LabelAreaPosition::Bottom, 40)
        .caption(format!("% ballcam - {}", file), ("sans-serif", 40))
        .build_cartesian_2d(start_date..end_date, min_val..max_val)
        .unwrap();

    ctx.configure_mesh().draw().unwrap();

    ctx.draw_series(
        LineSeries::new(self_series.clone(), &BLUE,)
    ).unwrap();

    let average_cnt = 7;
    let mut running_sum = self_series.iter().take(average_cnt-1).map(|z| {z.1}).sum::<f32>();
    ctx.draw_series(
        LineSeries::new(self_series.clone()
                        .into_iter()
                        .enumerate()
                        .skip(average_cnt-1)
                        .map(|(i, (dt, pt))| {
                            if i >= average_cnt {
                                running_sum -= self_series[i-(average_cnt-1)].1;
                            }
                            running_sum += pt;
                            (dt, running_sum / (average_cnt as f32))
                        }).collect::<Vec<(NaiveDate, f32)>>(),
        &GREEN,)
    ).unwrap();

    ctx.draw_series(
        self_series.iter()
            .map(|&(dt, vv)| Circle::new((dt, vv), 3, BLUE.filled())),
    ).unwrap();


    ctx.draw_series(
        LineSeries::new(other_series.clone(), &RED,)
    ).unwrap();

    ctx.draw_series(
        other_series.iter()
            .map(|&(dt, vv)| Circle::new((dt, vv), 3, RED.filled())),
    ).unwrap();

}

fn get_prop_string(replay: &Replay, prop: &str) -> Option<String> {
    let found = replay.properties.iter().find(|&p| {
        p.0 == prop
    });

    found.map(|f| {
        match &f.1 {
            HeaderProp::Str(s) => s.to_string(),
            HeaderProp::Bool(s) => s.to_string(),
            HeaderProp::Int(s) => s.to_string(),
            HeaderProp::Byte{kind: s, value: val} => s.to_string() + val.as_ref().unwrap_or(&String::from("")).as_str(),
            HeaderProp::Name(s) => s.to_string(),
            HeaderProp::Array(s) => format!("{:?}", s),
            HeaderProp::Float(s) => s.to_string(),
            HeaderProp::QWord(s) => s.to_string(),
        }
    })
}


#[derive(Debug)]
enum ChangeEvent {
    N(NewActor),
    D(ActorId),
    U(UpdatedAttribute),
}

impl ChangeEvent {
    fn actor_id(&self) -> i32 {
        match self {
            Self::N(na) => na.actor_id.0,
            Self::D(da) => da.0,
            Self::U(ua) => ua.actor_id.0,
        }
    }
}

#[derive(Debug)]
struct Event {
    event: ChangeEvent,
    frame: usize,
    time: f32,
}

impl Event {
    fn from(event: ChangeEvent, frame: usize, time: f32) -> Self {
        Self {
            event,
            frame,
            time,
        }
    }
}

#[derive(Debug)]
struct Lifetime {
    events: Vec<Event>,
}

impl Lifetime {
    fn from(v: Vec<Event>) -> Self {
        Self {
            events: v,
        }
    }
}

#[derive(Debug)]
struct LifetimeList {
    list: Vec<Lifetime>,
    actor_map: HashMap<i32, Vec<usize>>,
}

impl LifetimeList {
    fn from(list: Vec<Lifetime>) -> Self {
        Self {
            actor_map: bucket_index(&list, |lt| {lt.events[0].event.actor_id()}),
            list,
        }
    }

    fn lookup_actor(&self, actor_id: i32, frame_id: usize) -> Result<&Lifetime, ()> {
        //dbg!(actor_id, frame_id);
        self.actor_map.get(&actor_id).map(|vv| {
            //dbg!(&vv.iter().map(|&vx| self.list[vx].events[0].frame).collect::<Vec<usize>>());
            let rind = vv.partition_point(|&lt| {
                self.list[lt].events[0].frame <= frame_id
            });
            //&self.list[vv[rind.min(vv.len()-1)]] // TODO we got an out of bounds read here...
            &self.list[vv[rind-1]]
        }).ok_or(())
    }
}

fn bucket_index<T: Hash+Eq, F: Fn(&Lifetime) -> T>(v: &Vec<Lifetime>, func: F) -> HashMap<T, Vec<usize>> {
    v.into_iter()
        .enumerate()
        .fold(HashMap::new(), |mut ret, (ind, lt)| {
            let entry = ret.entry(func(lt)).or_insert(vec![]);
            entry.push(ind);
            ret
        })
}

trait DumpEvent {
    fn dump(&self, replay: &Replay) -> String;
}

impl DumpEvent for Event {
    fn dump(&self, replay: &Replay) -> String {
        vec![
            format!("FrameID:           {}", self.frame),
            format!("time:              {}", self.time),
            self.event.dump(&replay),
        ].join("\n")
    }
}

impl DumpEvent for ChangeEvent {
    fn dump(&self, replay: &Replay) -> String {
        match self {
            ChangeEvent::N(na) => na.dump(&replay),
            ChangeEvent::D(da) => da.dump(&replay),
            ChangeEvent::U(ua) => ua.dump(&replay),
        }
    }
}

impl DumpEvent for UpdatedAttribute {
    fn dump(&self, replay: &Replay) -> String {
        vec![
            //String::from("========================"),
            format!("Updated Actor ID:  {}", self.actor_id.0),
            format!("Object:            {}", replay.objects.get(self.object_id.0 as usize).unwrap_or(&String::from("N/A"))),
            format!("Stream ID:         {}", self.stream_id),
            format!("Attribute:\n{:?}", self.attribute),
        ].join("\n")
    }
}

impl DumpEvent for NewActor {
    fn dump(&self, replay: &Replay) -> String {
        vec![
            //String::from("========================"),
            format!("New Actor ID:      {}", self.actor_id.0),
            format!("Object:            {}", replay.objects.get(self.object_id.0 as usize).unwrap_or(&String::from("N/A"))),
            format!("Name:              {}", replay.names.get(self.name_id.unwrap_or(i32::MAX) as usize).unwrap_or(&String::from("N/A"))),
            format!("Initial Trajectory:\n{:?}", self.initial_trajectory),
        ].join("\n")
    }
}

impl DumpEvent for ActorId {
    fn dump(&self, _replay: &Replay) -> String {
        vec![
            //String::from("========================"),
            format!("Deleted Actor ID: {}", self.0),
        ].join("\n")
    }
}

fn parse_lifetimes(replay: &Replay) -> LifetimeList {
    //let replay = parse_file(filename).unwrap();
    let mut active_lifetimes: HashMap<i32, Vec<Event>> = HashMap::new();
    let mut ret: Vec<Lifetime> = vec![];
    replay.network_frames.as_ref().unwrap()
        .frames.iter().enumerate()
        .for_each(|(frame_id, fr)| {
            fr.new_actors.iter().for_each(|na| {
                // flush previous active entry to return lifetime list
                let old_lifetime = active_lifetimes.remove(&na.actor_id.0);
                if old_lifetime.as_ref().map(|zz| zz.len() > 0).unwrap_or(false) {
                    ret.push(Lifetime::from(old_lifetime.unwrap()));
                }

                // insert the new create event to a new active entry
                active_lifetimes.insert(na.actor_id.0, vec![Event::from(ChangeEvent::N(na.clone()), frame_id, fr.time)]);
            });

            fr.deleted_actors.iter().for_each(|da| {
                // Append delete event to active entry
                let entry = active_lifetimes.entry(da.0).or_insert(vec![]);
                entry.push(Event::from(ChangeEvent::D(da.clone()), frame_id, fr.time));

                // flush deleted active entry to return lifetime list
                let old_lifetime = active_lifetimes.remove(&da.0);
                if old_lifetime.as_ref().map(|zz| zz.len() > 0).unwrap_or(false) {
                    ret.push(Lifetime::from(old_lifetime.unwrap()));
                }
            });

            fr.updated_actors.iter().for_each(|ua| {
                // append update event to active entry
                let entry = active_lifetimes.entry(ua.actor_id.0).or_insert(vec![]);
                entry.push(Event::from(ChangeEvent::U(ua.clone()), frame_id, fr.time));
            });
        });

    for key in active_lifetimes.keys().map(|&nn| {nn}).collect::<Vec<i32>>() {
        let old_lifetime = active_lifetimes.remove(&key);
        if old_lifetime.as_ref().map(|zz| zz.len() > 0).unwrap_or(false) {
            ret.push(Lifetime::from(old_lifetime.unwrap()));
        }
    }

//    ret.iter().enumerate().take(50).for_each(|(i, lf)| {
//        eprintln!("\n\nLIFETIME {}", i+1);
//        lf.events.iter().for_each(|ev| {
//            eprintln!("===================================================================");
//            eprintln!("{}", ev.dump(&replay));
//            /*
//            match ev {
//                Event::N(na) => eprintln!("{}", na.dump(&replay)), //{dbg!(replay.objects.get(na.actor_id.0 as usize)); ()},
//                Event::U(ua) => eprintln!("{}", ua.dump(&replay)), //{dbg!(replay.objects.get(ua.actor_id.0 as usize)); ()},
//                Event::D(da) => eprintln!("{}", da.dump(&replay)),
//            };
//            */
//            //dbg!(ev);
//        });
//        eprintln!("-----------------\n");
//    });

    LifetimeList::from(ret)
}
fn parse_id(atr: &Attribute) -> Result<String, ()> {
    if let Attribute::UniqueId(uid) = atr {
        match &uid.remote_id {
            boxcars::RemoteId::QQ(rid) => Ok(format!("QQ-{}", rid)),
            boxcars::RemoteId::Xbox(rid) => Ok(format!("Xbox-{}", rid)),
            boxcars::RemoteId::Epic(rid) => Ok(format!("Epic-{}", rid)),
            boxcars::RemoteId::Steam(rid) => Ok(format!("Steam-{}", rid)),
            boxcars::RemoteId::PsyNet(psy_id) => Ok(format!("PsyNet-{}", psy_id.online_id)),
            boxcars::RemoteId::Switch(switch_id) => Ok(format!("Switch-{}", switch_id.online_id)),
            boxcars::RemoteId::PlayStation(psn_id) => Ok(format!("PlayStation-{}", psn_id.online_id)),
            boxcars::RemoteId::SplitScreen(split_id) => Ok(format!("SplitScreen-{}", split_id)),
        }
    } else {
        Err(())
    }
}

fn parse_actor_reference(atr: &Attribute) -> Result<i32, ()> {
    if let Attribute::ActiveActor(ActiveActor { active, actor }) = atr {
        Ok(actor.0)
    } else {
        Err(())
    }
}

fn player_id_buckets(ltl: &LifetimeList, replay: &Replay) -> HashMap<Option<String>, Vec<usize>> {

    let camera_create   = replay.objects.iter().position(|pp| pp == "TAGame.Default__CameraSettingsActor_TA").unwrap();
    let cam_to_pri      = replay.objects.iter().position(|pp| pp == "TAGame.CameraSettingsActor_TA:PRI").unwrap();
    let pri_to_unique   = replay.objects.iter().position(|pp| pp == "Engine.PlayerReplicationInfo:UniqueId").unwrap();

    let mut player_history: HashMap<Option<String>, Vec<usize>> = bucket_index(&ltl.list, |lt| {
        if !match lt.events[0].event {
            ChangeEvent::N(na) => na.object_id.0 as usize == camera_create,
            _ => false,
        } {
            return None;
        }
        let pri_attr = lt.events.iter().find_map(|cvt| {
            match &cvt.event {
                ChangeEvent::U(ua) => {
                    if ua.object_id.0 as usize == cam_to_pri {
                        Some((cvt.frame, &ua.attribute))
                    } else {
                        None
                    }
                },
                _ => None,
            }
        });
        if let Some((ref_frame_id, pri_ref)) = pri_attr {
            if let Ok(pri_actor) = parse_actor_reference(pri_ref) {
                let pri_lifetime = ltl.lookup_actor(pri_actor, ref_frame_id).expect("ACTOR ASSOC DIDN'T EXIST. PAIN");
                let unique_atr = pri_lifetime.events.iter().find_map(|pvt| {
                    match &pvt.event {
                        ChangeEvent::U(ua) => {
                            if ua.object_id.0 as usize == pri_to_unique {
                                Some(&ua.attribute)
                            } else {
                                None
                            }
                        },
                        _ => None,
                    }
                });
                if let Some(uniq_atr) = unique_atr {
                    return parse_id(uniq_atr).ok();
                }
            }
        }
        return None;
    });
    player_history.remove(&None);

    player_history
}

#[derive(Debug, Clone)]
struct BallcamResults {
    self_percent: f32,
    other_percent: f32,
}

impl BallcamResults {
    fn from(self_percent: f32, other_percent: f32) -> Self {
        Self {
            self_percent,
            other_percent,
        }
    }
}

fn ballcam_lifetimes(ltl: &LifetimeList, replay: &Replay, target_player: &str) -> BallcamResults {

    let ballcam_id = replay.objects.iter().position(|pp| pp == "TAGame.CameraSettingsActor_TA:bUsingSecondaryCamera").unwrap();

    let player_buckets = player_id_buckets(ltl, replay);
    //dbg!(&player_buckets);

    let mut self_percent = 0f32;
    let mut other_total = 0f32;
    let mut other_count = 0;
    for (pid, idx_list) in player_buckets.iter() {
        //TODO eprintln!("\n\nCHECKING: {:?}", pid);
        let mut min_time: Option<f32> = None;
        let mut max_time: Option<f32> = None;
        let mut last_time: Option<f32> = None;
        let mut last_state: Option<bool> = Some(false); // default is false i think? See notes
        let mut total = 0f32;
        let mut ballcam = 0f32;
        idx_list.iter().for_each(|&cfi| {
            ltl.list[cfi].events.iter().for_each(|ev| {
                if last_time.is_none() {
                    last_time = Some(ev.time);
                }
                min_time = Some(min_time.map(|v| v.min(ev.time)).unwrap_or(ev.time));
                max_time = Some(max_time.map(|v| v.max(ev.time)).unwrap_or(ev.time));
                match &ev.event {
                    ChangeEvent::U(ua) if ua.object_id.0 as usize == ballcam_id => {
                        //if let Some(prev_time) = last_time {
                        if last_time.is_some() && last_state.is_some() {
                            total += ev.time - last_time.unwrap();
                            if last_state.unwrap() == true {
                                ballcam += ev.time - last_time.unwrap();
                            }
                        }
                        last_time = Some(ev.time);
                        last_state = match &ua.attribute {
                            Attribute::Boolean(bval) => Some(*bval),
                            _ => None,
                        };
                        //eprintln!("{: >10.6} - {:?}", last_time.unwrap(), last_state);
                    },
                    ChangeEvent::D(_) => {
                        if last_time.is_some() && last_state.is_some() {
                            total += ev.time - last_time.unwrap();
                            if last_state.unwrap() == true {
                                ballcam += ev.time - last_time.unwrap();
                            }
                        }
                        last_time = None;
                        last_state = Some(false);
                        //eprintln!("DELETE AT: {}", ev.time);
                    }
                    _ => (),
                }
            });
        });

        // handle time to end
        if last_time.is_some() && last_state.is_some() && max_time.is_some() {
            total += max_time.unwrap() - last_time.unwrap();
            if last_state.unwrap() == true {
                ballcam += max_time.unwrap() - last_time.unwrap();
            }
        }

        /*TODO
        eprintln!("MIN:             {}", min_time.unwrap_or(-1f32));
        eprintln!("MAX:             {}", max_time.unwrap_or(-1f32));
        eprintln!("DIFF:            {}", max_time.unwrap_or(-1f32) - min_time.unwrap_or(0f32));
        eprintln!("Total Time:      {}", total);
        eprintln!("Ballcam:         {}", ballcam);
        eprintln!("Standard:        {}", total - ballcam);
        eprintln!("Ballcam percent: {}", ballcam / total * 100f32);
        */

        if pid.is_some() && pid.as_ref().unwrap() == target_player {
            self_percent = ballcam / total * 100f32;
        } else {
            other_total += ballcam / total * 100f32;
            other_count += 1;
        }

    };
    BallcamResults::from(self_percent, other_total / (other_count as f32))
}

fn main() {
    let start_time = time::Instant::now();

    let _replay_file = &dotenv::var("TEST_FILE").ok().expect("Please specify a TEST_FILE in the .env file");
    let replay_dir = &dotenv::var("REPLAY_DIR").ok().expect("Please specify a REPLAY_DIR in the .env file");
    let target_player = &dotenv::var("TARGET_PLAYER").ok().expect("Please specify a TARGET_PLAYER in the .env file");

    let replays = get_replay_list(replay_dir);
    let mut ballcam_results: Vec<(Metadata, BallcamResults)> = replays.iter().map(|rfile| {
        let replay = parse_file(&rfile).unwrap();
        let lifetimes = parse_lifetimes(&replay);
        let metadata = get_metadata(&replay);
        //TODO eprintln!("\nDOING: {:?}\n", metadata);
        (metadata, ballcam_lifetimes(&lifetimes, &replay, target_player))
    }).collect();

    ballcam_results.sort_by_key(|(md, _)| {md.date});

    //TODO dbg!(&ballcam_results, ballcam_results.len());

    //let mut deduped: Vec<(Metadata, BallcamResults)> = Vec::with_capacity(ballcam_results.len());

    let dedupe = false;

    let ballcam_results = if dedupe {
        let (mut ballcam_results, (mdp, _, totalp, cntp)): (Vec<(Metadata, BallcamResults)>, _) = ballcam_results.into_iter().fold((vec![], (None, None, (0f32, 0f32), 0)), |(mut ret, (lmd, last, sum, cnt)), e| {
            if Some(e.0.date) == last {
                (ret, (lmd, last, (sum.0 + e.1.self_percent, sum.1 + e.1.other_percent), cnt+1))
            } else {
                if last.is_some() {
                    eprintln!("{} -> {:?}, {}", last.unwrap(), sum, cnt);
                    ret.push((lmd.unwrap(), BallcamResults::from(sum.0 / (cnt as f32), sum.1 / (cnt as f32))));
                }
                else {
                    eprintln!("PAIIIINNN: {:?}", e);
                }
                (ret, (Some(e.0.clone()), Some(e.0.date), (e.1.self_percent, e.1.other_percent), 1))
            }
        });

        ballcam_results.push((mdp.unwrap(), BallcamResults::from(totalp.0 / (cntp as f32), totalp.1 / (cntp as f32))));
        ballcam_results
    } else {
        ballcam_results
    };

    //TODO dbg!(&ballcam_results, ballcam_results.len());

    let after2023: Vec<(Metadata, BallcamResults)> = ballcam_results.iter()
        .filter(|r| r.0.date >= NaiveDate::from_ymd_opt(2023, 01, 01).unwrap())
        .map(|(md, bc)| {(md.clone(), bc.clone())})
        .collect();

    let ones: Vec<(Metadata, BallcamResults)> = ballcam_results.iter()
        .filter(|r| r.0.game_mode == "TAGame.Replay_Soccar_TA-1")
        .map(|(md, bc)| {(md.clone(), bc.clone())})
        .collect();

    let twos: Vec<(Metadata, BallcamResults)> = ballcam_results.iter()
        .filter(|r| r.0.game_mode == "TAGame.Replay_Soccar_TA-2")
        .map(|(md, bc)| {(md.clone(), bc.clone())})
        .collect();

    let threes: Vec<(Metadata, BallcamResults)> = ballcam_results.iter()
        .filter(|r| r.0.game_mode == "TAGame.Replay_Soccar_TA-3")
        .map(|(md, bc)| {(md.clone(), bc.clone())})
        .collect();

    let ones2023: Vec<(Metadata, BallcamResults)> = ballcam_results.iter()
        .filter(|r| r.0.date >= NaiveDate::from_ymd_opt(2023, 01, 01).unwrap())
        .filter(|r| r.0.game_mode == "TAGame.Replay_Soccar_TA-1")
        .map(|(md, bc)| {(md.clone(), bc.clone())})
        .collect();

    let twos2023: Vec<(Metadata, BallcamResults)> = ballcam_results.iter()
        .filter(|r| r.0.date >= NaiveDate::from_ymd_opt(2023, 01, 01).unwrap())
        .filter(|r| r.0.game_mode == "TAGame.Replay_Soccar_TA-2")
        .map(|(md, bc)| {(md.clone(), bc.clone())})
        .collect();

    let threes2023: Vec<(Metadata, BallcamResults)> = ballcam_results.iter()
        .filter(|r| r.0.date >= NaiveDate::from_ymd_opt(2023, 01, 01).unwrap())
        .filter(|r| r.0.game_mode == "TAGame.Replay_Soccar_TA-3")
        .map(|(md, bc)| {(md.clone(), bc.clone())})
        .collect();

    plot_updated(ballcam_results, &format!("both-sides-ballcam-full-{}", dedupe));
    plot_updated(after2023, &format!("both-sides-ballcam-2023-later-{}", dedupe));
    plot_updated(ones, &format!("both-sides-ballcam-full-1s-{}", dedupe));
    plot_updated(twos, &format!("both-sides-ballcam-full-2s-{}", dedupe));
    plot_updated(threes, &format!("both-sides-ballcam-full-3s-{}", dedupe));
    plot_updated(ones2023, &format!("both-sides-ballcam-2023-1s-{}", dedupe));
    plot_updated(twos2023, &format!("both-sides-ballcam-2023-2s-{}", dedupe));
    plot_updated(threes2023, &format!("both-sides-ballcam-2023-3s-{}", dedupe));


    let main_duration = start_time.elapsed();
    eprintln!("Time elapsed is: {:?}", main_duration);

    //get_usage_stats();
}

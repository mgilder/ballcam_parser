use boxcars::{ActiveActor, Frame, UpdatedAttribute, UniqueId, ActorId, NewActor};
use boxcars::{ParseError, Replay, Attribute, HeaderProp};
use std::cmp::Ordering;
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

//pub mod ballcam_stats;

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
pub struct Metadata {
    pub name: Option<String>,
    pub date: NaiveDate,
    pub playlist: String,
}

impl Metadata {
    fn new(name: Option<String>, date: NaiveDate, playlist: String) -> Self {
        Self {
            name,
            date,
            playlist
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
        .find(|prop| prop.0 == "PlayerName");
    if player_name_prop.is_some() {
        if let HeaderProp::Str(pname) = &player_name_prop.unwrap().1 {
            result_name = Some(pname.clone());
        }
    }

    let game_time_prop = replay.properties
        .iter()
        .find(|prop| prop.0 == "Date").unwrap();
    if let HeaderProp::Str(gdate) = &game_time_prop.1 {
        result_date = gdate.split_whitespace().next().map(|v| NaiveDate::parse_from_str(v, "%Y-%m-%d").ok()).flatten();
    }

    let playlist_prop = replay.properties
        .iter()
        .find(|prop| prop.0 == "TeamSize").unwrap();
    if let HeaderProp::Int(tsize) = &playlist_prop.1 {
        result_mode = Some(format!("{}-{}", replay.game_type, tsize));
    }

    Metadata::new(result_name, result_date.unwrap(), result_mode.unwrap())
}

pub fn get_replay_list(dir: &str) -> Vec<String> {
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

fn fix_bytes(b: u64) -> String {
    let big_endian = b.to_be_bytes().map(|b| format!("{:02x}", b)).join("");
    let little_endian = b.to_le_bytes().map(|b| format!("{:02x}", b)).join("");
    little_endian
}

fn uid_to_string(uid: &UniqueId) -> String {
    match &uid.remote_id {
        boxcars::RemoteId::QQ(rid) => format!("qq-{}-{}", rid, uid.local_id),
        boxcars::RemoteId::Xbox(rid) => format!("xbox-{}-{}", fix_bytes(*rid), uid.local_id),
        boxcars::RemoteId::Epic(rid) => format!("epic-{}-{}", rid, uid.local_id),
        boxcars::RemoteId::Steam(rid) => format!("steam-{}-{}", rid, uid.local_id),
        boxcars::RemoteId::PsyNet(psy_id) => format!("psynet-{}-{}", fix_bytes(psy_id.online_id), uid.local_id),
        boxcars::RemoteId::Switch(switch_id) => format!("switch-{}-{}", switch_id.online_id, uid.local_id),
        boxcars::RemoteId::PlayStation(psn_id) => format!("ps4-{}-{}", psn_id.name, uid.local_id),
        boxcars::RemoteId::SplitScreen(split_id) => format!("splitscreen-{}-{}", split_id, uid.local_id),
    }
}

pub fn plot_updated(data: Vec<(Metadata, HashMap<UniqueId, PlayerResult>)>, file: &str, target_player: &str) {
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

    let mut self_series: Vec<(NaiveDate, f32)> = Vec::with_capacity(data.len());
    let mut other_series: Vec<(NaiveDate, f32)> = Vec::with_capacity(data.len());

    data.iter().for_each(|(md, hm)| {
        let mut other_top = 0f32;
        let mut other_bot = 0f32;
        for (key, val) in hm {
            if uid_to_string(key) == target_player {
                self_series.push((md.date, 100f32 * val.ballcam_active_only / val.total_time_active_only));
            } else {
                other_top += val.ballcam_active_only;
                other_bot += val.total_time_active_only;
            }
        }
        other_series.push((md.date, 100f32 * other_top / other_bot));
    });

    /*
    let self_series: Vec<(NaiveDate, f32)> = data.iter().map(|(md, lst)| {
        (md.date, lst.get(target_player).)
    }).collect();
    let other_series: Vec<(NaiveDate, f32)> = data.iter().map(|(md, lst)| {
        (md.date, lst.other_percent)
    }).collect();
    */

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

    /*
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
    */

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
            actor_map: bucket_index(&list, |lt| {Some(lt.events[0].event.actor_id())}),
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

fn bucket_index<T: Hash+Eq, F: Fn(&Lifetime) -> Option<T>>(v: &Vec<Lifetime>, func: F) -> HashMap<T, Vec<usize>> {
    v.into_iter()
        .enumerate()
        .fold(HashMap::new(), |mut ret, (ind, lt)| {
            let key = func(lt);
            if let Some(key_inner) = key {
                let entry = ret.entry(key_inner).or_insert(vec![]);
                entry.push(ind);
            }
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
    // let reservations   = replay.objects.iter().position(|pp| pp == "ProjectX.GRI_X:Reservations").unwrap() as i32;
    let reservations = get_object_id(replay, "ProjectX.GRI_X:Reservations").expect("Expected Reservations"); // TODO handle result better here
    /*
    let match_has_begun   = replay.objects.iter().position(|pp| pp == "Engine.GameReplicationInfo:bMatchHasBegun").unwrap() as i32;
    let match_is_over   = replay.objects.iter().position(|pp| pp == "Engine.GameReplicationInfo:bMatchIsOver").unwrap() as i32;
    let match_endded   = replay.objects.iter().position(|pp| pp == "TAGame.GameEvent_Soccar_TA:bMatchEnded").unwrap() as i32;
    let timed_out   = replay.objects.iter().position(|pp| pp == "Engine.PlayerReplicationInfo:bTimedOut").unwrap() as i32;
    let distracted   = replay.objects.iter().position(|pp| pp == "TAGame.PRI_TA:bIsDistracted").unwrap() as i32;
    let state_change   = replay.objects.iter().position(|pp| pp == "TAGame.GameEvent_TA:ReplicatedStateName").unwrap() as i32;

    let mut object_counts: HashMap<&String, i32> = HashMap::new();
    // */

    let mut res_changes: HashMap<UniqueId, (bool, bool)> = HashMap::new();

    //let replay = parse_file(filename).unwrap();
    let mut active_lifetimes: HashMap<i32, Vec<Event>> = HashMap::new();
    let mut ret: Vec<Lifetime> = vec![];
    //eprintln!("start frame time: {}", replay.network_frames.as_ref().unwrap().frames[0].time);
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
                //if fr.time > 20.0 && fr.time < 35.0 {
                //    *object_counts.entry(&replay.objects[na.object_id.0 as usize]).or_insert(0) += 1;
                //}
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
               
                /*
                if ua.object_id.0 == match_has_begun 
                        || ua.object_id.0 == match_is_over 
                        || ua.object_id.0 == match_endded 
                        || ua.object_id.0 == timed_out 
                        //|| ua.object_id.0 == distracted
                        || ua.object_id.0 == state_change
                        //|| ua.object_id.0 == reservations
                        {
                    eprintln!("\n\n{}", fr.time);
                    eprintln!("{}", ua.dump(replay));
                    dbg!(&ua.attribute);
                }
                if fr.time > 26.0 && fr.time < 39.0 {
                    *object_counts.entry(&replay.objects[ua.object_id.0 as usize]).or_insert(0) += 1;
                    if object_counts.get(&replay.objects[ua.object_id.0 as usize]).unwrap_or(&0) < &10 {
                        eprintln!("\n\n{}\n{}",fr.time, ua.dump(replay));
                    }
                }
                // */
                if ua.object_id.0 == reservations {
                    if let Attribute::Reservation(trev) = &ua.attribute {
                        //eprintln!("\n\ntime: {}", fr.time);
                        //dbg!(trev);
                        let res_entry = res_changes.entry(trev.unique_id.clone()).or_insert((trev.unknown1, trev.unknown2));
                        if (res_entry.0 != trev.unknown1 || res_entry.1 != trev.unknown2)
                            && !res_entry.0 && !res_entry.1 // only when was false previously
                            {
                            eprintln!("\n\ntime: {}", fr.time);
                            eprintln!("was:");
                            eprintln!("unknown1: {}", res_entry.0);
                            eprintln!("unknown2: {}", res_entry.1);
                            dbg!(trev);
                            //panic!("rejoined?");
                        }
                        res_entry.0 = trev.unknown1;
                        res_entry.1 = trev.unknown2;
                    }
                }
            });
        });

    for key in active_lifetimes.keys().map(|&nn| {nn}).collect::<Vec<i32>>() {
        let old_lifetime = active_lifetimes.remove(&key);
        if old_lifetime.as_ref().map(|zz| zz.len() > 0).unwrap_or(false) {
            ret.push(Lifetime::from(old_lifetime.unwrap()));
        }
    }


    //dbg!(object_counts);
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
/*
fn parse_id(atr: &Attribute) -> Result<String, ()> {
    if let Attribute::UniqueId(uid) = atr {
        dbg!(&uid);
        match &uid.remote_id {
            boxcars::RemoteId::QQ(rid) => Ok(format!("qq-{}", rid)),
            boxcars::RemoteId::Xbox(rid) => Ok(format!("xbox-{}", rid)),
            boxcars::RemoteId::Epic(rid) => Ok(format!("epic-{}", rid)),
            boxcars::RemoteId::Steam(rid) => Ok(format!("steam-{}", rid)),
            boxcars::RemoteId::PsyNet(psy_id) => Ok(format!("psynet-{}", psy_id.online_id)),
            boxcars::RemoteId::Switch(switch_id) => Ok(format!("switch-{}", switch_id.online_id)),
            boxcars::RemoteId::PlayStation(psn_id) => Ok(format!("ps4-{}", psn_id.online_id)),
            boxcars::RemoteId::SplitScreen(split_id) => Ok(format!("SplitScreen-{}", split_id)),
        }
    } else {
        Err(())
    }
}
*/

fn parse_actor_reference(atr: &Attribute) -> Result<i32, ()> {
    if let Attribute::ActiveActor(ActiveActor { active, actor }) = atr {
        Ok(actor.0)
    } else {
        Err(())
    }
}

//fn player_id_buckets(ltl: &LifetimeList, replay: &Replay) -> HashMap<Option<String>, Vec<usize>> {
fn player_id_buckets(ltl: &LifetimeList, replay: &Replay) -> HashMap<UniqueId, Vec<usize>> {

    let camera_create   = replay.objects.iter().position(|pp| pp == "TAGame.Default__CameraSettingsActor_TA").unwrap();
    let cam_to_pri      = replay.objects.iter().position(|pp| pp == "TAGame.CameraSettingsActor_TA:PRI").unwrap();
    let pri_to_unique   = replay.objects.iter().position(|pp| pp == "Engine.PlayerReplicationInfo:UniqueId").unwrap();

    //let mut player_history: HashMap<Option<String>, Vec<usize>> = bucket_index(&ltl.list, |lt| {
    let player_history: HashMap<UniqueId, Vec<usize>> = bucket_index(&ltl.list, |lt| {
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
                    if let Attribute::UniqueId(uid) = uniq_atr {
                        return Some(*uid.to_owned()); //parse_id(uniq_atr).ok();
                    }
                }
            }
        }
        return None;
    });
    //player_history.remove(&None);

    player_history
}


fn get_disconnect_players(ltl: &LifetimeList, replay: &Replay) -> HashMap<UniqueId, f32> {
    //eprintln!("Investigating disconnected players!!!");
    let reservations   = replay.objects.iter().position(|pp| pp == "ProjectX.GRI_X:Reservations").unwrap() as i32;
    //let gri_new   = replay.objects.iter().position(|pp| pp == "GameInfo_Soccar.GameInfo.GameInfo_Soccar:GameReplicationInfoArchetype").unwrap() as i32;


    let mut ret: HashMap<UniqueId, f32> = HashMap::new();

    /*
    ltl.list.iter().for_each(|ll| {
        if ll.events.iter().find(|ff| {
            match &ff.event {
                ChangeEvent::N(na) => na.object_id.0 == reservations,
                ChangeEvent::U(na) => na.object_id.0 == reservations,
                _ => false,
            }
        }).is_some() {
            eprintln!("\n\nSUCCESS!!!=====================\n");
            //dbg!(ll.events.iter().for_each(|aa| {dbg!(aa.event.dump(replay));}));
            eprintln!("TARGET: {}", ll.events[0].event.dump(replay));
        }
        //if let ChangeEvent::N(na) = &ll.events[0].event { true
    });
    // */
    let target_object_id = ltl.list.iter().find_map(|ll| {
        if ll.events.iter().find(|ff| {
            match &ff.event {
                ChangeEvent::U(na) => na.object_id.0 == reservations,
                _ => false,
            }
        }).is_some() {
            //eprintln!("\n\nSUCCESS!!!=====================\n");
            //dbg!(ll.events.iter().for_each(|aa| {dbg!(aa.event.dump(replay));}));
            //eprintln!("TARGET: {}", ll.events[0].event.dump(replay));
            if let ChangeEvent::N(na) = ll.events[0].event {
                Some(na.object_id.0)
            } else {
                None
            }
        } else {
            None
        }
        //if let ChangeEvent::N(na) = &ll.events[0].event { true
    });

    let mut res_changes: HashMap<UniqueId, (bool, bool)> = HashMap::new();

    bucket_index(&ltl.list, |ll| {
        if let ChangeEvent::N(na) = ll.events[0].event {
            if Some(na.object_id.0) == target_object_id {
                Some("res")
            } else {
                None
            }
        } else {
            None
        }
    }).get("res").unwrap_or(&vec![]).iter().for_each(|&ll| {
        ltl.list[ll].events.iter().for_each(|ff| {
            if let ChangeEvent::U(ua) = &ff.event {
                if ua.object_id.0 == reservations {
                    if let Attribute::Reservation(trev) = &ua.attribute {
                        if !ret.contains_key(&trev.unique_id)
                                && res_changes.contains_key(&trev.unique_id)
                                && !(trev.unknown1 && trev.unknown2)
                                && (res_changes.get(&trev.unique_id).unwrap().0     // weren't already
                                    || res_changes.get(&trev.unique_id).unwrap().1) // both false
                                {
                            /*
                            eprintln!("\n\ndisconnect!!!");
                            eprintln!("time: {}", ff.time);
                            eprintln!("prev:");
                            eprintln!("  u1: {}", res_changes.get(&trev.unique_id).unwrap().0);
                            eprintln!("  u2: {}", res_changes.get(&trev.unique_id).unwrap().1);
                            eprintln!("curr:");
                            eprintln!("  u1: {}", trev.unknown1);
                            eprintln!("  u2: {}", trev.unknown2);
                            // */
                            ret.insert(trev.unique_id.clone(), ff.time);
                        }
                        res_changes.insert(trev.unique_id.clone(), (trev.unknown1, trev.unknown2));

                        /*
                        let res_entry = res_changes.entry(trev.unique_id.clone()).or_insert((trev.unknown1, trev.unknown2));
                        if (res_entry.0 != trev.unknown1 || res_entry.1 != trev.unknown2)
                            && !res_entry.0 && !res_entry.1 // only when was false previously
                            {
                            eprintln!("\n\ntime: {}", fr.time);
                            eprintln!("was:");
                            eprintln!("unknown1: {}", res_entry.0);
                            eprintln!("unknown2: {}", res_entry.1);
                            dbg!(trev);
                        }
                        res_entry.0 = trev.unknown1;
                        res_entry.1 = trev.unknown2;
                        */
                    }
                }
            }
        });
    });

    //eprintln!("event: {}", replay.objects[target_object_id.unwrap() as usize]);

    ret
}


fn get_ping_from_cam(index: usize, ltl: &LifetimeList, replay: &Replay) -> Option<(f32, usize)> {
    let cam_to_pri      = replay.objects.iter().position(|pp| pp == "TAGame.CameraSettingsActor_TA:PRI").unwrap();
    let ping_object = replay.objects.iter().position(|pp| pp == "Engine.PlayerReplicationInfo:Ping").unwrap();
    let pri_attr = ltl.list[index].events.iter().find_map(|cvt| {
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
            let ping_raw = pri_lifetime.events.iter().find_map(|pvt| {
                match &pvt.event {
                    ChangeEvent::U(ua) => {
                        if ua.object_id.0 as usize == ping_object {
                            Some((pvt.time, &ua.attribute))
                        } else {
                            None
                        }
                    },
                    _ => None,
                }
            });
            if let Some((ping_time, ping_unwrapped)) = ping_raw {
                //dbg!(&ping_unwrapped);
                if let Attribute::Byte(ping_val) = ping_unwrapped {
                    return Some((ping_time, *ping_val as usize)); //parse_id(uniq_atr).ok();
                }
            }
        }
    }
    return None;
}

/*
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
*/

#[derive(Debug, Clone)]
pub struct BallcamResults {
    //results: HashMap<String, (f32, i32)>,
    pub results: HashMap<UniqueId, (f32, i32)>,
}

impl BallcamResults {
    //fn from(results: HashMap<String, (f32, i32)>) -> Self {
    fn from(results: HashMap<UniqueId, (f32, i32)>) -> Self {
        Self {
            results,
        }
    }
}

#[derive(Debug)]
struct FrameInfo {
    time: f32,
    frame: usize,
}

impl FrameInfo {
    fn from(time: f32, frame: usize) -> Self {
        Self {
            time,
            frame,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum GameState {
    Goal,
    Active,
    Countdown,
}

#[derive(Debug)]
struct GameStateEvent {
    variant: GameState,
    info: FrameInfo,
}

impl GameStateEvent {
    fn from(time: f32, frame: usize, variant: GameState) -> Self {
        Self {
            variant,
            info: FrameInfo::from(time, frame),
        }
    }
}

/*
impl PartialEq<BallcamEvent> for GameStateEvent {
    fn eq(&self, other: &BallcamEvent) -> bool {
        self.info == other.info
    }
}

impl PartialOrd<BallcamEvent> for GameStateEvent {
    fn partial_cmp(&self, other: &BallcamEvent) -> Option<Ordering> {
        Some(self.info.cmp(other.info))
    }
}
*/


//fn get_state_changes(ltl: &LifetimeList, replay: &Replay) -> Vec<(f32, bool)> {
fn get_state_changes(ltl: &LifetimeList, replay: &Replay) -> Vec<GameStateEvent> {
    let state_change_object = replay.objects.iter().position(|pp| pp == "TAGame.GameEvent_TA:ReplicatedStateName").unwrap() as i32;
    let countdown_event = replay.names.iter().position(|pp| pp == "Countdown").unwrap_or(99993) as i32;
    let active_event = replay.names.iter().position(|pp| pp == "Active").unwrap_or(99992) as i32;
    //dbg!(&replay.names);
    let goal_event = replay.names.iter().position(|pp| pp == "PostGoalScored").unwrap_or(99991) as i32;

    let target_object_id = ltl.list.iter().find_map(|ll| {
        if ll.events.iter().find(|ff| {
            match &ff.event {
                ChangeEvent::U(na) => na.object_id.0 == state_change_object,
                _ => false,
            }
        }).is_some() {
            if let ChangeEvent::N(na) = ll.events[0].event {
                Some(na.object_id.0)
            } else {
                None
            }
        } else {
            None
        }
    });

    //let mut state_changes: Vec<(f32, bool)> = Vec::new();
    let mut real_state_changes: Vec<GameStateEvent> = Vec::new();

    bucket_index(&ltl.list, |ll| {
        if let ChangeEvent::N(na) = ll.events[0].event {
            if Some(na.object_id.0) == target_object_id {
                Some(0)
            } else {
                None
            }
        } else {
            None
        }
    }).values().next().unwrap_or(&vec![]).iter().for_each(|&ll| {
        ltl.list[ll].events.iter().for_each(|ff| {
            if let ChangeEvent::U(ua) = &ff.event {
                if ua.object_id.0 == state_change_object {
                    if let &Attribute::Int(new_state) = &ua.attribute {
                        match new_state {
                            x if x == countdown_event => real_state_changes.push(GameStateEvent::from(ff.time, ff.frame, GameState::Countdown)),
                            x if x == active_event => real_state_changes.push(GameStateEvent::from(ff.time, ff.frame, GameState::Active)),
                            x if x == goal_event => real_state_changes.push(GameStateEvent::from(ff.time, ff.frame, GameState::Goal)),
                            _ => (),
                        }
                        //let current_bool = new_state == active_event;  // for exclude kickoff
                        //let current_bool = new_state != goal_event;       // for include kickoff
                        /*
                        if match state_changes.last() {
                            Some((_last_time, last_state)) => *last_state != current_bool,
                            None => true,
                        } {
                            state_changes.push((ff.time, current_bool));
                        }
                        */
                    }
                }
            }
        });
    });
/*
    let min_time = real_state_changes.first().unwrap().0;
    let max_time = real_state_changes.last().unwrap().0;

    let active_time = real_state_changes.iter().enumerate().skip(1).fold(0f32, |acc, (i, ee)| {
        acc + if state_changes[i-1].1 {
            ee.0 - state_changes[i-1].0
        } else {0f32}
    });

    eprintln!("total_time: {}", max_time - min_time);
    eprintln!("active_time:    {}", active_time);
    eprintln!("inactive_time: {}", max_time - min_time - active_time);
    */
    //dbg!(&state_changes);
    //dbg!(&real_state_changes);
    real_state_changes
}

#[derive(Debug, Clone)]
pub struct PlayerResult {
    pub swaps_all: i32,
    pub swaps_with_freeze: i32,
    pub swaps_active_only: i32,
    pub ballcam_all: f32,
    pub ballcam_with_freeze: f32,
    pub ballcam_active_only: f32,
    pub total_time_all: f32,
    pub total_time_with_freeze: f32,
    pub total_time_active_only: f32,
}

impl PlayerResult {
    fn new() -> Self {
        Self {
            swaps_all: 0,
            swaps_with_freeze: 0,
            swaps_active_only: 0,
            ballcam_all: 0f32,
            ballcam_with_freeze: 0f32,
            ballcam_active_only: 0f32,
            total_time_all: 0f32,
            total_time_with_freeze: 0f32,
            total_time_active_only: 0f32,
        }
    }

    fn update(&mut self, last_time: f32, ballcam_was_on: bool, new_ballcam: bool, cur_time: f32, last_game_state: &GameState) {
        let delta = cur_time - last_time;
//        eprintln!("Current time: {}", cur_time);
//        eprintln!("Prev time:    {}", last_time);
//        eprintln!("Found delta: {}", delta);
//        eprintln!("Old total: {}", self.total_time_all);
        //eprintln!("OLD");
        //dbg!(&self);
        self.total_time_all += delta;
        if ballcam_was_on {
            self.ballcam_all += delta;
        }
        if new_ballcam != ballcam_was_on && delta > 0.00001 {
            self.swaps_all += 1;
        }
        if last_game_state != &GameState::Goal {   // include countdown/freeze phase
            self.total_time_with_freeze += delta;
            if ballcam_was_on {
                self.ballcam_with_freeze += delta;
            }
            if new_ballcam != ballcam_was_on && delta > 0.00001 {
                self.swaps_with_freeze += 1;
            }
        }
        if last_game_state == &GameState::Active {   // only count active time
            self.total_time_active_only += delta;
            if ballcam_was_on {
                self.ballcam_active_only += delta;
            }
            if new_ballcam != ballcam_was_on && delta > 0.00001 {
                self.swaps_active_only += 1;
            }
        }
        //eprintln!("New total: {}", self.total_time_all);
        //eprintln!("NEW");
        //dbg!(&self);
    }
}

/*
fn get_no_replay_time(new_time: f32, new_state: f32, last_state: Option<bool>, last_time: Option<f32>, game_state_change: &Vec<(f32, bool)>) -> (f32, bool) {
    let current_state = game_state_change.partition_point(|ee| {ee.0 <= new_time}) - 1;
}
*/

#[derive(Debug)]
enum BallcamVariant {
    Start,
//    Create,
//    Delete,
    Update(bool),
    Disconnect,
}


#[derive(Debug)]
struct BallcamEvent {
    info: FrameInfo,
    variant: BallcamVariant,
}

impl BallcamEvent {
    fn from(frame: usize, time: f32, variant: BallcamVariant) -> Self {
        Self {
            info: FrameInfo::from(time, frame),
            variant
        }
    }
}

/*
impl PartialEq<GameStateEvent> for BallcamEvent {
    fn eq(&self, other: &GameStateEvent) -> bool {
        self.info == other.info
    }
}

impl PartialOrd<GameStateEvent> for BallcamEvent {
    fn partial_cmp(&self, other: &GameStateEvent) -> Option<Ordering> {
        Some(self.info.cmp(other.info))
    }
}
*/

//fn get_ballcam_list(ltl: &LifetimeList, replay: &Replay, player_buckets: &HashMap<UniqueId, &Vec<usize>>) -> HashMap<UniqueId, Vec<BallcamEvent>> {
fn get_ballcam_list(ltl: &LifetimeList, replay: &Replay, pid: &UniqueId, idx_list: &Vec<usize>, disconnect_time: Option<&f32>) -> Vec<BallcamEvent> {
    let mut ret: Vec<BallcamEvent> = Vec::new();
    
    let ballcam_id = replay.objects.iter().position(|pp| pp == "TAGame.CameraSettingsActor_TA:bUsingSecondaryCamera").unwrap() as i32;
    //let camera_create   = replay.objects.iter().position(|pp| pp == "TAGame.Default__CameraSettingsActor_TA").unwrap() as i32;

    //let mut min_time: Option<f32> = None;
    let mut max_time: Option<f32> = None;
    let mut max_frame: Option<usize> = None;
//    let mut last_time: Option<f32> = None;
//    let mut last_state: Option<bool> = Some(false); // default is false i think? See notes
//    let mut swaps = 0;
//    let mut swap_times: Vec<(f32, String)> = vec![];
//    let mut total = 0f32;
//    let mut ballcam = 0f32;
    let mut is_disconnected = false;
    //let mut actor_exists = false;

    idx_list.iter().for_each(|&cfi| {
        ltl.list[cfi].events.iter().enumerate().for_each(|(index, ev)| {
            if ret.len() == 0 {
                ret.push(BallcamEvent::from(ev.frame, ev.time, BallcamVariant::Start));
            }
            if !is_disconnected {
                max_time = Some(max_time.map(|v| v.max(ev.time)).unwrap_or(ev.time));
                max_frame = Some(max_frame.map(|v| v.max(ev.frame)).unwrap_or(ev.frame));
            }
            if disconnect_time.is_some() && ev.time >= *disconnect_time.unwrap() {
                is_disconnected = true;
                return;
            }
            match &ev.event {
//                ChangeEvent::N(na) if na.object_id.0 == camera_create => {
//                    if !actor_exists {
//                        ret.push(BallcamEvent::from(ev.frame, ev.time, BallcamVariant::Create));
//                        actor_exists = true;
//                    }
//                }
                ChangeEvent::U(ua) if ua.object_id.0 == ballcam_id => {
                    if let Attribute::Boolean(u_state) = &ua.attribute {
                        ret.push(BallcamEvent::from(ev.frame, ev.time, BallcamVariant::Update(*u_state)));
                    }
                },
//                ChangeEvent::D(_) => {
//                    ret.push(BallcamEvent::from(ev.frame, ev.time, BallcamVariant::Delete));
//                    actor_exists = false;
//                }
                _ => (),
            }
        });
    });
    ret.push(BallcamEvent::from(max_frame.unwrap(), max_time.unwrap(), BallcamVariant::Disconnect));

    ret.sort_by_key(|rr| rr.info.frame);

    //eprintln!("\n\nplayer: {:?}", pid);
    //dbg!(&ret);
    //eprintln!("^^ that was player: {:?}\n\n", pid);
    return ret;
}

fn new_ballcam_lifetimes(ltl: &LifetimeList, replay: &Replay) -> HashMap<UniqueId, PlayerResult> {
    let mut results: HashMap<UniqueId, PlayerResult> = HashMap::new();

    let player_buckets = player_id_buckets(ltl, replay);
    let disconnect_players = get_disconnect_players(ltl, replay);
    let game_state_changes: Vec<GameStateEvent> = get_state_changes(ltl, replay);
    for (pid, idx_list) in player_buckets.iter() {
        let ballcam_events = get_ballcam_list(ltl, replay, pid, idx_list, disconnect_players.get(pid));
        if let Some(res) = process_ballcam(ltl, replay, pid, &ballcam_events, &game_state_changes) {
            results.insert(pid.clone(), res);
        }
    }
/*
    eprintln!("New Results!!!");
    dbg!(&results);
    for (r, v) in results.iter() {
        eprintln!("\nPlayer:");
        dbg!(&r);
        dbg!(&v);
        eprintln!("FULL BALLCAM %:     {}%", 100f32 * v.ballcam_all / v.total_time_all);
        eprintln!("NON-GOAL BALLCAM %: {}%", 100f32 * v.ballcam_with_freeze / v.total_time_with_freeze);
        eprintln!("ACTIVE BALLCAM %:   {}%", 100f32 * v.ballcam_active_only / v.total_time_active_only);
    }
*/
    results
}

fn process_ballcam(ltl: &LifetimeList, replay: &Replay, pid: &UniqueId, ball_events: &Vec<BallcamEvent>, game_events: &Vec<GameStateEvent>) -> Option<PlayerResult> {
    //eprintln!("\n\n\nProcessing Ballcam!!! for {:?}", pid);
    let mut ret = PlayerResult::new();
    let mut current_ballcam = false;
    let mut current_game: GameState = GameState::Countdown;
    let mut ball_index = 0;
    let mut game_index = 0;
    //let mut last_ball_time: Option<f32> = None;
    //let mut last_game_time: Option<f32> = None;
    let mut last_time: f32 = ball_events[0].info.time.min(game_events[0].info.time);
    //dbg!(last_time);
    while ball_index < ball_events.len() {
        if game_index == game_events.len() || ball_events[ball_index].info.frame < game_events[game_index].info.frame {
            let next_bc = match ball_events[ball_index].variant {
                BallcamVariant::Start => false,
                BallcamVariant::Update(vv) => vv,
                BallcamVariant::Disconnect => current_ballcam,
            };
            ret.update(last_time, current_ballcam, next_bc, ball_events[ball_index].info.time, &current_game);
            //eprintln!("Next event is ballcam!");
            //dbg!(&ball_events[ball_index]);
            /*
            current_ballcam = match ball_events[ball_index].variant {
                BallcamVariant::Start => false,
                BallcamVariant::Update(vv) => vv,
                BallcamVariant::Disconnect => false,
            };
            */
            current_ballcam = next_bc;
            last_time = ball_events[ball_index].info.time;
            ball_index += 1;
        } else {
            ret.update(last_time, current_ballcam, current_ballcam, game_events[game_index].info.time, &current_game);
            //eprintln!("Next event is game!");
            //dbg!(&game_events[game_index]);
            last_time = game_events[game_index].info.time;
            current_game = game_events[game_index].variant;
            game_index += 1;
        }
    }
    
    /*
    eprintln!("Player: {:?}", pid);
    dbg!(&ret);
    eprintln!("FULL BALLCAM %:     {}%", 100f32 * ret.ballcam_all / ret.total_time_all);
    eprintln!("NON-GOAL BALLCAM %: {}%", 100f32 * ret.ballcam_with_freeze / ret.total_time_with_freeze);
    eprintln!("ACTIVE BALLCAM %:   {}%", 100f32 * ret.ballcam_active_only / ret.total_time_active_only);
    eprintln!("all    SWAPS:   {}%", ret.swaps_all);
    eprintln!("freeze SWAPS:   {}%", ret.swaps_with_freeze);
    eprintln!("active SWAPS:   {}%", ret.swaps_active_only);
    eprintln!("DONE!!!");
    // */
    Some(ret)
}


fn ballcam_lifetimes(ltl: &LifetimeList, replay: &Replay) -> BallcamResults {

    let ballcam_id = replay.objects.iter().position(|pp| pp == "TAGame.CameraSettingsActor_TA:bUsingSecondaryCamera").unwrap();

    //let ping_object = replay.objects.iter().position(|pp| pp == "Engine.PlayerReplicationInfo:Ping").unwrap();

    let player_buckets = player_id_buckets(ltl, replay);
    //dbg!(&player_buckets);
    let disconnect_players = get_disconnect_players(ltl, replay);
    //get_state_changes(ltl, replay);
    //new_ballcam_lifetimes(ltl, replay);
   
    //let mut results: HashMap<String, (f32, i32)> = HashMap::new();
    let mut results: HashMap<UniqueId, (f32, i32)> = HashMap::new();

    //let mut self_percent = 0f32;
    //let mut other_total = 0f32;
    //let mut other_count = 0;
    for (pid, idx_list) in player_buckets.iter() {
        get_ballcam_list(ltl, replay, pid, idx_list, disconnect_players.get(pid));
        //TODO eprintln!("\n\nCHECKING: {:?}", pid);
        let mut min_time: Option<f32> = None;
        let mut max_time: Option<f32> = None;
        let mut last_time: Option<f32> = None;
        let mut last_state: Option<bool> = Some(false); // default is false i think? See notes
        let mut swaps = 0;
        let mut swap_times: Vec<(f32, String)> = vec![];
        let mut total = 0f32;
        let mut ballcam = 0f32;

        let disconnect_time = disconnect_players.get(pid);

        idx_list.iter().for_each(|&cfi| {
            //dbg!(get_ping_from_cam(cfi, ltl, replay));
            ltl.list[cfi].events.iter().enumerate().for_each(|(index, ev)| {
                if disconnect_time.is_some() && ev.time >= *disconnect_time.unwrap() {
                    return;
                }
                //if pid.eq(&UniqueId { system_id: 1, remote_id: boxcars::RemoteId::Steam(76561198022491694), local_id: 0 }) {
                    //eprintln!("---------------------------------------------");
                    //eprintln!("{}", ev.dump(&replay));
                //}
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
                            if ev.time - last_time.unwrap() > 0.0001 && ua.attribute != Attribute::Boolean(last_state.unwrap()) && index != 0 {
                                swaps += 1;
                                swap_times.push((ev.time, format!("{:?}", ua.attribute)));
                            } else {
                                //dbg!(index, &ua.attribute, last_state, ev.time - last_time.unwrap());
                            }
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
                        // eprintln!("DELETE EVENT AT {} for {:?}", ev.time, pid);
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

        // /*TODO
        eprintln!("\nPlayer:");
        dbg!(&pid);
        eprintln!("MIN:             {}", min_time.unwrap_or(-1f32));
        eprintln!("MAX:             {}", max_time.unwrap_or(-1f32));
        eprintln!("DIFF:            {}", max_time.unwrap_or(-1f32) - min_time.unwrap_or(0f32));
        eprintln!("Total Time:      {}", total);
        eprintln!("Ballcam:         {}", ballcam);
        eprintln!("Standard:        {}", total - ballcam);
        eprintln!("Ballcam percent: {}", ballcam / total * 100f32);
        eprintln!("Swaps:           {}", swaps);
        //dbg!(&swap_times);
        // */
        //dbg!(&pid);
        //dbg!(&swap_times);

        //results.insert(pid.to_owned().unwrap_or(format!("no-pid-{:?}", time::Instant::now())), (ballcam / total * 100f32, swaps));
        results.insert(pid.to_owned(), (ballcam / total * 100f32, swaps));
        /*
        if pid.is_some() && pid.as_ref().unwrap() == target_player {
            self_percent = ballcam / total * 100f32;
        } else {
            other_total += ballcam / total * 100f32;
            other_count += 1;
        }
        */
    };
    //BallcamResults::from(self_percent, other_total / (other_count as f32))
    BallcamResults::from(results)
}


pub fn parse_replay_file(replay_file: &str) -> Result<(Metadata, HashMap<UniqueId, PlayerResult>), ()> {
    let replay = parse_file(&replay_file).map_err(|e| {
        eprintln!("\nHIDDEN ERROR:\n{}\n\n", e);
        ()
    })?;
    //let replay = parse_file(&replay_file).unwrap();
    let lifetimes = parse_lifetimes(&replay);
    /*
    for l in lifetimes.list.iter() {
        let object = match &l.events[0].event {
                ChangeEvent::N(ee) => format!("{}-{}", &replay.objects[ee.object_id.0 as usize], ee.name_id.map(|v| replay.names[v as usize].as_str()).unwrap_or("noname")),
                ChangeEvent::D(ee) => {panic!("AHJHH delete"); String::from("DELETE FIRST!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!")},
                ChangeEvent::U(ee) => {panic!("Aadfhdhfas update"); format!("{}-{}", &replay.objects[ee.object_id.0 as usize], "UPDATE FIRST!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!") },
        };
        println!("{} -> {:?} count", object, l.events.len());
    }
    */
    let metadata = get_metadata(&replay);
    let bresults = new_ballcam_lifetimes(&lifetimes, &replay);
    //ballcam_lifetimes(&lifetimes, &replay);
    //eprintln!("\nDOING: {}, {:?}", replay_file, metadata);
    //eprintln!("RESULTS:\n {:?}\n", bresults);
    Ok((metadata, bresults))
}

pub fn reservation_stats(replay_file: &str, results: &mut HashMap<(Option<(bool, bool)>, (bool, bool)), i64>) -> Result<(), ()> {
    let replay = parse_file(&replay_file).map_err(|e| {
        eprintln!("\nHIDDEN ERROR:\n{}\n\n", e);
        ()
    })?;

    let reservations = get_object_id(&replay, "ProjectX.GRI_X:Reservations").ok_or(())?;

    let mut res_changes: HashMap<UniqueId, (bool, bool)> = HashMap::new();

    replay.network_frames.as_ref().unwrap()
        .frames.iter().enumerate()
        .for_each(|(frame_id, fr)| {
            fr.updated_actors.iter().for_each(|ua| {
                if ua.object_id.0 == reservations {
                    if let Attribute::Reservation(trev) = &ua.attribute {
                        //eprintln!("\n\ntime: {}", fr.time);
                        //dbg!(trev);
                        if !res_changes.contains_key(&trev.unique_id) {
                            *results.entry((None, (trev.unknown1, trev.unknown2))).or_insert(0) += 1;
                        }
                        let res_entry = res_changes.entry(trev.unique_id.clone()).or_insert((trev.unknown1, trev.unknown2));
                        if (res_entry.0 != trev.unknown1 || res_entry.1 != trev.unknown2)
                            // && !res_entry.0 && !res_entry.1 // only when was false previously
                            {
                                /*
                            eprintln!("\n\ntime: {}", fr.time);
                            eprintln!("was:");
                            eprintln!("unknown1: {}", res_entry.0);
                            eprintln!("unknown2: {}", res_entry.1);
                            dbg!(trev);
                            */
                            //panic!("rejoined?");
                            *results.entry(( Some(*res_entry), (trev.unknown1, trev.unknown2) )).or_insert(0) += 1;
                        }
                        res_entry.0 = trev.unknown1;
                        res_entry.1 = trev.unknown2;
                    }
                }
            });
        });

    Ok(())
}
/*
pub fn parse_replay_file(replay_file: &str) -> Result<(Metadata, BallcamResults), ()> {
    let replay = parse_file(&replay_file).map_err(|e| {
        eprintln!("\nHIDDEN ERROR:\n{}\n\n", e);
        ()
    })?;
    //let replay = parse_file(&replay_file).unwrap();
    let lifetimes = parse_lifetimes(&replay);
    let metadata = get_metadata(&replay);
    let bresults = ballcam_lifetimes(&lifetimes, &replay);
    //eprintln!("\nDOING: {}, {:?}", replay_file, metadata);
    //eprintln!("RESULTS:\n {:?}\n", bresults);
    Ok((metadata, bresults))
}
*/

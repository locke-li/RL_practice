use std::cmp::{ min, max };
use std::collections::{BTreeMap, HashSet};
use std::collections::btree_map::Entry::{ Vacant, Occupied };
use std::error::Error;
use rand::prelude::*;

type Vec2 = (i32, i32);
type State = (Vec2, Vec2);
type Action = Vec2;
// type StateAction = (State, Action);
// type EpisodeStep = (State, Action, i32);

struct Field {
    pub boundary:Vec<Vec2>,//index:y, value:(x_min, x_max)
    pub start_line:i32,//min-y-row
    pub finish_line:i32,//max-x-column
    pub corner:i32,
}

struct ControlInfo {
    pub max_episode:usize,
    pub episode_check_interval:usize,
    pub epsilon:f64,
    pub gamma:f64,
    pub estimator:i32,
    pub horizon:i32,
}

struct AgentInfo {
    pub velocity_max:i32,
    pub action:Action,
    pub a_space:(i32, f32),
    pub step_reward:f64,
    pub p_vel_inc0:f64,
}

struct Agent<'a> {
    pub info:&'a AgentInfo,
    pub velocity:Vec2,
    pub position:Vec2,
}

struct Episode {
    rng:ThreadRng,
    pub state:Vec<State>,
    pub action:Vec<Action>,
}

struct Graph<'a> {
    pub g:f64,
    pub w:f64,
    pub q:BTreeMap<State, BTreeMap<Action, (f64, f64)>>,
    pub p_ref:&'a mut Policy,
}

struct Policy {
    state_action:BTreeMap<State, Action>,
}

impl Field {
    fn new() -> Self {
        Self { boundary:Vec::new(), start_line:0, finish_line:0, corner:0 }
    }

    fn append_row(&mut self, x_range:(i32, i32), y_range:i32) -> &mut Self {
        // self.finish_line = max(x_range.1, self.finish_line);
        for _ in 0..y_range {
            self.boundary.push(x_range);
        }
        self
    }

    fn setup_v1(&mut self) {
        self.append_row((3, 8), 3)
            .append_row((2, 8), 6)
            .append_row((1, 8), 8)
            .append_row((0, 8), 7)
            .append_row((0, 9), 1)
            .append_row((0, 16), 2)
            .append_row((1, 16), 1)
            .append_row((2, 16), 2)
            .append_row((3, 16), 1);
        self.finish_line = 16;
        self.corner = 25;
    }

    fn setup_v2(&mut self) {
        self.append_row((0, 23), 3);
        for k in 1..=15 {
            self.append_row((k, 23), 1);
        }
        self.append_row((15, 24), 1)
            .append_row((15, 26), 1)
            .append_row((15, 27), 1)
            .append_row((15, 30), 1)
            .append_row((14, 32), 1)
            .append_row((13, 32), 1)
            .append_row((12, 32), 4)
            .append_row((13, 32), 1)
            .append_row((14, 32), 1)
            .append_row((17, 32), 1);
        self.finish_line = 32;
        self.corner = 20;
    }

    fn is_outside(&self, p:&Vec2) -> bool {
        let row = match self.boundary.get(p.1 as usize) {
            Some(v) => v,
            None => return true,
        };
        p.0 < row.0 || p.0 > row.1
    }

    fn reset_if_outside(&self, p:&mut Vec2, v:&mut Vec2) {
        let row = match self.boundary.get(p.1 as usize) {
            Some(v) => v,
            None => return self.reset_to_start(p, v),
        };
        if p.0 < row.0 || p.0 > row.1 { self.reset_to_start(p, v) }
    }

    fn reset_to_start(&self, p:&mut Vec2, v:&mut Vec2) {
        *v = (0, 0);
        p.1 = self.start_line;
        let row = &self.boundary[self.start_line as usize];
        p.0 = if p.0 < row.0 { row.0 }
        else if p.0 > row.1 { row.1 }
        else { p.0 };
    }

    fn crossed_finish_line(&self, p:&Vec2) -> bool {
        p.0 >= self.finish_line
    }

    fn sample_start(&self) -> Vec2 {
        let row = self.boundary[self.start_line as usize];
        ((row.0 + row.1) / 2, self.start_line)
    }

    fn print(&self) {
        let y_max = self.boundary.len() - 1;
        for (i, (x_min, x_max)) in self.boundary.iter().rev().enumerate() {
            for _ in 0..*x_min {
                print!("       ");
            }
            for k in *x_min..=*x_max {
                print!("|{:02},{:02}|", k, y_max - i);
            }
            println!();
        }
    }
}

impl AgentInfo {
    fn setup(&mut self) {
        let r = self.action.1 - self.action.0 + 1;
        self.a_space = (r, (r * r - 1) as f32);
    }
}

impl<'a> Agent<'a> {
    fn new(info:&'a AgentInfo) -> Self {
        Self { velocity:(0, 0), position:(0, 0), info }
    }

    fn action(&mut self, v_a:&Action) -> (&mut Vec2, &mut Vec2) {
        let v_max = self.info.velocity_max;
        let v_min = -v_max;
        let v = &mut self.velocity;
        v.0 = min(max(v.0 + v_a.0, v_min), v_max);
        v.1 = min(max(v.1 + v_a.1, v_min), v_max);
        let p = &mut self.position;
        p.0 = p.0 + v.0;
        p.1 = p.1 + v.1;
        (p, v)
    }

    fn state(&self) -> State {
        (self.position, self.velocity)
    }
}

impl Episode {
    fn new() -> Self {
        Self { state:Vec::new(), action:Vec::new(), rng:rand::thread_rng() }
    }

    fn step(b:&mut Policy, f:&Field, a:&mut Agent, c_info:&ControlInfo, rng:&mut ThreadRng) -> (State, Action, bool) {
        let s = a.state();
        let info = a.info;
        let a_min = info.action.0;
        let (act_r, act_s)= info.a_space;
        let v0 = -(s.1).0;
        let v1 = -(s.1).1;
        let v_zero = v0 == 0 && v1 == 0;
        let act:Action;
        let r:f64 = rng.gen();
        if r < info.p_vel_inc0 && !v_zero {
            act = (0, 0);
        }
        else {
            let r:f64 = rng.gen();
            if r > c_info.epsilon {
                //equiprobable explore
                let mut aa = (rng.gen::<f32>() * act_s) as i32;
                let skip = v0 - a_min + (v1 - a_min) * act_r;
                if aa >= skip {//velocity will become (0, 0)
                    aa += 1;
                }
                act = (aa % act_r + a_min, aa / act_r + a_min);
            }
            else {
                //greedy with policy
                act = match b.state_action.get(&s) {
                    Some(v) => *v,
                    None => {
                        let y = (s.0).1;
                        if y > f.corner { (1, 0) }
                        else { (0, 1) }
                    },
                };
            }
        }
        let (p, v) = a.action(&act);
        f.reset_if_outside(p, v);
        (s, act, f.crossed_finish_line(p))
    }

    fn generate(&mut self, b:&mut Policy, f:&Field, a:&mut Agent, c_info:&ControlInfo) {
        let state = &mut self.state;
        let action = &mut self.action;
        state.clear();
        action.clear();
        let start = &f.boundary[f.start_line as usize];
        a.velocity = (0, 0);
        let r = self.rng.gen_range(start.0..=start.1);
        a.position = (r, f.start_line);
        let mut finish = false;
        while !finish {
            let s:State;
            let act:Action;
            (s, act, finish) = Episode::step(b, f, a, c_info, &mut self.rng);
            state.push(s);
            action.push(act);
            // println!("{:?}:{:?}->{:?}", s, act, a.state());
        }
        // println!("episode generated");
    }
}

impl<'a> Graph<'a> {
    fn new(p_ref:&'a mut Policy) -> Self {
        Self { q:BTreeMap::new(), g:0.0, w:1.0, p_ref }
    }

    fn mc_control(&mut self, ep:&Episode, a_info:&AgentInfo, c_info:&ControlInfo, b:Option<&Graph>) {
        match c_info.estimator {
            //weighted importance sampling
            0 => self.mc_control_wis(ep, a_info, c_info, b),
            //weighted truncated importance sampling
            1 => self.mc_control_wtis(ep, a_info, c_info, b),
            _ => todo!()
        }
    }

    fn mc_control_wis(&mut self, ep:&Episode, a_info:&AgentInfo, c_info:&ControlInfo, b:Option<&Graph>) {
        for k in (0..ep.state.len()).rev() {
            let s = &ep.state[k];
            let a = &ep.action[k];
            let r = k as f64 * a_info.step_reward;
            self.g += r;
            let a_map = match self.q.entry(*s) {
                Vacant(v) => v.insert(BTreeMap::new()),
                Occupied(v) => v.into_mut(),
            };
            let q = match a_map.entry(*a) {
                Vacant(v) => v.insert((0.0, 0.0)),
                Occupied(v) => v.into_mut(),
            };
            q.1 += self.w;
            q.0 += self.w * (self.g - q.0) / q.1;
            let a_match = match self.improve_policy(s) {
                Some(v) => v == a,
                None => false,
            };
            match b {
                Some(v) => {
                    if !a_match { return }
                    self.w *= 1.0 / v.p_epsilon(s, a, c_info);
                },
                None => {}
            }
        }
    }

    fn mc_control_wtis(&mut self, ep:&Episode, a_info:&AgentInfo, c_info:&ControlInfo, b:Option<&Graph>) {
        
    }

    fn improve_policy(&mut self, s:&State) -> Option<&Action> {
        let p = &mut *self.p_ref;
        let a_map = match self.q.get(s) {
            Some(v) => v,
            None => return None,
        };
        let (a, _) = a_map.iter().max_by(|(_, (q0, _)), (_, (q1, _))| q0.total_cmp(q1)).unwrap();
        p.state_action.insert(*s, *a);
        Some(a)
    }

    fn p_epsilon(&self, s:&State, a:&Action, info:&ControlInfo) -> f64 {
        let c_a = match self.q.get(s) {
            Some(v) => v.len() as f64,
            None => return 0.0,
        };
        match self.p_ref.state_action.get(s) {
            Some(v) => {
                let ep = info.epsilon;
                if v == a { 1.0 - ep + ep / c_a }
                else { ep / c_a }
            },
            None => return 0.0,
        }
    }

    fn print_policy_sample(&self, f:&Field, info:&AgentInfo, msg:&str, p_start:Vec2) {
        println!("{}", msg);
        let map = &self.p_ref.state_action;
        let mut visit:HashSet<Vec2> = HashSet::new();
        visit.insert(p_start);
        let mut agent = Agent::new(info);
        agent.position = p_start;
        loop {
            let act = match map.get(&agent.state()) {
                Some(v) => v,
                None => {
                    println!("state not found {:?}", &agent.state());
                    break
                }
            };
            agent.action(&act);
            let p = &agent.position;
            if visit.contains(p) { break }
            visit.insert(*p);
            if f.crossed_finish_line(p) || f.is_outside(p) { break }
        }
        let y_max = (f.boundary.len() - 1) as i32;
        for (i, (x_min, x_max)) in f.boundary.iter().rev().enumerate() {
            for _ in 0..*x_min {
                print!("   ");
            }
            for k in *x_min..=*x_max {
                if visit.contains(&(k, y_max - i as i32)) { print!("|+|") }
                else { print!("| |"); }
            }
            println!();
        }
    }
}

impl Policy {
    fn new() -> Self {
        Self { state_action:BTreeMap::new() }
    }
}

fn iteration(c_info:&ControlInfo, a:&mut Agent, f:&Field, b:&mut Graph, pi:&mut Graph) {
    let mut ep = Episode::new();
    let mut ep_c = 0;
    let a_info = a.info;
    let sample_start = f.sample_start();
    while ep_c < c_info.max_episode {
        let mut ep_cc = 0;
        while ep_cc < c_info.episode_check_interval {
            ep_cc += 1;
            ep.generate(b.p_ref, f, a, c_info);
            b.mc_control(&ep, a.info, c_info, None);
            pi.mc_control(&ep, a.info, c_info, Some(b));
        }
        b.print_policy_sample(f, a_info, "b:", sample_start);
        pi.print_policy_sample(f, a_info, "pi:", sample_start);
        ep_c += ep_cc;
    }
}

pub fn run() -> Result<(), Box<dyn Error>> {
    let mut f = Field::new();
    f.setup_v1();
    f.print();
    let mut a_info = AgentInfo {
        velocity_max:5, action:(-1, 1), step_reward:-1.0,
        p_vel_inc0:0.1, a_space:(0, 0.0),
    };
    a_info.setup();
    let c_info = ControlInfo {
        max_episode:10000, episode_check_interval:1000,
        epsilon:0.2, gamma:0.9,
        estimator:0, horizon:4,
    };
    let mut agent = Agent::new(&a_info);
    let mut b = Policy::new();
    let mut pi = Policy::new();
    let mut g_b = Graph::new(&mut b);
    let mut g_pi = Graph::new(&mut pi);
    iteration(&c_info, &mut agent, &f, &mut g_b, &mut g_pi);
    Ok(())
}
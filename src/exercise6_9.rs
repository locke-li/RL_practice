use std::cmp::{ min, max };
use std::collections::HashSet;
use std::error::Error;
use std::time::Instant;
use rand::prelude::*;

use crate::nd_vec::NdVec2;

type Vec2 = (i32, i32);
type State = Vec2;
type Action = Vec2;
// type StateAction = (State, Action);
// type EpisodeStep = (State, Action, i32);

struct World {
    pub size:(usize, usize),
    pub start:Vec2,
    pub goal:Vec2,
    pub wind:Vec<i32>,
}

struct ControlInfo {
    pub max_episode:usize,
    pub episode_check:usize,
    pub epsilon:f64,
    pub alpha:f64,
    pub stochastic_wind:bool,
}

struct AgentInfo {
    pub action:NdVec2<i32>,
    pub no_stay:bool,
    pub step_reward:f64,
}

struct Agent<'a> {
    pub info:&'a AgentInfo,
    pub position:Vec2,
    pub rng:ThreadRng,
}

struct Graph<'a> {
    pub q:NdVec2<NdVec2<f64>>,//<state:<Action: ActionValue>>
    pub p_ref:&'a mut Policy,
}

struct Policy {
    p:NdVec2<Action>,
}

impl World {
    fn is_terminal(&self, p:&Vec2) -> bool {
        self.goal == *p
    }
}

impl<'a> Agent<'a> {
    fn new(info:&'a AgentInfo) -> Self {
        Self { info, position:(0, 0), rng:rand::thread_rng() }
    }

    fn state(&self) -> State {
        self.position
    }

    fn reset(&mut self, p:&Vec2) {
        self.position = *p;
    }

    fn action(&mut self, a:&Vec2, w:&World, c_info:Option<&ControlInfo>) -> (State, f64, State) {
        let ss = &mut self.position;
        let s = *ss;
        let x_max = (w.size.0 - 1) as i32;
        let y_max = (w.size.1 - 1) as i32;
        let wind = w.wind[s.0 as usize] + 
            match c_info {
                Some(v) => {
                    if v.stochastic_wind {
                        let r:f64 = self.rng.gen();
                        if r < 0.33 { -1 }
                        else if r < 0.66 { 1 }
                        else { 0 }
                    }
                    else { 0 }
                },
                None => { 0 }
            };
        ss.0 = max(0, min(ss.0 + a.0, x_max));
        ss.1 = max(0, min(ss.1 + a.1 + wind, y_max));
        (s, self.info.step_reward, *ss)
    }
}

impl<'a> Graph<'a> {
    fn new(p_ref:&'a mut Policy, w:&World) -> Self {
        Self { q:NdVec2::from_size(w.size), p_ref }
    }

    fn fill_q(&mut self, w:&World) {
        let q = &mut self.q;
        let c = w.size.0 * w.size.1;
        for _ in 0..c {
            let mut a:NdVec2<f64> = NdVec2::new((-1, 1), (-1, 1));
            a.fill(0.0);
            q.push(a);
        }
    }

    fn update(&mut self, s:&State, a:&Action, r:f64, ss:&State, aa:&Action, c_info:&ControlInfo) {
        // println!("{:?} {:?} {} {:?} {:?}", s, a, r, ss, aa);
        let qq = self.q[ss][aa];
        let q = &mut self.q[s][a];
        *q += c_info.alpha * (r + qq - *q);
    }

    fn update_policy(&mut self, s:&State, a_info:&AgentInfo) {
        let (a, _) = self.q[s].iter().enumerate()
            .max_by(|(_, q0), (_, q1)| q0.total_cmp(q1)).unwrap();
        self.p_ref.p[s] = a_info.action.rev_index(a);
    }

    fn print_policy_sample(&self, w:&World, a_info:&AgentInfo) {
        println!();
        let map = &self.p_ref.p;
        let mut visit:HashSet<Vec2> = HashSet::new();
        visit.insert(w.start);
        let mut agent = Agent::new(a_info);
        agent.reset(&w.start);
        let (finish, s) = loop {
            let s = agent.state();
            let act = map[s];
            agent.action(&act, w, None);
            let p = &agent.position;
            if visit.contains(p) {
                println!("position visited (loop) {:?} {:?}", s, p);
                break (false, s)
            }
            visit.insert(*p);
            if w.is_terminal(p) { break (true, s) }
        };
        println!("sample steps {}", visit.len());
        if !finish {
            for (a, q) in self.q[s].iter().enumerate() {
                println!("{:?} {:?}", a, q);
            }
            println!("{:?}", map[s]);
            return
        }
        let size = &w.size;
        for y in (0..size.1).rev() {
            for x in 0..size.0 {
                if visit.contains(&(x as i32, y as i32)) { print!("|+|") }
                else { print!("| |"); }
            }
            println!();
        }
    }
}

impl Policy {
    fn new(w:&World) -> Self {
        Self { p:NdVec2::from_size(w.size) }
    }

    fn fill_random(&mut self, w:&mut World, agent:&mut Agent) {
        let c = w.size.0 * w.size.1;
        let rng = &mut agent.rng;
        let p = &mut self.p;
        for _ in 0..c {
            let a = Policy::random_action(agent.info, rng);
            // println!(" {} {:?}", k, a);
            p.push(a);
        }
    }

    fn random_action(a_info:&AgentInfo, rng:&mut ThreadRng) -> Action {
        let r = if a_info.no_stay { 8.0 } else { 9.0 };//8-dir move
        let rn:f64 = rng.gen();
        let mut r = (rn * r).floor() as i32;
        if a_info.no_stay && r >= 4 { r += 1 }//skip (0, 0)
        a_info.action.rev_index(r as usize)
    }

    fn select_action(&self, s:&State, c_info:&ControlInfo, a_info:&AgentInfo, rng:&mut ThreadRng) -> Action {
        let rn:f64 = rng.gen();
        if rn < c_info.epsilon { Policy::random_action(a_info, rng) }
        else { self.p[*s] }
    }
}

fn episode(c_info:&ControlInfo, agent:&mut Agent, w:&mut World, g:&mut Graph) {
    agent.reset(&w.start);
    let mut a = g.p_ref.select_action(&agent.position, c_info, agent.info, &mut agent.rng);
    loop {
        let (s, r, ss) = agent.action(&a, w, Some(c_info));
        let aa = g.p_ref.select_action(&ss, c_info, agent.info, &mut agent.rng);
        g.update(&s, &a, r, &ss, &aa, c_info);
        g.update_policy(&s, agent.info);
        a = aa;
        if w.is_terminal(&ss) { break }
    }
}

fn iteration(c_info:&ControlInfo, agent:&mut Agent, w:&mut World, g:&mut Graph) {
    let mut ep_c = 0;
    let interval = c_info.max_episode / c_info.episode_check;
    while ep_c < c_info.max_episode {
        let mut ep_cc = 0;
        while ep_cc < interval {
            episode(c_info, agent, w, g);
            ep_cc += 1;
        }
        g.print_policy_sample(w, agent.info);
        ep_c += ep_cc;
    }
}

pub fn run() -> Result<(), Box<dyn Error>> {
    let c_info = ControlInfo {
        max_episode:2000, episode_check:50,
        epsilon:0.1, alpha:0.5,
        stochastic_wind:true,
    };
    let mut w = World {
        size:(10, 7), start:(0, 3), goal:(7, 3),
        wind:vec!(0, 0, 0, 1, 1, 1, 2, 2, 1, 0),
    };
    let a_info = AgentInfo {
        action:NdVec2::new((-1, 1), (-1, 1)),
        step_reward:-1.0, no_stay:false,
    };
    let mut agent = Agent::new(&a_info);
    let mut pi = Policy::new(&w);
    pi.fill_random(&mut w, &mut agent);
    let mut g = Graph::new(&mut pi, &w);
    g.fill_q(&w);
    iteration(&c_info, &mut agent, &mut w, &mut g);
    Ok(())
}
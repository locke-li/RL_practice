use std::cmp::{ min, max };
use std::collections::BTreeMap;
use std::collections::btree_map::Entry::{ Vacant, Occupied };
use std::error::Error;
use rand::prelude::*;

type State = ((i32, i32), (i32, i32));
type Vec2 = (i32, i32);
type StateAction = (State, Vec2);

struct Field{
    boundary:Vec<Vec2>,//index:y, value:(x_min, x_max)
    start_line:i32,//min-y-row
    finish_line:i32,//max-x-column
}

struct AgentInfo {
    pub velocity_max:i32,
    pub action:Vec2,
    pub a_space:(i32, f32),
    pub step_reward:i32,
    pub p_vel_inc0:f64,
    pub epsilon:f64,
}

struct Agent<'a> {
    info:&'a AgentInfo,
    pub velocity:Vec2,
    pub position:Vec2,
}

struct Episode {
    rng:ThreadRng, 
    pub state:Vec<State>,
    pub action:Vec<Vec2>,
    pub reward:i32,
}

struct Graph {
    pub q:BTreeMap<State, BTreeMap<Vec2, (f64, i32)>>,
}

struct Policy {
    state_action:BTreeMap<State, Vec2>,
}

impl Field {
    fn new() -> Self {
        Self { boundary:Vec::new(), start_line:0, finish_line:0 }
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
    }

    fn reset_if_outside(&self, p:&mut Vec2, v:&mut Vec2) {
        let row = match self.boundary.get(p.1 as usize) {
            Some(v) => v,
            None => return self.reset_to_start(p, v),
        };
        if p.0 < row.0 || p.1 > row.1 { self.reset_to_start(p, v) }
    }

    fn reset_to_start(&self, p:&mut Vec2, v:&mut Vec2) {
        *v = (0, 0);
        p.1 = self.start_line;
        let row = &self.boundary[self.start_line as usize];
        p.0 = if p.0 < row.0 { row.0 }
        else if p.0 > row.1 { row.1 }
        else { p.0 };
    }
}

impl AgentInfo {
    fn setup(&mut self) {
        let r = self.action.1 - self.action.0;
        self.a_space = (r, (r * r - 1) as f32);
    }
}

impl<'a> Agent<'a> {
    fn new(velocity:Vec2, position:Vec2, info:&'a AgentInfo) -> Self {
        Self { velocity, position, info }
    }

    fn action(&mut self, v_a:Vec2) {
        let v = self.velocity;
        let v_max = self.info.velocity_max;
        let v0 = min(max(v.0 + v_a.0, 0), v_max);
        let v1 = min(max(v.1 + v_a.1, 0), v_max);
        self.velocity = (v0, v1);
    }

    fn state(&self) -> State {
        (self.position, self.velocity)
    }
}

impl Episode {
    fn new() -> Self {
        Self { state:Vec::new(), action:Vec::new(), reward:0, rng:rand::thread_rng() }
    }

    fn Vec2Add(a:Vec2, b:Vec2) -> Vec2 {
        (a.0 + b.0, a.1 + b.1)
    }

    fn step(b:&mut Policy, f:&Field, a:&mut Agent, rng:&mut ThreadRng) -> StateAction {
        let s = a.state();
        let info = a.info;
        let action = info.action;
        let (act_r, act_s)= info.a_space;
        let v0 = -(s.1).0;
        let v1 = -(s.1).1;
        let act:Vec2;
        let r:f64 = rng.gen();
        if r > info.epsilon {
            //equiprobable explore
            let mut aa = (rng.gen::<f32>() * act_s) as i32;
            let skip = v0 - action.0 + (v1 - action.1) * act_r;
            if aa >= skip {//velocity will become (0, 0)
                aa += 1;
            }
            act = (aa % act_r + action.0, aa / act_r + action.1);
        }
        else {
            //greedy with policy
            act = match b.state_action.get(&s) {
                Some(v) => *v,
                None => if v0 == 0 && v1 == 0 { (1, 1) }
                        else { (0, 0) },
            };
        }
        let v = &mut a.velocity;
        v.0 = min(max(v.0 + act.0, action.0), action.1);
        v.1 = min(max(v.1 + act.1, action.0), action.1);
        let p = &mut a.position;
        p.0 = p.0 + v.0;
        p.1 = p.1 + v.1;
        f.reset_if_outside(p, v);
        (s, act)
    }

    fn generate(&mut self, b:&mut Policy, f:&Field, a:&mut Agent) {
        let mut state:Vec<StateAction> = Vec::new();
    }
}

impl Graph {
    fn new() -> Self {
        Self { q:BTreeMap::new() }
    }
}

impl Policy {
    fn new() -> Self {
        Self { state_action:BTreeMap::new() }
    }
}

fn improve_policy(p:&mut Policy, g:&Graph, s:&State) {
    let a_map = match g.q.get(s) {
        Some(v) => v,
        None => return,
    };
    let (a, _) = a_map.iter().max_by(|(_, (q0, _)), (_, (q1, _))| q0.total_cmp(q1)).unwrap();
    p.state_action.insert(*s, *a);
}

pub fn run() -> Result<(), Box<dyn Error>> {
    let mut a_info = AgentInfo {
        velocity_max:5, action:(-1, 1), step_reward:-1,
        a_space:(0, 0.0),
        p_vel_inc0:0.1, epsilon:0.25,
    };
    a_info.setup();
    let agent = Agent::new((0, 0), (0, 0), &a_info);
    let g_b = Graph::new();
    let g_pi = Graph::new();
    let b = Policy::new();
    let pi = Policy::new();
    Ok(())
}
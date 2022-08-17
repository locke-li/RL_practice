use std::cmp::{ min, max };
use std::collections::BTreeMap;
use std::error::Error;
use rand::prelude::*;

type StateKey = (i32, i32, i32, i32);
type Vec2 = (i32, i32);

struct Field{
    boundary:Vec<Vec2>,//index:y, value:(x_min, x_max)
    start_line:i32,//min-y-row
    finish_line:i32,//max-x-column
}

struct AgentInfo {
    pub velocity_max:i32,
    pub action:Vec2,
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
    pub state:Vec<StateKey>,
    pub action:Vec<Vec2>,
    pub reward:i32,
}

struct Policy {
    state_action:BTreeMap<StateKey, Vec2>,
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

    fn state(&self) -> StateKey {
        let p = self.position;
        let v = self.velocity;
        (p.0, p.1, v.0, v.1)
    }
}

impl Episode {
    fn new() -> Self {
        Self { state:Vec::new(), action:Vec::new(), reward:0, rng:rand::thread_rng() }
    }

    fn step(b:&mut Policy, f:&Field, a:&mut Agent, p_zero:f64, epsilon:f64, rng:&mut ThreadRng) -> (StateKey, Vec2) {
        let s = a.state();
        let r = rng.gen();
        if r < epsilon {
            //explore
        }
        (s, act)
    }

    fn generate(&mut self, b:&mut Policy, f:&Field, a:&mut Agent) {
        let mut state:Vec<StateKey> = Vec::new();
        let p_zero = a.info.p_vel_inc0;
        let epsilon = a.info.epsilon;
    }
}

pub fn run() -> Result<(), Box<dyn Error>> {
    let a_info = AgentInfo {
        velocity_max:5, action:(-1, 1), step_reward:-1,
        p_vel_inc0:0.1, epsilon:0.25,
    };
    let agent = Agent::new((0, 0), (0, 0), &a_info);
    Ok(())
}
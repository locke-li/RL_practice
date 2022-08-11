use std::cmp::{ min, max };

struct Graph {
    pub state:Vec<State>,
}

struct GraphInfo {
    pub theta:f64,
    pub p_win:f64,
    pub state_range:i32,
    pub state_active:(i32, i32),
}

struct State {
    pub capital:i32,
    pub reward:f64,
    pub state_v:f64,
}

struct Policy {
    pub state_action:Vec<i32>,
}

impl State {
    fn new(capital:i32, reward:f64) -> Self {
        Self { capital, reward, state_v:0.0 }
    }
}

impl Graph {
    fn new() -> Self {
        Self { state:Vec::new() }
    }

    fn setup(&mut self, gi:&GraphInfo) {
        let sr = gi.state_range;
        for k in 0..sr {
            self.state.push(State::new(k, 0.0));
        }
        self.state.push(State::new(sr, 1.0));
    }

    fn expected_reward(&self, s:&State, a:i32, gi:&GraphInfo) -> f64 {
        let s_win = min(s.capital + a, gi.state_range) as usize;
        let s_lose = max(s.capital - a, 0) as usize;
        let r_win = self.state[s_win].state_v * gi.p_win;
        let r_lose = self.state[s_lose].state_v * (1.0 - gi.p_win);
        s.reward + r_win + r_lose
    }
}

fn value_iteration(g:&mut Graph, gi:&GraphInfo) {
    let pg:*const Graph = g;
    //hack to grant shared access to graph
    let gs = unsafe { &(*pg) };
    let (s_min, s_max) = gi.state_active;
    loop {
        let mut delta:f64 = 0.0;
        for k in s_min..=s_max {
            let s = &mut g.state[k as usize];
            let v_old = s.state_v;
            let v_new = (1..=s.capital).map(|a| gs.expected_reward(s, a, gi))
                .max_by(|x, y| x.total_cmp(y)).unwrap();
            s.state_v = v_new;
            delta = delta.max((v_new - v_old).abs());
            if delta < gi.theta { break }
        }
    }
}

fn check_policy() {

}

pub fn run() {
    let mut g = Graph::new();
    let g_info = GraphInfo {
        theta: 0.05,
        p_win: 0.4,
        state_range: 100,
        state_active: (1, 99),
    };
    g.setup(&g_info);
    let p = Policy { state_action:Vec::new() };
}
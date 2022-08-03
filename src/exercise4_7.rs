use std::collections::BTreeMap;
use std::collections::btree_map::Entry::{ Vacant, Occupied };
use std::cmp::{ min, max };

use crate::nd_vec::{ NdVec1, NdVec2 };

//for cyclic reference:
//https://eli.thegreenplace.net/2021/rust-data-structures-with-circular-references/

struct Graph {
    pub state: NdVec2<State>,
    pub action: NdVec1<Action>,
}

struct GraphInfo {
    pub l_rent_0:i32,
    pub l_rent_1:i32,
    pub l_return_0:i32,
    pub l_return_1:i32,
    pub move_limit:i32,
    pub state_range:i32,
    pub nf:Vec<i64>,
}

struct AgentInfo {
    pub discount:f32,
    pub theta:f32,
    pub max_iter:i32,
}

struct Policy {
    pub state_action: NdVec2<i32>,//state index - action index
}

struct StateDesc {
    pub name: String,
    pub count: (i32, i32),
}

struct State {
    pub desc: StateDesc,
    pub reward: f32,
    pub action: Vec<(i32, Vec<i32>)>,
    pub transition: Vec<Transition>,
    pub state_v: f32,
}

struct ActionDesc {
    pub name: String,
    pub count: i32,
}

struct Action {
    pub desc: ActionDesc,
    pub reward: f32,
}

struct Transition {
    pub action: i32,
    pub from: (i32, i32),
    pub to: (i32, i32),
    pub prob: f32,
}

impl StateDesc {
    fn new(name:String, count:(i32, i32)) -> Self {
        Self { name, count }
    }
}

impl State {
    fn new(desc:StateDesc, reward:f32) -> Self {
        Self { desc, reward, action: Vec::new(), transition: Vec::new(), state_v: 0.0}
    }

    fn name(&self) -> &str {
        &self.desc.name
    }

    fn count(&self) -> (i32, i32) {
        self.desc.count
    }
}

impl PartialEq for State {
    fn eq(&self, other: &Self) -> bool {
        self.name() == other.name()
    }
}

impl ActionDesc {
    fn new(name:String, count:i32) -> Self {
        Self { name, count }
    }
}

impl Action {
    fn new(desc:ActionDesc, reward:f32) -> Self {
        Self { desc, reward }
    }

    fn name(&self) -> &str {
        &self.desc.name
    }

    fn count(&self) -> i32 {
        self.desc.count
    }
}

impl Transition {
    fn reward(&self, g:&Graph, discount:f32) -> f32 {
        g.state[self.from].reward + g.action[self.action].reward + discount * g.state[self.to].state_v
    }
}

impl GraphInfo {
    fn setup(&mut self) {
        let nf = &mut self.nf;
        nf.push(1);
        for k in 1..=self.state_range as i64 {
            nf.push(nf.last().unwrap() * k);
        }
    }
}

impl Graph {
    fn new(gi:&GraphInfo) -> Self {
        let s = gi.state_range;
        let a = gi.move_limit;
        Self {
            state: NdVec2::new((0, s), (0, s)),
            action: NdVec1::new((-a, a)),
        }
    }

    fn add_state(&mut self, desc:StateDesc, reward:f32) {
        let state = State::new(desc, reward);
        self.state.push(state);
    }

    fn add_action(&mut self, desc:ActionDesc, reward:f32) {
        let action = Action::new(desc, reward);
        self.action.push(action);
    }

    fn add_transition(&mut self, action:i32, from:(i32, i32), to:(i32, i32), prob:f32) {
        let s_from = &mut self.state[from];
        s_from.transition.push(Transition { action, from, to, prob });
    }

    fn state_name(m:i32, n:i32) -> String {
        format!("{}_{}", m, n)
    }

    fn action_name(v:i32) -> String {
        format!("{:+}", v)
    }

    fn state_reward(v:i32, l:i32) -> f32 {
        //Poisson distribution with mean l
        min(v, l) as f32 * 10.0
    }

    fn add_transition_for_move(s:&mut State, k:i32, gi:&GraphInfo, prob:f32) {
        let (c0, c1) = s.count();
        let e:f32 = 2.7182818284;
        let l0 = gi.l_return_0 as f32;
        let l1 = gi.l_return_1 as f32;
        let sr = gi.state_range;
        for n0 in 0..=sr {
            for n1 in  0..=sr {
                let to = (min(sr, c0 - k + n0), min(sr, c1 + k + n1));
                if to.0 < 0 || to.1 < 0 { continue }
                let prob0 = e.powf(-l0) * l0.powf(n0 as f32) / gi.nf[n0 as usize] as f32;
                let prob1 = e.powf(-l1) * l1.powf(n1 as f32) / gi.nf[n1 as usize] as f32;
                s.transition.push(Transition { action:k, from:s.count(), to, prob: prob * prob0 * prob1 });
            }
        }
    }

    fn parse_action(s:&mut State) {
        let mut map:BTreeMap<i32, Vec<i32>> = BTreeMap::new();
        let mut i = 0;
        for t in s.transition.iter() {
            let list = match map.entry(t.action) {
                Vacant(v) => v.insert(Vec::new()),
                Occupied(v) => v.into_mut(),
            };
            list.push(i);
            i += 1;
        }
        s.action = map.into_iter().collect();
    }

    fn setup(&mut self, gi:&GraphInfo) {
        for m in 0..=gi.state_range {
            for n in 0..=gi.state_range {
                let desc = StateDesc::new(Graph::state_name(m, n), (m, n));
                self.add_state(desc, Graph::state_reward(m, gi.l_rent_0) + Graph::state_reward(n, gi.l_rent_1));
            }
        }
        let m = gi.move_limit;
        for k in -m..=m {
            let desc = ActionDesc::new(Graph::action_name(k), k);
            self.add_action(desc, k.abs() as f32 * -2.0);
        }
        let m = gi.move_limit;
        for s in self.state.iter_mut() {
            let prob = 1.0 / (m * 2 + 1) as f32;
            // println!("{} {} {} {}", c0, c1, range0, range1);
            //self transition
            Graph::add_transition_for_move(s, 0, gi, prob);
            //move out
            for k in 1..=m {
                Graph::add_transition_for_move(s, k, gi, prob);
            }
            //move in
            for k in 1..=m {
                Graph::add_transition_for_move(s, -k, gi, prob);
            }
            Graph::parse_action(s);
        }
    }

    fn print_info(&self) {
        println!("action:");
        for a in self.action.iter() {
            println!("\t{}:{}", a.name(), a.reward);
        }
        println!("state:");
        for s in self.state.iter() {
            println!("\t{}:{}", s.name(), s.reward);
            for t in s.transition.iter() {
                println!("\t\t{}:{:?}->{:?}|{}", t.action, t.from, t.to, t.prob);
            }
        }
    }

    fn print_state(&self) {
        //TODO
        let limit = 20;
        let mut count = 0;
        for s in self.state.iter() {
            print!("\t{:.1}", s.state_v);
            count += 1;
            if count > limit {
                count = 0;
                println!();
            }
        }
        println!();
    }

    fn print_reward(&self) {
        //TODO
        let limit = 20;
        let mut count = 0;
        for s in self.state.iter() {
            print!("\t{}", s.reward);
            count += 1;
            if count > limit {
                count = 0;
                println!();
            }
        }
        println!();
    }

    fn print_policy(&self, p:&Policy) {
        //TODO
        let limit = 20;
        let mut count = 0;
        for s in self.state.iter() {
            let sn = s.count();
            let a = p.state_action[sn];
            // print!("{:?}|{} ", sn, a);
            print!("{} ", a);
            count += 1;
            if count > limit {
                count = 0;
                println!();
            }
        }
        println!();
    }
}

impl Policy {
    fn new(gi:&GraphInfo) -> Self {
        let s = gi.state_range;
        let mut v =  NdVec2::new((0, s), (0, s));
        let s = s + 1;
        v.resize((s * s) as usize, 0);
        Self { state_action: v}
    }
}

fn evaluate_policy(g:&mut Graph, info:&AgentInfo) {
    let mut i = 0;
    let pg:*const Graph = g;
    //hack to grant shared graph access
    let gs = unsafe { &(*pg) };
    loop {
        let mut delta:f32 = 0.0;
        for s in g.state.iter_mut() {
            let v_old = s.state_v;
            let v_new = s.transition.iter()
                .map(|t| t.prob * t.reward(gs, info.discount))
                .sum::<f32>();
            s.state_v = v_new;
            // println!("{} {} {}", s.name(), v_old, v_new);
            delta = delta.max((v_new - v_old).abs());
        }
        i += 1;
        // println!("{}:{}", i, delta);
        if delta <= info.theta || i >= info.max_iter { break }
    }
}

fn policy_improvement(p:&mut Policy, g:&Graph, info:&AgentInfo) -> bool {
    println!("improvement:");
    let mut policy_stable = true;
    for s in g.state.iter() {
        let sn = s.count();
        let a_old = p.state_action[sn];
        let (a_new, _) = s.action.iter()
            .map(|(a, vec_t)| (*a, vec_t))
            .map(|(a, vec_t)|
                (a, vec_t.iter()
                    .map(|t| &s.transition[*t as usize])
                    .map(|t| t.prob * t.reward(g, info.discount))
                    .sum::<f32>()))
            .max_by(|(_, x), (_, y)| x.total_cmp(y)).unwrap();
        // s.action.iter()
        //     .map(|(_, vec_t)| (vec_t[0].action, vec_t))
        //     .map(|(a, vec_t)|
        //         (a, vec_t.iter().map(|t| t.prob * t.reward(discount)).sum::<f32>() / vec_t.len() as f32))
        //     .for_each(|(a, v)| println!("{} {}", a, v));
        let state_stable = a_old == a_new;
        // println!("{} {}", sn, a_new);
        p.state_action[sn] = a_new;
        policy_stable = policy_stable && state_stable;
    }
    policy_stable
}

pub fn run() {
    let agent_info = AgentInfo { discount:0.9, theta:0.1, max_iter:128 };
    let mut graph_info = GraphInfo { move_limit:5, l_rent_0:3, l_rent_1:4, l_return_0:3, l_return_1:2, state_range:20, nf:Vec::new() };
    graph_info.setup();
    let mut g = Graph::new(&graph_info);
    g.setup(&graph_info);
    // g.print_reward();
    // g.print_info();
    let mut p = Policy::new(&graph_info);
    loop {
        evaluate_policy(&mut g, &agent_info);
        // g.print_state();
        let stable = policy_improvement(&mut p, &g, &agent_info);
        g.print_state();
        g.print_policy(&p);
        if stable { break }
    }
    println!("finish");
}
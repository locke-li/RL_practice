

use std::collections::BTreeMap;
use std::collections::btree_map::Entry::{ Vacant, Occupied };
use std::cmp::{ min, max };

use crate::nd_vec::{ NdVec1, NdVec2 };
use crate::poisson::Poisson;

//for cyclic reference:
//https://eli.thegreenplace.net/2021/rust-data-structures-with-circular-references/

struct Graph {
    pub state: NdVec2<State>,
    pub action: NdVec1<Action>,
}

struct GraphInfo {
    pub dist_rent_0:Poisson,
    pub dist_rent_1:Poisson,
    pub dist_return_0:Poisson,
    pub dist_return_1:Poisson,
    pub move_limit:i32,
    pub state_range:i32,
    pub rent_reward:i32,
}

struct GraphChange {
    pub free_shuttle:i32,
    pub parking_limit:i32,
    pub parking_cost:i32,
}

struct AgentInfo {
    pub discount:f64,
    pub theta:f64,
    pub max_iter:i32,
}

struct Policy {
    pub state_action: NdVec2<i32>,//state index - action index
}

struct StateDesc {
    pub name: String,
    pub count: (i32, i32),
    pub rent: (f64, f64),
}

struct State {
    pub desc: StateDesc,
    pub reward: f64,
    pub action: Vec<(i32, Vec<i32>)>,
    pub transition: Vec<Transition>,
    pub state_v: f64,
}

struct ActionDesc {
    pub name: String,
    pub count: i32,
}

struct Action {
    pub desc: ActionDesc,
    pub reward: f64,
}

struct Transition {
    pub action: i32,
    pub from: (i32, i32),
    pub to: (i32, i32),
    pub prob: f64,
}

impl StateDesc {
    fn new(name:String, count:(i32, i32), rent:(f64, f64)) -> Self {
        Self { name, count, rent }
    }
}

impl State {
    fn new(desc:StateDesc, reward:f64) -> Self {
        Self { desc, reward, action: Vec::new(), transition: Vec::new(), state_v: 0.0}
    }

    fn name(&self) -> &str {
        &self.desc.name
    }

    fn count(&self) -> (i32, i32) {
        self.desc.count
    }

    fn expected_count(&self) -> (f64, f64) {
        let c = self.desc.count;
        let r = self.desc.rent;
        (c.0 as f64 - r.0, c.1 as f64 - r.1)
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
    fn new(desc:ActionDesc, reward:f64) -> Self {
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
    fn reward(&self, g:&Graph, discount:f64) -> f64 {
        g.state[self.from].reward + g.action[self.action].reward + discount * g.state[self.to].state_v
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

    fn add_state(&mut self, desc:StateDesc, reward:f64) {
        let state = State::new(desc, reward);
        self.state.push(state);
    }

    fn add_action(&mut self, desc:ActionDesc, reward:f64) {
        let action = Action::new(desc, reward);
        self.action.push(action);
    }

    fn add_transition(&mut self, action:i32, from:(i32, i32), to:(i32, i32), prob:f64) {
        let s_from = &mut self.state[from];
        s_from.transition.push(Transition { action, from, to, prob });
    }

    fn state_name(m:i32, n:i32) -> String {
        format!("{}_{}", m, n)
    }

    fn action_name(v:i32) -> String {
        format!("{:+}", v)
    }

    fn expected_count(v:i32, dist:&Poisson) -> f64 {
        let v = v as usize;
        let mut r:f64 = 0.0;
        r += (0..=v).map(|n| dist.pmf(n) * n as f64).sum::<f64>();
        r += (1.0 - dist.cdf(v)) * v as f64;
        r
    }

    fn add_transition_for_move(s:&mut State, k:i32, gi:&GraphInfo, prob_a:f64) {
        let (c0, c1) = s.expected_count();
        let dist0 = &gi.dist_return_0;
        let dist1 = &gi.dist_return_1;
        let sr = gi.state_range;
        // for n0 in 0..=sr {
        //     for n1 in  0..=sr {
        //         let to = (
        //             max(min(sr, c0 as i32 - k + n0), 0),
        //             max(min(sr, c1 as i32 + k + n1), 0),
        //         );
        //         let prob = prob_a * dist0.pmf(n0 as usize) * dist1.pmf(n1 as usize);
        //         s.transition.push(Transition { action:k, from:s.count(), to, prob });
        //     }
        // }
        let c0 = c0 as f64;
        let c1 = c1 as f64;
        let kf = k as f64;
        let return0 = Graph::expected_count(sr, dist0);
        let return1 = Graph::expected_count(sr, dist1);
        let to = (
            max(min(sr, (c0 - kf + return0).round() as i32), 0), 
            max(min(sr, (c1 + kf + return1).round() as i32), 0)
        );
        s.transition.push(Transition { action:k, from:s.count(), to, prob:prob_a });
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

    fn setup(&mut self, gi:&GraphInfo, c:Option<&GraphChange>) {
            for n in 0..=gi.state_range {
            for m in 0..=gi.state_range {
                let rent0 = Graph::expected_count(m, &gi.dist_rent_0);
                let rent1 = Graph::expected_count(n, &gi.dist_rent_1);
                let desc = StateDesc::new(Graph::state_name(m, n), (m, n), (rent0, rent1));
                let state_reward = (rent0 + rent1) * gi.rent_reward as f64
                    + match c {
                        Some(v) => {
                            //possible parking costs
                            (if m > v.parking_limit { -v.parking_cost } else { 0 }) +
                            if n > v.parking_limit { -v.parking_cost } else { 0 }
                        }
                        None => 0,
                    } as f64;
                self.add_state(desc, state_reward);
            }
        }
        let m = gi.move_limit;
        for k in -m..=m {
            let desc = ActionDesc::new(Graph::action_name(k), k);
            let action_reward = (k.abs() - match c {
                Some(v) => v.free_shuttle,
                None => 0,
            }).max(0) as f64 * -2.0;
            self.add_action(desc, action_reward);
        }
        let m = gi.move_limit;
        for s in self.state.iter_mut() {
            let prob = 1.0 / (m * 2 + 1) as f64;
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

    fn print_state(&self, gi:&GraphInfo) {
        let limit = gi.state_range;
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

    fn print_reward(&self, gi:&GraphInfo) {
        let limit = gi.state_range;
        let mut count = 0;
        for s in self.state.iter() {
            print!("\t{:.1}", s.reward);
            count += 1;
            if count > limit {
                count = 0;
                println!();
            }
        }
        println!();
    }

    fn print_policy(&self, p:&Policy, gi:&GraphInfo) {
        let limit = gi.state_range;
        let mut count = 0;
        for s in self.state.iter() {
            let sn = s.count();
            let a = p.state_action[sn];
            // print!("{:?} {}|{:+} ", sn, self.state.index(sn), a);
            print!("{:+} ", a);
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
    //hack to grant shared access to graph
    let gs = unsafe { &(*pg) };
    loop {
        let mut delta:f64 = 0.0;
        for s in g.state.iter_mut() {
            let v_old = s.state_v;
            let v_new = s.transition.iter()
                .map(|t| t.prob * t.reward(gs, info.discount))
                .sum::<f64>();
            s.state_v = v_new;
            // println!("{} {} {}", s.name(), v_old, v_new);
            delta = delta.max((v_new - v_old).abs());
        }
        i += 1;
        // println!("{}:{}", i, delta);
        if delta <= info.theta || i >= info.max_iter { break }
    }
}

fn policy_improvement(p:&mut Policy, g:&Graph, info:&AgentInfo, gi:&GraphInfo) -> bool {
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
                    .sum::<f64>()))
            .max_by(|(_, x), (_, y)| x.total_cmp(y)).unwrap();
        // s.action.iter()
        //     .map(|(_, vec_t)| (vec_t[0].action, vec_t))
        //     .map(|(a, vec_t)|
        //         (a, vec_t.iter().map(|t| t.prob * t.reward(discount)).sum::<f32>() / vec_t.len() as f32))
        //     .for_each(|(a, v)| println!("{} {}", a, v));
        let state_stable = a_old == a_new;
        // if !state_stable { println!("{:?} {:+} {:+}", sn, a_old, a_new); }
        // println!("{} {}", sn, a_new);
        p.state_action[sn] = a_new;
        policy_stable = policy_stable && state_stable;
        // g.print_policy(p, gi);
    }
    policy_stable
}

pub fn run() {
    let agent_info = AgentInfo { discount:0.9, theta:0.1, max_iter:1 };
    let state_range:usize = 20;
    let graph_info = GraphInfo { 
        move_limit:5, state_range:state_range as i32,
        rent_reward:10,
        dist_rent_0:Poisson::new(3, state_range),
        dist_rent_1:Poisson::new(4, state_range),
        dist_return_0:Poisson::new(3, state_range),
        dist_return_1:Poisson::new(2, state_range),
    };
    let graph_change = GraphChange {
        free_shuttle:1,
        parking_limit:10,
        parking_cost:4,
    };
    //changes switch
    let option_change = 
        // Some(&graph_change);
        None;
    let mut g = Graph::new(&graph_info);
    g.setup(&graph_info, option_change);
    g.print_reward(&graph_info);
    // g.print_info();
    let mut p = Policy::new(&graph_info);
    loop {
        evaluate_policy(&mut g, &agent_info);
        // g.print_state();
        let stable = policy_improvement(&mut p, &g, &agent_info, &graph_info);
        g.print_state(&graph_info);
        g.print_policy(&p, &graph_info);
        if stable { break }
    }
    println!("finish");
}
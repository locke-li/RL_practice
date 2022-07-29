use std::collections::BTreeMap;
use std::collections::btree_map::Entry::{ Vacant, Occupied };
use std::cmp::{ min, max };

//for cyclic reference:
//https://eli.thegreenplace.net/2021/rust-data-structures-with-circular-references/

struct Graph<'a> {
    pub state: Vec<State<'a>>,
    pub action: Vec<Action>,
    pub state_lookup: BTreeMap<&'a str, *mut State<'a>>,
    pub action_lookup: BTreeMap<&'a str, *const Action>,
}

struct GraphInfo {
    pub l_rent_0:i32,
    pub l_rent_1:i32,
    pub l_return_0:i32,
    pub l_return_1:i32,
    pub move_limit:i32,
    pub state_range:i32,
    pub nf:Vec<i32>,
}

struct AgentInfo {
    pub discount:f32,
    pub theta:f32,
    pub max_iter:i32,
}

struct Policy {
    pub state_action: BTreeMap<String, String>,
}

struct StateDesc {
    pub name: String,
    pub count: Vec<i32>,
}

struct State<'a> {
    pub desc: StateDesc,
    pub reward: f32,
    pub action: Vec<(&'a str, Vec<&'a Transition<'a>>)>,
    pub transition: Vec<Transition<'a>>,
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

struct Transition<'a> {
    pub action: &'a Action,
    pub from: &'a State<'a>,
    pub to: &'a State<'a>,
    pub prob: f32,
}

impl StateDesc {
    fn new(name:String, count:Vec<i32>) -> Self {
        Self { name, count }
    }
}

impl<'a> State<'a> {
    fn new(desc:StateDesc, reward:f32) -> Self {
        Self { desc, reward, action: Vec::new(), transition: Vec::new(), state_v: 0.0}
    }

    fn name(&self) -> &str {
        &self.desc.name
    }
}

impl<'a> PartialEq for State<'a> {
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

impl<'a> Transition<'a> {
    fn reward(&self, discount:f32) -> f32 {
        self.from.reward + self.action.reward + discount * self.to.state_v
    }
}

impl GraphInfo {
    fn setup(&mut self) {
        let nf = &mut self.nf;
        nf.push(1);
        for k in 1..10 {
            nf.push(nf.last().unwrap() * k);
        }
    }
}

impl<'a> Graph<'a> {
    fn new() -> Self {
        Self {
            state: Vec::new(), state_lookup: BTreeMap::new(),
            action: Vec::new(), action_lookup: BTreeMap::new(),
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

    fn add_transition(&self, action:&str, from:&str, to:&str, prob:f32){
        if !self.action_lookup.contains_key(action)
            || !self.state_lookup.contains_key(from)
            || !self.state_lookup.contains_key(to) {
            println!("invalid transition {:?}:{:?}->{:?}", action, from, to);
            return;
        }
        unsafe {
            let action = &(*self.action_lookup[action]);
            let p_from = self.state_lookup[from];
            let from = &(*p_from);
            let from_mut = &mut(*p_from);
            let to = &(*self.state_lookup[to]);
            from_mut.transition.push(Transition::<'a> { action, from, to, prob });
        }
    }

    fn refresh_lookup(&mut self) {
        self.action_lookup = self.action.iter()
            .map(|a| {
                let p:*const Action = a;
                unsafe { ((*p).name(), p) }
            }).collect();
        self.state_lookup = self.state.iter_mut()
            .map(|s| {
                let p:*mut State = s;
                unsafe { ((*p).name(), p) }
            }).collect();
    }

    fn refresh_state_action(&self) {
        for s in self.state.iter() {
            let mut map:BTreeMap<&str, Vec<&Transition>> = BTreeMap::new();
            for t in s.transition.iter() {
                let pt:*const Transition = t;
                let t_ref = unsafe { &(*pt) };
                let list = match map.entry(t_ref.action.name()) {
                    Vacant(v) => v.insert(Vec::new()),
                    Occupied(v) => v.into_mut(),
                };
                list.push(t_ref);
            }
            let ps = self.state_lookup[s.name()];
            let s_ref = unsafe { &mut(*ps) };
            s_ref.action = map.into_iter().collect();
        }
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

    fn state_change(m:i32, v:i32, l:i32) -> f32 {
        //Poisson distribution with mean l
        min(m - v, l) as f32
    }

    fn add_transition_for_move(&self, s:&str, c0:i32, c1:i32, k:i32, gi:&GraphInfo, prob:f32) {
        let action = &Graph::action_name(k);
        let e:f32 = 2.7182818284;
        let r0 = min(gi.move_limit - c0, gi.l_return_0 * 2);
        let r1 = min(gi.move_limit - c1, gi.l_return_1 * 2);
        let l0 = gi.l_return_0 as f32;
        let l1 = gi.l_return_1 as f32;
        for n0 in 0..=r0 {
            for n1 in  0..=r1 {
                let to = &Graph::state_name(c0 - k + n0, c1 + k + n1);
                let prob0 = e.powf(-l0) * l0.powf(n0 as f32) / gi.nf[n0 as usize] as f32;
                let prob1 = e.powf(-l1) * l1.powf(n1 as f32) / gi.nf[n1 as usize] as f32;
                self.add_transition(action, s, to, prob0 * prob1 * prob);
            }
        }
    }

    fn setup(&mut self, gi:&GraphInfo) {
        for m in 0..=gi.state_range {
            for n in 0..=gi.state_range {
                let mut count:Vec<i32> = Vec::new();
                count.push(m);
                count.push(n);
                let desc = StateDesc::new(Graph::state_name(m, n), count);
                self.add_state(desc, Graph::state_reward(m, gi.l_rent_0) + Graph::state_reward(n, gi.l_rent_1));
            }
        }
        for k in 0..=gi.move_limit {
            let desc = ActionDesc::new(Graph::action_name(k), k);
            self.add_action(desc, k as f32 * -2.0);
            let desc = ActionDesc::new(Graph::action_name(-k), -k);
            self.add_action(desc, k as f32 * -2.0);
        }
        self.refresh_lookup();
        let a0 = self.action.get(0).unwrap();
        for s in self.state.iter() {
            let count = &s.desc.count;
            let c0 = count[0];
            let c1 = count[1];
            let range0 = min(c0, gi.move_limit);
            let range1 = min(c1, gi.move_limit);
            let prob = 1.0 / (range0 + range1 + 1) as f32;
            // println!("{} {} {} {}", c0, c1, range0, range1);
            //self transition
            self.add_transition(a0.name(), s.name(), s.name(), prob);
            //move out
            for k in 1..=range0 {
                self.add_transition_for_move(s.name(), c0, c1, k, gi, prob);
            }
            //move in
            for k in 1..=range1 {
                self.add_transition_for_move(s.name(), c0, c1, -k, gi, prob);
            }
        }
        self.refresh_state_action();
    }

    fn print_info(&self) {
        println!("action:");
        for a in self.action.iter() {
            println!("\t{:?}:{:?}", a.name(), a.reward);
        }
        println!("state:");
        for s in self.state.iter() {
            println!("\t{:?}:{:?}", s.name(), s.reward);
            for t in s.transition.iter() {
                println!("\t\t{:?}:{:?}->{:?}|{:?}", t.action.name(), t.from.name(), t.to.name(), t.prob);
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
            let sn = s.name();
            let a = &p.state_action[sn];
            // print!("{}|{} ", sn, a);
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
    fn new() -> Self {
        Self { state_action: BTreeMap::new() }
    }
}

fn evaluate_policy(state:&mut Vec<State>, info:&AgentInfo) {
    let mut i = 0;
    loop {
        let mut delta:f32 = 0.0;
        for s in state.iter_mut() {
            let v_old = s.state_v;
            let v_new = s.transition.iter()
                .map(|t| t.prob * t.reward(info.discount))
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
        let sn = s.name();
        let a_old = p.state_action.get(sn);
        let (a_new, _) = s.action.iter()
            .map(|(_, vec_t)| (vec_t[0].action, vec_t))
            .map(|(a, vec_t)|
                (a, vec_t.iter().map(|t| t.prob * t.reward(info.discount)).sum::<f32>()))
            .max_by(|(_, x), (_, y)| x.total_cmp(y)).unwrap();
        // s.action.iter()
        //     .map(|(_, vec_t)| (vec_t[0].action, vec_t))
        //     .map(|(a, vec_t)|
        //         (a, vec_t.iter().map(|t| t.prob * t.reward(discount)).sum::<f32>() / vec_t.len() as f32))
        //     .for_each(|(a, v)| println!("{} {}", a.name(), v));
        let state_stable = match a_old {
            Some(v) => v.eq(a_new.name()),
            None => false,
        };
        // println!("{} {}", sn, a_new.name());
        p.state_action.insert(sn.to_owned().clone(), a_new.name().to_owned().clone());
        policy_stable = policy_stable && state_stable;
    }
    policy_stable
}

pub fn run() {
    let agent_info = AgentInfo { discount:0.9, theta:0.1, max_iter:128 };
    let mut graph_info = GraphInfo { move_limit:5, l_rent_0:3, l_rent_1:4, l_return_0:3, l_return_1:2, state_range:20, nf:Vec::new() };
    graph_info.setup();
    let mut g = Graph::new();
    g.setup(&graph_info);
    // g.print_reward();
    g.print_info();
    let mut p = Policy::new();
    loop {
        evaluate_policy(&mut g.state, &agent_info);
        // g.print_state();
        let stable = policy_improvement(&mut p, &g, &agent_info);
        g.print_state();
        g.print_policy(&p);
        if stable { break }
    }
    println!("finish");
}
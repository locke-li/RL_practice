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
        self.action.reward as f32 + discount * self.to.state_v
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
        format!("s{}_{}", m, n)
    }

    fn action_name(v:i32) -> String {
        format!("a{}", v)
    }

    fn state_reward(v:i32, l:i32) -> f32 {
        //Poisson distribution with mean l
        min(v, l) as f32 * 10.0
    }

    fn state_change(m:i32, v:i32, l:i32) -> f32 {
        //Poisson distribution with mean l
        min(m - v, l) as f32
    }

    fn setup(&mut self) {
        let l_rent_0 = 3;
        let l_rent_1 = 4;
        let l_return_0 = 3;
        let l_return_1 = 2;
        let move_limit = 5;
        let state_range = 20;
        let action_range = move_limit;
        for m in 0..=state_range {
            for n in 0..=state_range {
                let mut count:Vec<i32> = Vec::new();
                count.push(m);
                count.push(n);
                let desc = StateDesc::new(Graph::state_name(m, n), count);
                self.add_state(desc, Graph::state_reward(m, l_rent_0) + Graph::state_reward(n, l_rent_1));
            }
        }
        for k in 0..=action_range {
            let desc = ActionDesc::new(Graph::action_name(k), k);
            self.add_action(desc, k as f32 * -2.0);
        }
        self.refresh_lookup();
        let a0 = self.action.get(0).unwrap();
        for s in self.state.iter() {
            let count = &s.desc.count;
            let c0 = max(count[0] + l_return_0 - l_rent_0, 0);
            let c1 = max(count[1] + l_return_1 - l_rent_1, 0);
            let range0 = min(min(c0, state_range - c1), move_limit);
            let range1 = min(min(c1, state_range - c0), move_limit);
            let prob = 1.0 / (range0 + range1 + 1) as f32;
            // println!("{} {} {} {}", c0, c1, range0, range1);
            //self transition
            self.add_transition(a0.name(), s.name(), s.name(), prob);
            //move out
            for k in 1..=range0 {
                let action = &Graph::action_name(k);
                let to = &Graph::state_name(c0 - k, c1 + k);
                self.add_transition(action, s.name(), to, prob);
            }
            //move in
            for k in 1..=range1 {
                let action = &Graph::action_name(k);
                let to = &Graph::state_name(c0 + k, c1 - k);
                self.add_transition(action, s.name(), to, prob);
            }
            self.refresh_state_action();
        }
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
}

impl Policy {
    fn new() -> Self {
        Self { state_action: BTreeMap::new() }
    }

    fn print(&self) {
        //TODO
        let limit = 20;
        let mut count = 0;
        for (s, a) in self.state_action.iter() {
            print!("{} ", a);
            count += 1;
            if count > limit {
                count = 0;
                println!();
            }
        }
    }
}

fn evaluate_policy(state:&mut Vec<State>, discount:f32, theta: f32, max_iter:i32) {
    let mut i = 0;
    loop {
        let mut delta:f32 = 0.0;
        for s in state.iter_mut() {
            let v_old = s.state_v;
            let v_new = s.transition.iter()
                .map(|t| t.prob * t.reward(discount))
                .sum::<f32>() + s.reward;
            s.state_v = v_new;
            delta = delta.max((v_new - v_old).abs());
        }
        i += 1;
        // println!("{}:{}", i, delta);
        if delta <= theta || i >= max_iter { break }
    }
}

fn policy_improvement(p:&mut Policy, g:&Graph, discount:f32) -> bool {
    println!("improvement:");
    let mut policy_stable = true;
    for s in g.state.iter() {
        let sn = s.name();
        let a_old = p.state_action.get(sn);
        let (a_new, _) = s.action.iter()
            .map(|(_, vec_t)| (vec_t[0].action, vec_t))
            .map(|(a, vec_t)|
                (a, a.reward + vec_t.iter().map(|t| t.prob * t.reward(discount)).sum::<f32>()))
            .max_by(|(_, x), (_, y)| x.total_cmp(y)).unwrap();
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
    let discount = 0.9;
    let mut g = Graph::new();
    g.setup();
    // g.print_info();
    let mut p = Policy::new();
    loop {
        evaluate_policy(&mut g.state, discount, 0.1, 128);
        // g.print_state();
        let stable = policy_improvement(&mut p, &g, discount);
        p.print();
        if stable { break }
    }
    println!("finish");
}
use std::collections::BTreeMap;

//for cyclic reference:
//https://eli.thegreenplace.net/2021/rust-data-structures-with-circular-references/

struct Graph<'a> {
    pub state: Vec<State<'a>>,
    pub action: Vec<Action>,
    pub state_lookup: BTreeMap<&'a str, *mut State<'a>>,
    pub action_lookup: BTreeMap<&'a str, *const Action>,
    pub theta: f32,
    pub policy_v: f32,
}

struct StateDesc {
    pub name: String,
    pub count: Vec<i32>,
}

struct State<'a> {
    pub desc: StateDesc,
    pub reward: i32,
    pub action: Vec<Transition<'a>>,
    pub state_v: f32,
}

struct ActionDesc {
    pub name: String,
    pub count: i32,
}

struct Action {
    pub desc: ActionDesc,
    pub reward: i32,
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
    fn new(desc:StateDesc, reward:i32) -> Self {
        Self { desc, reward, action: Vec::new(), state_v: 0.0}
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
    fn new(desc:ActionDesc, reward:i32) -> Self {
        Self { desc, reward }
    }

    fn name(&self) -> &str {
        &self.desc.name
    }

    fn count(&self) -> i32 {
        self.desc.count
    }
}
impl<'a> Graph<'a> {
    fn new() -> Self {
        Self {
            state: Vec::new(), state_lookup: BTreeMap::new(),
            action: Vec::new(), action_lookup: BTreeMap::new(),
            theta: 0.0, policy_v: 0.0,
        }
    }

    fn add_state(&mut self, desc:StateDesc, reward:i32) -> &State {
        let state = State::new(desc, reward);
        self.state.push(state);
        let state:*mut State = self.state.last_mut().unwrap();
        unsafe {
            self.state_lookup.insert((*state).name(), state);
            &(*state)
        }
    }

    fn add_action(&mut self, desc:ActionDesc, reward:i32) -> &Action {
        let action = Action::new(desc, reward);
        self.action.push(action);
        let action:*const Action = self.action.last().unwrap();
        unsafe {
            self.action_lookup.insert((*action).name(), action);
            &(*action)
        }
    }

    fn add_transition(&self, action:&str, from:&str, to:&str, prob:f32) {
        if !self.action_lookup.contains_key(action)
            || !self.state_lookup.contains_key(from)
            || !self.state_lookup.contains_key(to) {
            println!("invalid transition {:?}:{:?}->{:?}", action, from, to);
            return
        }
        unsafe {
            let action = &(*self.action_lookup[action]);
            let from = self.state_lookup[from];
            let to = &(*self.state_lookup[to]);
            (*from).action.push(Transition::<'a> { action, from: &(*from), to, prob });
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

    fn state_name(m:i32, n:i32) -> String {
        format!("s{}_{}", m, n)
    }

    fn action_name(v:i32) -> String {
        format!("a{}", v)
    }

    fn setup(&mut self) {
        let move_limit = 5;
        let state_range = 20;
        let action_range = move_limit;
        for m in 0..=state_range {
            for n in 0..=state_range {
                let mut count:Vec<i32> = Vec::new();
                count.push(m);
                count.push(n);
                let desc = StateDesc::new(Graph::state_name(m, n), count);
                self.add_state(desc, 0);
            }
        }
        for k in 0..=action_range {
            let desc = ActionDesc::new(Graph::action_name(k), k);
            self.add_action(desc, k * -2);
        }
        self.refresh_lookup();
        let a0 = self.action.get(0).unwrap();
        for s in self.state.iter() {
            let count = &s.desc.count;
            let range0 = min(min(count[0], state_range - count[1]), move_limit);
            let range1 = min(min(count[1], state_range - count[0]), move_limit);
            let prob = 1.0 / (range0 + range1 + 1) as f32;
            // println!("{} {} {} {}", count[0], count[1], range0, range1);
            //self transition
            self.add_transition(a0.name(), s.name(), s.name(), prob);
            //move out
            for k in 1..=range0 {
                let action = &Graph::action_name(k);
                let to = &Graph::state_name(count[0] - k, count[1] + k);
                self.add_transition(action, s.name(), to, prob)
                }
            //move in
            for k in 1..=range1 {
                let action = &Graph::action_name(k);
                let to = &Graph::state_name(count[0] + k, count[1] - k);
                self.add_transition(action, s.name(), to, prob)
            }
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
            for t in s.action.iter() {
                println!("\t\t{:?}:{:?}->{:?}|{:?}", t.action.name(), t.from.name(), t.to.name(), t.prob);
        }
    }
}

fn policy_improvement() {

}

fn phase_external() {

}

fn evaluate_policy(state:&mut Vec<State>, theta: f32) {
    loop {
        let mut delta:f32 = 0.0;
        for s in state.iter_mut() {
            let v_old = s.state_v;
            let v_new = s.action.iter()
                .map(|a| a.action.reward as f32 + a.to.state_v)
                .sum();
            s.state_v = v_new;
            delta = delta.max((v_new - v_old).abs());
        }
        if delta < theta { break }
    }
}

pub fn run() {
    let mut g = Graph::new();
    g.setup();
    g.print_info();
}
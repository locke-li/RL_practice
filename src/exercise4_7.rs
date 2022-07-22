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

struct State<'a> {
    pub name: String,
    pub reward: i32,
    pub action: Vec<Transition<'a>>,
    pub state_v: f32,
}

struct Action {
    pub name: String,
    pub reward: i32,
}

struct Transition<'a> {
    pub action: &'a Action,
    pub from: &'a State<'a>,
    pub to: &'a State<'a>,
    pub prob: f32,
}

impl<'a> Graph<'a> {
    fn new() -> Self {
        Self {
            state: Vec::new(), state_lookup: BTreeMap::new(),
            action: Vec::new(), action_lookup: BTreeMap::new(),
            theta: 0.0, policy_v: 0.0,
        }
    }

    fn add_state(&mut self, name:String, reward:i32) -> &State {
        let state = State { name, reward, action: Vec::new(), state_v: 0.0 };
        self.state.push(state);
        let state:*mut State = self.state.last_mut().unwrap();
        unsafe {
            self.state_lookup.insert(&(*state).name, state);
            &(*state)
        }
    }

    fn add_action(&mut self, name:String, reward:i32) -> &Action {
        let action = Action { name, reward };
        self.action.push(action);
        let action:*const Action = self.action.last().unwrap();
        unsafe {
            self.action_lookup.insert(&(*action).name, action);
            &(*action)
        }
    }

    fn add_transition(&self, action:&str, from:&'a str, to:&str, prob:f32) {
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

    fn print(&self) {
        // println!("lookup");
        // for (n, a) in self.action_lookup.iter() {
        //     unsafe { println!("{} {}", n, (**a).name); }
        // }
        // for (n, s) in self.state_lookup.iter() {
        //     unsafe { println!("{} {}", n, (**s).name); }
        // }
        println!("action:");
        for a in self.action.iter() {
            println!("\t{:?}:{:?}", a.name, a.reward);
        }
        println!("state:");
        for s in self.state.iter() {
            println!("\t{:?}:{:?}", s.name, s.reward);
            for t in s.action.iter() {
                println!("\t\t{:?}:{:?}->{:?}|{:?}", t.action.name, t.from.name, t.to.name, t.prob);
            }
        }
    }
}

fn parse_graph<'a>() -> Graph<'a> {
    // let s = "
    //     theta:1
    //     stay:0
    //     move:-4
    //     s0,0
    //     >stay,s0,0.5,0
    //     >move,s1,0.5,0
    //     s1,0
    //     >stay,s1,0.5,0
    //     >move,s0,0.5,0
    // ";
    //TODO parse from text
    let mut g = Graph::new();
    g.add_state(String::from("s0"), 0);
    g.add_state(String::from("s1"), 0);
    let state_name:Vec<&String> = g.state.iter().map(|s| unsafe { &(*(&s.name as *const String)) }).collect();
    let prob = 1.0 / 6.0;
    for k in 0..=5 {
        let action = g.add_action(format!("move{}", k), k * -2) as *const Action;
        for s_from in state_name.iter() {
            for s_to in state_name.iter() {
                if s_from != s_to {
                    unsafe { g.add_transition(&(*action).name, &s_from, &s_to, prob); }
                }
            }
        }
    }
    // g.add_transition("stay", "s0", "s0", 0.5);
    // g.add_transition("move", "s0", "s1", 0.5);
    // g.add_transition("stay", "s1", "s1", 0.5);
    // g.add_transition("move", "s1", "s0", 0.5);
    g
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
    let g = parse_graph();
    g.print();
}
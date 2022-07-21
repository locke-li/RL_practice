use std::collections::BTreeMap;

struct GraphPrototype<'a> {
    pub state: Vec<StatePrototype<'a>>,
    pub action: Vec<ActionPrototype<'a>>,
    pub transition: Vec<TransitionPrototype>,
}

struct GraphLookup<'a> {
    pub state: BTreeMap<&'a str, usize>,
    pub action: BTreeMap<&'a str, usize>,
}

struct StatePrototype<'a> {
    pub name: &'a str,
    pub reward: i32,
    pub index: usize,
}

struct ActionPrototype<'a> {
    pub name: &'a str,
    pub reward: i32,
    pub index: usize,
}

struct TransitionPrototype {
    pub action: usize,
    pub from: usize,
    pub to: usize,
    pub prob: f32,
}

struct Graph<'a> {
    pub state: Vec<State<'a>>,
    pub policy_v: f32,
    pub state_v: Vec<f32>,
}

struct State<'a> {
    pub proto: &'a StatePrototype<'a>,
    pub action_list: Vec<Action<'a>>,
}

struct Action<'a> {
    pub proto: &'a ActionPrototype<'a>,
    pub to: &'a State<'a>,
    pub prob: f32,
}

impl<'a> State<'a> {
    fn new(proto:&'a StatePrototype) -> Self {
        Self { proto, action_list: Vec::new() }
    }
}

impl<'a> Action<'a> {
    fn new(proto:&'a ActionPrototype, to:&'a State, prob:f32) -> Self {
        Self { proto, to, prob }
    }
}

impl<'a> GraphPrototype<'a> {
    fn new() -> Self {
        Self {
            state: Vec::new(),
            action: Vec::new(),
            transition: Vec::new(),
        }
    }

    fn add_state(&mut self, name:&'a str, reward:i32) {
        let state = StatePrototype { name, reward, index: self.state.len() };
        self.state.push(state);
    }

    fn add_action(&mut self, name:&'a str, reward:i32) {
        let action = ActionPrototype { name, reward, index: self.action.len() };
        self.action.push(action);
    }

    fn add_transition(&mut self, lookup: &GraphLookup, action:&str, from:&str, to:&str, prob:f32) {
        let action = lookup.action[action];
        let from = lookup.state[from];
        let to = lookup.state[to];
        self.transition.push(TransitionPrototype { action, from, to, prob });
    }
}

impl<'a> GraphLookup<'a> {
    fn new(proto:&GraphPrototype<'a>) -> Self {
        Self {
            state: proto.state.iter().map(|s| (s.name, s.index)).collect(),
            action: BTreeMap::new()
        }
    }
}

fn parse_graph_prototype<'a>() -> GraphPrototype<'a> {
    // let s = "
    //     stay:0
    //     move:-4
    //     s0,0
    //     >stay,s0,0.5,0
    //     >move,s1,0.5,0
    //     s1,0
    //     >stay,s1,0.5,0
    //     >move,s0,0.5,0
    // ";
    let mut g = GraphPrototype::new();
    //TODO parse from text
    g.add_state("s0", 0);
    g.add_state("s1", 0);
    g.add_action("stay", 0);
    g.add_action("move", -4);
    let l = GraphLookup::new(&g);
    g.add_transition(&l, "stay", "s0", "s0", 0.5);
    g.add_transition(&l, "move", "s0", "s1", 0.5);
    g.add_transition(&l, "stay", "s1", "s1", 0.5);
    g.add_transition(&l, "move", "s1", "s0", 0.5);
    g
}

fn setup_graph<'a>(gp:&'a GraphPrototype) -> Graph<'a> {
    let state:Vec<State> = gp.state.iter()
        .map(|p| State::new(p))
        .collect();
    let mut g = Graph { state, policy_v: 0.0, state_v: Vec::new() };
    for t in gp.transition.iter() {
        let a = Action::new(&gp.action[t.action], &g.state[t.to], t.prob);
        g.state[t.from].action_list.push(a);
    }
    g
}

fn phase_improve() {

}

fn phase_external() {

}

fn evaluate_policy() {

}

fn select_action() {

}

fn reward() {

}

pub fn run() {
    let gProto = parse_graph_prototype();
    let g = setup_graph(&gProto);
}
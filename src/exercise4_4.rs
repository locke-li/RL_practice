use std::collections::BTreeMap;

struct GraphPrototype<'a> {
    pub state: BTreeMap<&'a str, StatePrototype<'a>>,
    pub action: BTreeMap<&'a str, ActionPrototype<'a>>,
}

struct StatePrototype<'a> {
    pub name: &'a str,
    pub reward: i32,
}

struct ActionPrototype<'a> {
    pub name: &'a str,
    pub reward: i32,
}

struct Graph<'a> {
    pub state_list: Vec<State<'a>>,
    pub policy_v: f32,
    pub state_v: Vec<f32>,
}

struct State<'a> {
    pub proto: &'a StatePrototype<'a>,
    pub action_list: Vec<StateAction<'a>>,
}

struct StateAction<'a> {
    pub proto: &'a ActionPrototype<'a>,
    pub to_state: i32,
    pub prob: f32,
}

impl<'a> State<'a> {
    fn new(proto_:&'a StatePrototype) -> Self {
        Self { proto: proto_, action_list: Vec::new() }
    }

    fn action(&mut self, proto_:&'a ActionPrototype, to_state_:i32,  prob_:f32) {
        self.action_list.push(StateAction { proto: proto_, to_state: to_state_, prob: prob_ });
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
    let mut state: Vec<StatePrototype> = Vec::new();
    let mut action: Vec<ActionPrototype> = Vec::new();
    //TODO
    GraphPrototype {
        state: state.into_iter().map(|s| (s.name, s)).collect(),
        action: action.into_iter().map(|a| (a.name, a)).collect(),
    }
}

fn setup_graph<'a>(proto:&GraphPrototype) -> Graph<'a> {
    //TODO
    let state:Vec<State> = Vec::new();
    let mut s0 = State::new("s0", 0);
    let mut s1 = State::new("s1", 0);
    s0.action(&s0, 0.5, 0);
    s0.action(&s1, 0.5, -4);
    s1.action(&s0, 0.5, -4);
    s1.action(&s1, 0.5, 0);
    Graph { state_list: state, policy_v: 0.0, state_v: Vec::new() }
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
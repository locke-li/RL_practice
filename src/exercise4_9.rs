use std::cmp::{ min, max };
use std::error::Error;
use plotters::{prelude::*, coord::Shift};

struct Graph {
    pub state:Vec<State>,
}

struct GraphInfo {
    pub theta:f64,
    pub p_win:f64,
    pub state_range:i32,
    pub state_active:(i32, i32),
    pub print_per_line:usize,
}

struct State {
    pub capital:i32,
    pub reward:f64,
    pub state_v:f64,
}

struct Policy {
    pub state_action:Vec<i32>,
    pub state_v_max:f64,
    pub action_max:i32,
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
        let s_t = self.state.last_mut().unwrap();
        s_t.state_v = s_t.reward;
    }

    fn expected_reward(&self, s:&State, a:i32, gi:&GraphInfo) -> f64 {
        let s_win = min(s.capital + a, gi.state_range) as usize;
        let s_lose = max(s.capital - a, 0) as usize;
        let r_win = self.state[s_win].state_v * gi.p_win;
        let r_lose = self.state[s_lose].state_v * (1.0 - gi.p_win);
        s.reward + r_win + r_lose
    }

    // fn print_state(&self, gi:&GraphInfo) {
    //     let mut k = 0;
    //     for s in self.state.iter() {
    //         print!("\t{}: {:.2}", s.capital, s.state_v);
    //         k += 1;
    //         if k> 0 && k % gi.print_per_line == 0 { println!(); }
    //     }
    //     println!();
    // }

    fn print_policy(&self, p:&Policy, gi:&GraphInfo) {
        let p_v = &p.state_action;
        for k in 0..p_v.len() {
            print!("\t{}: {}", k, p_v[k]);
            if k> 0 && k % gi.print_per_line == 0 { println!(); }
        }
        println!();
    }

    fn draw_policy(&self, p:&Policy, gi:&GraphInfo, canvas:&DrawingArea<BitMapBackend, Shift>) -> Result<(), Box<dyn Error>> {
        let (s_min, s_max) = gi.state_active;
        let mut chart = ChartBuilder::on(canvas)
            .margin(5)
            .x_label_area_size(50)
            .y_label_area_size(50)
            .build_cartesian_2d(0..gi.state_range, 0..p.action_max)?;
        chart.configure_mesh().draw()?;
        chart.draw_series(LineSeries::new(
            (s_min..s_max).map(|i| (i, p.state_action[i as usize]))
            , &BLUE))?;
        Ok(())
    }
}

impl Policy {
    fn new(gi:&GraphInfo) -> Self {
        let mut state_action:Vec<i32> = Vec::new();
        state_action.resize((gi.state_range + 1) as usize, 0);
        Self { state_action, state_v_max:0.0, action_max:0 }
    }
}

fn value_iteration(g:&mut Graph, gi:&GraphInfo, canvas:&DrawingArea<BitMapBackend, Shift>) -> Result<(), Box<dyn Error>> {
    let pg:*const Graph = g;
    //hack to grant shared access to graph
    let gs = unsafe { &(*pg) };
    let (s_min, s_max) = gi.state_active;
    let mut chart = ChartBuilder::on(canvas)
        .margin(5)
        .x_label_area_size(50)
        .y_label_area_size(50)
        .build_cartesian_2d(0..gi.state_range, -0.1f64..1.0f64)?;
    chart.configure_mesh().draw()?;
    let mut sweep = 0;
    let sweep_band = 10;
    loop {
        let mut delta:f64 = 0.0;
        for k in s_min..=s_max {
            let s = &mut g.state[k as usize];
            let v_old = s.state_v;
            let bet_max = min(s.capital, gi.state_range - s.capital);
            let v_new = (1..=bet_max).map(|a| gs.expected_reward(s, a, gi))
                .max_by(|x, y| x.total_cmp(y)).unwrap();
            s.state_v = v_new;
            delta = delta.max((v_new - v_old).abs());
        }
        let sweepf = ((1.0f32).min(sweep as f32 / sweep_band as f32) * 255.0).round() as u8;
        let color = if sweep % 2 == 0 { RGBColor { 0:sweepf, 1:128, 2:128 } }
            else { RGBColor { 0:128, 1:sweepf, 2:128 } };
        let line = chart.draw_series(LineSeries::new(
            (s_min..s_max).map(|i| (i, g.state[i as usize].state_v))
            , &color))?;
        if sweep < sweep_band {
            line.label(format!("sweep {}", sweep))
            .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], color));
        }
        // gs.print_state(&gi);
        sweep += 1;
        if delta < gi.theta { break }
    }
    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;
    Ok(())
}

fn check_policy(p:&mut Policy, g:&Graph, gi:&GraphInfo) {
    let (s_min, s_max) = gi.state_active;
    let mut v_max:f64 = 0.0;
    let mut a_max = 0;
    for k in s_min..=s_max {
        let ki = k as usize;
        let s = &g.state[ki];
        let bet_max = min(s.capital, gi.state_range - s.capital);
        let (a, v) = (1..=bet_max).rev().map(|a| (a, g.expected_reward(s, a, gi)))
            .max_by(|(_, x), (_, y)| x.total_cmp(y)).unwrap();
        println!("{} {}|{:.4}", k, a, s.state_v);
        (1..=bet_max).map(|a| (a, g.expected_reward(s, a, gi)))
            .for_each(|(a, v)| println!("{}:{}", a, v));
        p.state_action[ki] = a;
        v_max = v_max.max(v);
        a_max = a_max.max(a);
    }
    p.state_v_max = v_max;
    p.action_max = a_max;
}

pub fn run() -> Result<(), Box<dyn Error>> {
    let mut g = Graph::new();
    let g_info = GraphInfo {
        theta: 0.001,
        p_win: 0.55,
        state_range: 100,
        state_active: (1, 99),
        print_per_line: 10,
    };
    g.setup(&g_info);
    let mut p = Policy::new(&g_info);
    let file = format!("4_9_p{}.png", g_info.p_win);
    let canvas = BitMapBackend::new(&file, (1440, 1440)).into_drawing_area();
    canvas.fill(&WHITE)?;
    let canvas_split = canvas.split_evenly((2, 1));
    value_iteration(&mut g, &g_info, &canvas_split[0])?;
    check_policy(&mut p, &g, &g_info);
    g.print_policy(&p, &g_info);
    g.draw_policy(&p, &g_info, &canvas_split[1])?;
    canvas.present()?;
    Ok(())
}
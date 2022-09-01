
mod nd_vec;
mod poisson;
mod exercise4_7;
mod exercise4_9;
mod exercise5_12;
mod exercise6_9;

fn main() {
    let index = "6_9";
    let ret = match index {
        //policy iteraction: policy evaluation -> policy improvement, equiprobable
        "4_7" => exercise4_7::run(),
        //value iteraction -> policy, equiprobable
        "4_9" => exercise4_9::run(),
        //monte carlo off-policy b:Ɛ-soft
        "5_12" => exercise5_12::run(),
        //SARSA
        "6_9" => exercise6_9::run(),
        _ => Err(format!("invalid index {}", index).into()),
    };
    match ret {
        Ok(_) => {},
        Err(e) => { println!("{}", e.to_string()) }
    };
}

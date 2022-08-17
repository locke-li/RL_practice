
mod nd_vec;
mod poisson;
mod exercise4_7;
mod exercise4_9;
mod exercise5_12;

fn main() {
    let index = "5_12";
    let ret = match index {
        //policy iteraction: policy evaluation -> policy improvement, equiprobable
        "4_7" => exercise4_7::run(),
        //value iteraction -> policy, equiprobable
        "4_9" => exercise4_9::run(),
        //monte carlo off-poicy b:Ɛ-soft
        "5_12" => exercise5_12::run(),
        _ => Err(format!("invalid index {}", index).into()),
    };
    match ret {
        Ok(_) => {},
        Err(e) => { println!("{}", e.to_string()) }
    };
}

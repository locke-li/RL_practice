
mod nd_vec;
mod poisson;
mod exercise4_7;
mod exercise4_9;

fn main() {
    let index = "4_9";
    let ret = match index {
        "4_7" => exercise4_7::run(),
        "4_9" => exercise4_9::run(),
        _ => Err(format!("invalid index {}", index).into()),
    };
    match ret {
        Ok(_) => {},
        Err(e) => { println!("{}", e.to_string()) }
    };
}

use twistytimer::ParseConfig;

mod twistytimer;

fn main() {
    let config = ParseConfig::default();

    let set = twistytimer::parse_twistytimer("twistytimer.csv", Some(config)).unwrap();

    println!("{:?}", set);
}

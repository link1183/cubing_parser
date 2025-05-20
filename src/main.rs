// use twistytimer::ParseConfig;

mod models;
mod mysql;
mod twistytimer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let config = ParseConfig::default();

    // let set = twistytimer::parse_twistytimer("twistytimer.csv", Some(config)).unwrap();

    // println!("{:?}", set);

    mysql::main()
}

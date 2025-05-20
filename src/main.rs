mod models;
mod mysql;
mod twistytimer;
use dotenv::dotenv;
use mysql::get_conn;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let set = twistytimer::parse_twistytimer("twistytimer.csv").unwrap();

    println!("{:?}", set);

    let conn = get_conn()?;

    Ok(())
}

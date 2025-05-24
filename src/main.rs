mod models;
mod mysql;
mod twistytimer;
use dotenv::dotenv;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let args: Vec<String> = env::args().collect();
    let file_path = if args.len() > 1 {
        &args[1]
    } else {
        "data/twistytimer.csv"
    };

    mysql::import_twistytimer_csv(file_path)?;

    println!("Successfully imported TwistyTimer data to database");

    Ok(())
}

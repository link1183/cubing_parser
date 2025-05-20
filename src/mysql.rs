use mysql::prelude::*;
use mysql::*;

pub fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let url = "mysql://root:example@localhost:3306/solves";
    let pool = Pool::new(url)?;

    let mut conn = pool.get_conn()?;

    conn.query_drop(
        r"
        INSERT INTO `event` (`event_name`)
        VALUES ('333'), ('222')
    ",
    )?;

    Ok(())
}

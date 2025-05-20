use mysql::prelude::*;
use mysql::*;
use std::env;

pub fn get_conn() -> Result<PooledConn, Box<dyn std::error::Error>> {
    let url = env::var("URL").unwrap();
    let pool = Pool::new(url.as_str())?;

    Ok(pool.get_conn()?)
}

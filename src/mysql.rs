use crate::models::{TwistyTimer, TwistyTimerPuzzles};
use mysql::prelude::*;
use mysql::*;
use std::env;
use std::time::{Duration, Instant};

pub fn get_conn() -> Result<PooledConn, Box<dyn std::error::Error>> {
    let url = env::var("URL").unwrap();
    let pool = Pool::new(url.as_str())?;

    Ok(pool.get_conn()?)
}

fn format_duration(duration: Duration) -> String {
    let total_secs = duration.as_secs();
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;

    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}

pub fn insert_solves(
    conn: &mut PooledConn,
    solves: Vec<TwistyTimer>,
) -> Result<(), Box<dyn std::error::Error>> {
    let total_solves = solves.len();
    println!("Starting insertion of {} solves...", total_solves);

    let start_time = Instant::now();
    let mut last_update_time = start_time;

    let mut existing_events: Vec<String> = Vec::new();
    conn.query_map("SELECT event_name FROM event", |event_name: String| {
        existing_events.push(event_name)
    })?;
    println!(
        "Found {} existing event types in database",
        existing_events.len()
    );

    let mut tx = conn.start_transaction(TxOpts::default())?;

    tx.query_drop("SET foreign_key_checks = 0")?;
    let insert_solve_stmt = tx.prep(
        "INSERT IGNORE INTO solve (event_name, time, scramble, date, session_name, penalty, comment) 
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )?;

    let mut processed_count = 0;
    let mut skipped_unknown_count = 0;
    let mut skipped_duplicate_count = 0;

    let batch_size = 20000;

    for (chunk_index, chunk) in solves.chunks(batch_size).enumerate() {
        let params_start_time = Instant::now();
        let mut params = Vec::with_capacity(chunk.len());

        for solve in chunk {
            let event_name = match &solve.puzzle {
                TwistyTimerPuzzles::EventFMC => "fmc".to_string(),
                TwistyTimerPuzzles::Unknown(s) => {
                    format!("unknown:{}", s)
                }
                _ => solve.puzzle.to_string(),
            };

            if !existing_events.contains(&event_name) {
                skipped_unknown_count += 1;
                continue;
            }

            let penalty = match solve.penalty.as_str() {
                "+2" => 1,
                "DNF" => 2,
                _ => 0,
            };

            let date_str = solve.date.format("%Y-%m-%d %H:%M:%S").to_string();

            params.push((
                event_name,
                solve.time,
                &solve.scramble,
                date_str,
                &solve.category,
                penalty,
                &solve.comment,
            ));

            processed_count += 1;
        }

        let params_prep_time = params_start_time.elapsed();

        if !params.is_empty() {
            let insert_start_time = Instant::now();
            let params_count = params.len();

            tx.exec_batch(&insert_solve_stmt, params)?;

            let affected_rows = tx.affected_rows();
            if affected_rows < params_count as u64 {
                skipped_duplicate_count += (params_count as u64 - affected_rows) as usize;
            }

            let insert_time = insert_start_time.elapsed();

            let progress_pct = (processed_count as f64 / total_solves as f64) * 100.0;
            let elapsed = start_time.elapsed();
            let rows_per_sec = processed_count as f64 / elapsed.as_secs_f64();

            let remaining_rows = total_solves as i64 - processed_count as i64;
            let est_remaining_secs = if rows_per_sec > 0.0 {
                (remaining_rows as f64 / rows_per_sec).max(0.0)
            } else {
                0.0
            };
            let est_remaining = Duration::from_secs_f64(est_remaining_secs);

            let now = Instant::now();
            if now.duration_since(last_update_time) >= Duration::from_secs(2) {
                println!(
                    "Batch {}: Processed {} rows ({:.1}%) | Rate: {:.0} rows/sec | Time: {} | ETA: {} | Prep: {:.2}s | Insert: {:.2}s",
                    chunk_index + 1,
                    processed_count,
                    progress_pct,
                    rows_per_sec,
                    format_duration(elapsed),
                    format_duration(est_remaining),
                    params_prep_time.as_secs_f64(),
                    insert_time.as_secs_f64()
                );
                last_update_time = now;
            }
        }
    }

    tx.query_drop("SET foreign_key_checks = 1")?;
    tx.commit()?;

    let total_time = start_time.elapsed();
    let rows_per_sec = processed_count as f64 / total_time.as_secs_f64();

    // Calculate number of new inserts
    let new_inserts = processed_count - skipped_duplicate_count;

    println!("--------------------------------------------------");
    println!("IMPORT SUMMARY:");
    println!("--------------------------------------------------");
    println!("Total solves in CSV:       {}", total_solves);
    println!("Total solves processed:    {}", processed_count);
    println!("New inserts:               {}", new_inserts);
    println!("Skipped (duplicates):      {}", skipped_duplicate_count);
    println!("Skipped (unknown events):  {}", skipped_unknown_count);
    println!(
        "Total skipped:             {}",
        skipped_duplicate_count + skipped_unknown_count
    );
    println!("Total time:                {}", format_duration(total_time));
    println!("Average speed:             {:.0} rows/second", rows_per_sec);
    println!("--------------------------------------------------");

    Ok(())
}

pub fn import_twistytimer_csv(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("=================================================");
    println!("Starting import from: {}", file_path);
    println!("=================================================");

    let parse_start = Instant::now();

    let solves = crate::twistytimer::parse_twistytimer(file_path)?;

    let parse_time = parse_start.elapsed();
    println!(
        "Parsed {} solves in {:.2}s ({:.0} rows/sec)",
        solves.len(),
        parse_time.as_secs_f64(),
        solves.len() as f64 / parse_time.as_secs_f64()
    );

    let conn_start = Instant::now();
    let mut conn = get_conn()?;
    println!(
        "Connected to database in {:.2}s",
        conn_start.elapsed().as_secs_f64()
    );

    // Insert the solves into the database
    insert_solves(&mut conn, solves)?;

    println!("=================================================");
    println!("Import completed successfully!");
    println!("=================================================");

    Ok(())
}

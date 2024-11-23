use std::{fs::{File, OpenOptions}, io::Write};

use futures::TryStreamExt;
use sqlx::{Column, Row};
use tauri::Emitter;

async fn execute_query(
    host: String,
    port: String,
    user: String,
    pwd: String,
    db: String,
    table: String,
    sqlsrc: String,
    repcol: String,
    outpath: String,
    window: tauri::Window,
) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!("mysql://{}:{}@{}:{}/{}", user, pwd, host, port, db);

    let pool: sqlx::Pool<sqlx::MySql> = match sqlx::MySqlPool::connect(&url).await {
        Ok(pool) => pool,
        Err(err) => {
            eprintln!("connError: {}", err);
            window.emit("connErr", err.to_string()).unwrap();
            return Err(Box::new(err));
        }
    };

    File::create(format!("{}/logs.log", outpath)).expect("Failed to create file");
    let mut log_file = OpenOptions::new()
        .append(true)
        .open(format!("{}/logs.log", outpath))?;
    let check_msg = format!("Checking {}, please wait...", &table);
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let check_msg_log = format!("{} => {}\n", &timestamp, &check_msg);
    log_file.write_all(check_msg_log.as_bytes())?;

    // query headers
    let sql_query_header = format!("{} LIMIT 10", sqlsrc.clone());
    match sqlx::query(&sql_query_header).fetch_one(&pool).await {
        Ok(rows) => {
            let col_num = rows.columns().len();
            let mut vec_col_name: Vec<&str> = Vec::new();
            let mut vec_col_type: Vec<String> = Vec::new();
            for num in 0..col_num {
                vec_col_name.push(rows.column(num).name());
                vec_col_type.push(rows.column(num).type_info().to_string())
            }

            // execute query
            let sql_query = format!("{}", sqlsrc);
            let mut stream = sqlx::query(&sql_query).fetch(&pool);

            let emit_msg = format!("{}", table);
            let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
            let check_done_log = format!("{} => {}\n", &timestamp, &emit_msg);
            log_file.write_all(check_done_log.as_bytes())?;

            let folder_path = format!("{}\\{}", outpath, &table);

            if !folder_exists(&folder_path) {
                std::fs::create_dir(&folder_path)?;
            }

            // save path
            let output_path = format!("{}\\{}.csv", &folder_path, table);
            let mut wtr = csv::WriterBuilder::new()
                .delimiter(b'|')
                .from_path(output_path)?;

            // write headers
            wtr.serialize(vec_col_name.clone())?;
            while let Some(row) = stream.try_next().await? {
                let mut vec_wtr_str = Vec::new();
                for num in 0..col_num {
                    let value = match &vec_col_type[num][..] {
                        "DECIMAL" => {
                            let num: rust_decimal::Decimal = row.get(num);
                            num.to_string()
                        }
                        "DOUBLE" => {
                            let num: f64 = row.get(num);
                            num.to_string()
                        }
                        "FLOAT" => {
                            let num: f32 = row.get(num);
                            num.to_string()
                        }
                        "SMALLINT" | "TINYINT" => {
                            let num: i16 = row.get(num);
                            num.to_string()
                        }
                        "INT" | "MEDIUMINT" | "INTEGER" => {
                            let num: i32 = row.get(num);
                            num.to_string()
                        }
                        "BIGINT" => {
                            let num: i64 = row.get(num);
                            num.to_string()
                        }
                        "INT UNSIGNED" => {
                            let num: u32 = row.get(num);
                            num.to_string()
                        }
                        "DATETIME" => {
                            let num: chrono::DateTime<chrono::Local> = row.get(num);
                            num.to_string()
                        }
                        "DATE" => {
                            let num: sqlx::types::time::Date = row.get(num);
                            num.to_string()
                        }
                        "BOOLEAN" | "BOOL" => {
                            let num: i16 = row.get(num);
                            num.to_string()
                        }
                        _ if vec_col_name[num] == &repcol => {
                            let value: &str = row.get(num);
                            value.replace("|", "").to_string()
                        }
                        _ => {
                            let num: &str = row.get(num);
                            num.to_string()
                        }
                    };
                    vec_wtr_str.push(value);
                }
                wtr.serialize(vec_wtr_str)?;
            }
            wtr.flush()?;
            let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
            let output = format!("{}\\{}.csv", &folder_path, table);
            let output_log = format!("{} => {}\n", &timestamp, output);
            log_file.write_all(output_log.as_bytes())?;

            window.emit("message", &emit_msg)?;
        }
        Err(error) => {
            let err_msg = format!("Error with {}: {}", &table, error);
            let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
            let err_msg_log = format!("{} => {}\n", &timestamp, &err_msg);
            window.emit("errcode", &err_msg)?;
            let _failed_file = std::fs::File::create(format!("{}/failed.log", outpath))
                .expect("Failed to create file");
            let mut failed_file = std::fs::OpenOptions::new()
                .append(true)
                .open(format!("{}/failed.log", outpath))?;
            failed_file
                .write_all(err_msg.as_bytes())
                .expect("Failed to write to file");
            log_file.write_all(&err_msg_log.as_bytes())?;
        }
    }

    let msg_done = "Download done.".to_string();
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let msg_done_log = format!("{} => {}\n", &timestamp, &msg_done);
    log_file.write_all(msg_done_log.as_bytes())?;

    Ok(())
}

fn folder_exists(path: &str) -> bool {
    std::fs::metadata(path).is_ok()
}

#[tauri::command]
pub async fn download(
    host: String,
    port: String,
    user: String,
    pwd: String,
    db: String,
    table: String,
    sqlsrc: String,
    repcol: String,
    outpath: String,
    window: tauri::Window,
) {
    let window_exec = window.clone();
    match execute_query(
        host, port, user, pwd, db, table, sqlsrc, repcol, outpath, window,
    )
    .await
    {
        Ok(result) => result,
        Err(error) => {
            eprintln!("query Error: {}", error);
            window_exec.emit("queryErr", &error.to_string()).unwrap();
            return ();
        }
    };
}

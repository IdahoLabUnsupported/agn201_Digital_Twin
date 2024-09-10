#![feature(closure_track_caller)]
mod errors;
mod tests;

use crate::errors::ISUProcessorError;
use chrono::NaiveDateTime;
use jester_core::errors::ProcessorError;
use jester_core::DataSourceMessage;
use sqlx::sqlite::SqliteRow;
use sqlx::{Error, Pool, Row, Sqlite};
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::PathBuf;
use rusqlite::Connection;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

pub struct ISUProcessor {
    conn: rusqlite::Connection
}
pub struct ISUFile {
    path: String,
    last_position_read: i64,
    last_index: i32,
    headers: String,
    time: String, // time string as pulled from the file directory
}

impl ISUProcessor {
    fn new() -> Result<ISUProcessor, ISUProcessorError> {
        let conn = Connection::open("./agn201_plugin")?;
        Ok(ISUProcessor {conn})
    }
}
impl jester_core::Processor for ISUProcessor {
    fn init(&self, db: Pool<Sqlite>) -> Result<(), ProcessorError> {

        self.conn.execute("CREATE TABLE IF NOT EXISTS isu (path text UNIQUE ON CONFLICT REPLACE, last_position_read integer, headers text, time text, last_index integer);", []).map_err(|e| ProcessorError::from(ISUProcessorError::Rusqlite(e)))?;
        Ok(())

    /*    // in order to use the Tokio runtime to do blocking operations on async functions we must
        // run it in a separate thread because we don't know what the caller is using
        let result = std::thread::spawn(move || {
            Runtime::new()
                .unwrap()
                .block_on(
                    sqlx::query(
                        // last position read must be text because sqlite doesn't have a bigint type
                        "CREATE TABLE IF NOT EXISTS isu (path text UNIQUE ON CONFLICT REPLACE, last_position_read integer, headers text, time text, last_index integer);",
                    )
                    .execute(&db),
                )
                .expect("unable to run ISU processor migration scripts");
        })
        .join();

        match result {
            Ok(_) => Ok(()),
            Err(_) => Err(ProcessorError::from(ISUProcessorError::ThreadError)),
        }*/
    }

    fn process(
        &self,
        file: PathBuf,
        db: Pool<Sqlite>,
        timeseries_chan: Option<UnboundedSender<DataSourceMessage>>,
        graph_chan: Option<UnboundedSender<DataSourceMessage>>,
    ) -> Result<(), ProcessorError> {
        let path = match file.to_str() {
            None => return Err(ProcessorError::from(ISUProcessorError::BlankPath)),
            Some(p) => p,
        };

        if path.contains("Events") {
            let result = match fetch_file(path, &self.conn)? {
                None => initial_event_process(file, &self.conn)?,
                // on some we're basically tailing the file so run the tail function
                Some(f) => tail_event_process(f, file, &self.conn)?,
            };

            timeseries_chan
                .ok_or(ISUProcessorError::NoChannelError)?
                .send(DataSourceMessage::File((result, true)))?;
        } else {
            let result = match fetch_file(path, &self.conn)? {
                None => initial_process(file, &self.conn)?,
                // on some we're basically tailing the file so run the tail function
                Some(f) => tail_process(f, file, &self.conn)?,
            };

            timeseries_chan
                .ok_or(ISUProcessorError::NoChannelError)?
                .send(DataSourceMessage::File((result, true)))?;
        }

        Ok(())
    }
}

fn initial_process(path: PathBuf, db: &Connection) -> Result<PathBuf, ISUProcessorError> {
    let uuid = Uuid::new_v4();
    let file = File::open(&path)?;
    let output_file = File::create(format!("{uuid}.csv"))?;
    let mut reader = BufReader::new(file);
    let mut writer = csv::WriterBuilder::new()
        .flexible(true)
        .from_writer(output_file);

    let mut header_count = 0;
    loop {
        let mut s = String::new();
        reader.read_line(&mut s)?;

        if s.contains("***End_of_Header***") {
            header_count += 1
        }

        // total of two headers in this file
        if header_count >= 2 {
            break;
        }
    }

    let parent = &path
        .parent()
        .ok_or(ISUProcessorError::BlankPath)?
        .components()
        .last()
        .ok_or(ISUProcessorError::BlankPath)?;

    let time = NaiveDateTime::parse_from_str(
        parent
            .as_os_str()
            .to_str()
            .ok_or(ISUProcessorError::BlankPath)?,
        "%b_%d_%Y_%H_%M",
    )?;

    let mut csv_reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(reader);

    let mut headers = csv_reader.headers()?.clone();
    headers.truncate(headers.len() - 1);
    headers.push_field("DateTime");
    writer.write_byte_record(headers.as_byte_record())?;

    for result in csv_reader.records() {
        let mut record = result?;

        record.push_field(format!("{time}").as_str());
        writer.write_byte_record(record.as_byte_record())?;
    }

    let path = match path.into_os_string().into_string() {
        Ok(s) => s,
        Err(_) => return Err(ISUProcessorError::BlankPath),
    };

    let headers: Vec<String> = headers.deserialize(None)?;

    writer.flush()?;
    save_file(
        ISUFile {
            path,
            last_position_read: csv_reader.into_inner().stream_position()?.try_into()?,
            last_index: 0,
            headers: headers.join(","),
            time: format!("{time}"),
        },
        db,
    )?;

    Ok(PathBuf::from(format!("{uuid}.csv")))
}

fn initial_event_process(path: PathBuf, db: &Connection) -> Result<PathBuf, ISUProcessorError> {
    let uuid = Uuid::new_v4();
    let file = File::open(&path)?;
    let output_file = File::create(format!("{uuid}.csv"))?;
    let mut reader = BufReader::new(file);
    let mut writer = csv::WriterBuilder::new()
        .flexible(true)
        .from_writer(output_file);

    let parent = &path
        .parent()
        .ok_or(ISUProcessorError::BlankPath)?
        .components()
        .last()
        .ok_or(ISUProcessorError::BlankPath)?;

    let time = NaiveDateTime::parse_from_str(
        parent
            .as_os_str()
            .to_str()
            .ok_or(ISUProcessorError::BlankPath)?,
        "%b_%d_%Y_%H_%M",
    )?;

    writer.write_record(["Event", "Index", "DateTime"])?;

    let mut s = String::new();
    let mut i = 0;
    while let Ok(u) = reader.read_line(&mut s) {
        if u == 0 {
            break;
        }

        s.pop();
        writer.write_record([
            s[0..s.len() - 2].to_string(),
            format!("{i}"),
            format!("{time}"),
        ])?;
        s = String::new();
        i += 1;
    }

    let path = match path.into_os_string().into_string() {
        Ok(s) => s,
        Err(_) => return Err(ISUProcessorError::BlankPath),
    };

    writer.flush()?;
    save_file(
        ISUFile {
            path,
            last_position_read: reader.stream_position()?.try_into()?,
            last_index: i,
            headers: String::from("Event,Index,DateTime"),
            time: format!("{time}"),
        },
        db,
    )?;

    Ok(PathBuf::from(format!("{uuid}.csv")))
}

fn tail_process(
    mut db_file: ISUFile,
    path: PathBuf,
    db: &Connection,
) -> Result<PathBuf, ISUProcessorError> {
    let uuid = Uuid::new_v4();
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    reader.seek(SeekFrom::Start(db_file.last_position_read.try_into()?))?;

    let output_file = File::create(format!("{uuid}.csv"))?;
    let mut csv_writer = csv::WriterBuilder::new()
        .flexible(true)
        .from_writer(output_file);

    let mut csv_reader = csv::ReaderBuilder::new()
        .flexible(true)
        .has_headers(false)
        .from_reader(reader);

    csv_writer.write_record(db_file.headers.split(','))?;

    let time = db_file.time.clone();
    for result in csv_reader.records() {
        let mut record = result?;

        record.push_field(time.to_string().as_str());
        csv_writer.write_byte_record(record.as_byte_record())?;
    }

    csv_writer.flush()?;
    db_file.last_position_read = csv_reader.into_inner().stream_position()?.try_into()?;
    save_file(db_file, db)?;

    Ok(PathBuf::from(format!("{uuid}.csv")))
}

fn tail_event_process(
    mut db_file: ISUFile,
    path: PathBuf,
    db: &Connection,
) -> Result<PathBuf, ISUProcessorError> {
    let uuid = Uuid::new_v4();
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    reader.seek(SeekFrom::Start(db_file.last_position_read.try_into()?))?;

    let output_file = File::create(format!("{uuid}.csv"))?;
    let mut csv_writer = csv::WriterBuilder::new()
        .flexible(true)
        .from_writer(output_file);

    csv_writer.write_record(db_file.headers.split(','))?;

    let time = db_file.time.clone();
    let mut s = String::new();
    let mut i = db_file.last_index;
    while let Ok(u) = reader.read_line(&mut s) {
        if u == 0 {
            break;
        }

        s.pop();
        csv_writer.write_record([s.as_str(), format!("{i}").as_str(), time.clone().as_str()])?;
        s = String::new();
        i += 1;
    }

    csv_writer.flush()?;
    db_file.last_position_read = reader.stream_position()?.try_into()?;
    db_file.last_index = i;
    save_file(db_file, db)?;

    Ok(PathBuf::from(format!("{uuid}.csv")))
}
// fetch_file from sqlite db by path, error only on actual errors, not row not found
fn fetch_file(path: &str, db: &Connection) -> Result<Option<ISUFile>, ISUProcessorError> {
    let path = String::from(path);

    let result = db.query_row("SELECT * FROM isu WHERE path =?", [path],
    |row| Ok(ISUFile{
        path: row.get(0)?,
        last_position_read: row.get(1)?,
        headers: row.get(2)?,
        time: row.get(3)?,
        last_index: row.get(4)?,
    }));

    match result {
        Ok(r) => Ok(Some(r)),
        Err(e) => match e {
            rusqlite::Error::ExecuteReturnedResults => Ok(None),
            rusqlite::Error::QueryReturnedNoRows => Ok(None),
            _ => Err(e.into())
        }
    }
}

// write file to sqlite db
fn save_file(file: ISUFile, db: &Connection) -> Result<(), ISUProcessorError> {
   let mut stmt = db.prepare("INSERT INTO isu(path, last_position_read, headers, time, last_index) VALUES (?1,?2,?3,?4,?5)")?;
    stmt.execute([file.path, format!("{}", file.last_position_read), file.headers, file.time, format!("{}", file.last_index)])?;
        Ok(())

    /*// in order to use the Tokio runtime to do blocking operations on async functions we must
    // run it in a separate thread because we don't know what the caller is using
    let result = std::thread::spawn(move || {
        Runtime::new().unwrap().block_on(
            sqlx::query(
                "INSERT INTO isu(path, last_position_read, headers, time, last_index) VALUES (?,?,?,?,?)",
            )
            .bind(file.path)
            .bind(file.last_position_read)
            .bind(file.headers)
            .bind(file.time)
            .bind(file.last_index)
            .execute(&db),
        )
    })
    .join();

    match result {
        Ok(r) => match r {
            Ok(_) => Ok(()),
            Err(e) => Err(ISUProcessorError::SQLiteError(e)),
        },
        Err(_) => Err(ISUProcessorError::ThreadError),
    }*/
}

jester_core::export_plugin!(register);

extern "C" fn register(registrar: &mut dyn jester_core::PluginRegistrar) {
    registrar.register_function(Box::new(ISUProcessor::new().unwrap()));
}

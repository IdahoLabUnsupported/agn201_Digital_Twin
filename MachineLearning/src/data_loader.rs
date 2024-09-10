use crate::deep_lynx::{DeepLynxAPI, InitiateDataSourceDownloadQuery};
use crate::errors::MachineLearningError;
use crate::Configuration;
use chrono::{NaiveDateTime, Utc};
use duckdb::types::{TimeUnit, Value};
use duckdb::{AccessMode, Config, OptionalExt, Row};
use log::{error, info};
use serde::{Deserialize, Serialize};

use serde_yaml::from_reader;
use std::fs::File;
use std::marker::PhantomData;
use std::path::Path;
use std::sync::{Arc, RwLock};
use std::{fs, io};
use tokio::task::JoinSet;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct DataLoader {
    config: Configuration,
    // you might ask why we don't hold the duckdb connection open - that's because we really don't
    // want to hold a rw connection open while the python code runs, in case they want to use it for
    // something
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DataSourceConfiguration {
    table_name: String,
    container_id: u64,
    data_source_id: u64,
    timestamp_column_name: String,
    secondary_index: Option<String>,
    initial_timestamp: Option<String>,
    initial_index_start: Option<u64>,
}

struct InnerLock {
    _data: PhantomData<String>,
}

#[derive(Debug, Clone)]
pub struct Timestamps {
    pub experiment_start_time: String,
    pub temp_time: u64,
    pub eng_time: u64,
    pub ch3_time: u64,
}

impl DataLoader {
    pub fn new(config: Configuration) -> Self {
        DataLoader { config }
    }

    pub async fn load_data(&self) -> Result<(), MachineLearningError> {
        // open the duckdb connection outside the thread so we can indicate failure if needed
        let _db_path = Path::new(self.config.db_path.as_str());
        // we have to use a lock placeholder because duckdb can't be sent between threads safely, so we basically
        // use this lock to let us know when we can safely open a duckdb write connection
        let inner_lock = Arc::new(RwLock::new(InnerLock {
            _data: Default::default(),
        }));

        let mut set = JoinSet::new();
        let config = self.config.clone();
        for data_source in config.data_sources {
            let config = self.config.clone();
            let inner_lock = inner_lock.clone();
            set.spawn(async move {
                let mut client = match DeepLynxAPI::new(
                    config.deep_lynx_url.clone().clone(),
                    config.api_key.clone().clone(),
                    config.api_secret.clone().clone(),
                ) {
                    Ok(c) => c,
                    Err(e) => {
                        error!("error building api client {}", e);
                        return Err(MachineLearningError::Thread(format!(
                            "error building api client {}",
                            e
                        )));
                    }
                };
                // we could run this check just once on startup instead of checking each time, but this is more robust
                // and we have no idea what kind of SQL the other users might be running on it - changes how
                // we load data in
                let table_exists: Option<String> = match inner_lock.clone().write() {
                    Ok(_c) => {
                        let conn = duckdb::Connection::open_with_flags(
                            config.clone().db_path,
                            Config::default().access_mode(AccessMode::ReadWrite)?,
                        )?;

                        conn.query_row(
                            "SELECT table_name FROM duckdb_tables() WHERE table_name = ?",
                            [data_source.table_name.clone()],
                            |row| row.get(0),
                        )
                        .optional()
                        .unwrap_or(None)
                    }
                    Err(e) => {
                        error!("couldn't get lock on db connection {}", e);
                        return Ok(());
                    }
                };

                // if we don't have a table, treat this is as an initial fetch so the table gets created
                if table_exists.is_none() {
                    info!(
                        "table {} does not exist, running initial fetch",
                        data_source.table_name.clone()
                    );
                    DataLoader::initial_fetch_and_load(
                        config.clone(),
                        &data_source,
                        &mut client,
                        inner_lock.clone(),
                    )?;
                } else {
                    info!(
                        "table {} exists, running continual fetch",
                        data_source.table_name.clone()
                    );
                    DataLoader::continuous_fetch_and_load(
                        config.clone(),
                        &data_source,
                        &mut client,
                        inner_lock.clone(),
                    )?;

                    // run the data clean functionality
                    DataLoader::clean_data(config.clone(), &data_source, inner_lock.clone())?;
                }

                Ok(())
            });
        }

        while let Some(res) = set.join_next().await {
            info!("successfully executed data load for source {:?}", res);
        }

        Ok(())
    }

    // this is an extremely specific call for the ISU digital twin. It fetches the current timestamp
    // and indexes for certain tables and stores them in memory - this gets called _before_ we load
    // data so we have the starting point at which to let the notebooks run
    pub async fn current_timestamps(&mut self) -> Result<Timestamps, MachineLearningError> {
        // only a read connection initially, so we don't need to consult the lock
        let conn = duckdb::Connection::open_with_flags(
            self.config.db_path.clone(),
            Config::default().access_mode(AccessMode::ReadOnly)?,
        )?;

        struct Record {
            date_time: Option<NaiveDateTime>,
            time: u64,
            table: String,
        }

        let mut results: Vec<Record> = vec![];

        results.push(conn.query_row("SELECT DISTINCT date_time, time FROM temperature ORDER BY date_time DESC, time DESC LIMIT 1", [], |row: &Row| {
           Ok(Record {
               date_time: row.get(0)?,
               time: row.get(1)?,
               table: "temperature".to_string()
           })
        }).optional()?.unwrap_or(Record {date_time: None, time: 0, table: "temperature".to_string()}));

        results.push(conn.query_row("SELECT DISTINCT date_time, time FROM engineering_results ORDER BY date_time DESC, time DESC LIMIT 1", [], |row: &Row| {
            Ok(Record {
                date_time: row.get(0)?,
                time: row.get(1)?,
                table: "engineering_results".to_string()
            })
        }).optional()?.unwrap_or(Record {date_time: None, time: 0, table: "engineering_results".to_string()}));

        results.push(conn.query_row("SELECT DISTINCT date_time, time FROM ch3_engineering_data ORDER BY date_time DESC, time DESC LIMIT 1", [], |row: &Row| {
            Ok(Record {
                date_time: row.get(0)?,
                time: row.get(1)?,
                table: "ch3_engineering_data".to_string()
            })
        }).optional()?.unwrap_or(Record {date_time: None, time: 0, table: "ch3_engineering_data".to_string()}));

        // now we sort by time to pick the earliest timestamp
        results.sort_by(|a, b| a.date_time.cmp(&b.date_time));

        let mut timestamps = Timestamps {
            experiment_start_time: "".to_string(),
            temp_time: 0,
            eng_time: 0,
            ch3_time: 0,
        };

        // set to the earliest timestamp
        timestamps.experiment_start_time = match results.iter().find_map(|r| r.date_time) {
            None => Utc::now()
                .naive_utc()
                .format("%Y-%m-%d %H:%M:%S")
                .to_string(),
            Some(t) => t.format("%Y-%m-%d %H:%M:%S").to_string(),
        };

        results.iter().for_each(|r| match r.table.as_str() {
            "temperature" => timestamps.temp_time = r.time,
            "engineering_results" => timestamps.eng_time = r.time,
            "ch3_engineering_data" => timestamps.ch3_time = r.time,
            _ => {}
        });

        Ok(timestamps)
    }

    pub async fn current_date_time(&mut self) -> Result<String, MachineLearningError> {
        // only a read connection initially, so we don't need to consult the lock
        let conn = duckdb::Connection::open_with_flags(
            self.config.db_path.clone(),
            Config::default().access_mode(AccessMode::ReadOnly)?,
        )?;

        struct Record {
            date_time: Option<NaiveDateTime>,
            table: String,
        }

        let mut results: Vec<Record> = vec![];

        results.push(
            conn.query_row(
                "SELECT DISTINCT date_time, time FROM temperature ORDER BY date_time DESC LIMIT 1",
                [],
                |row: &Row| {
                    Ok(Record {
                        date_time: row.get(0)?,
                        table: "temperature".to_string(),
                    })
                },
            )
            .optional()?
            .unwrap_or(Record {
                date_time: None,
                table: "temperature".to_string(),
            }),
        );

        results.push(conn.query_row("SELECT DISTINCT date_time, time FROM engineering_results ORDER BY date_time DESC LIMIT 1", [], |row: &Row| {
            Ok(Record {
                date_time: row.get(0)?,
                table: "engineering_results".to_string()
            })
        }).optional()?.unwrap_or(Record {date_time: None,  table: "engineering_results".to_string()}));

        results.push(conn.query_row("SELECT DISTINCT date_time, time FROM ch3_engineering_data ORDER BY date_time DESC LIMIT 1", [], |row: &Row| {
            Ok(Record {
                date_time: row.get(0)?,
                table: "ch3_engineering_data".to_string()
            })
        }).optional()?.unwrap_or(Record {date_time: None,  table: "ch3_engineering_data".to_string()}));

        // now we sort by time to pick the earliest timestamp
        results.sort_by(|a, b| a.date_time.cmp(&b.date_time));

        // set to the earliest timestamp
        match results.iter().find_map(|r| r.date_time) {
            None => Ok(Utc::now()
                .naive_utc()
                .format("%Y-%m-%d %H:%M:%S")
                .to_string()),
            Some(t) => Ok(t.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }

    fn clean_data(
        config: Configuration,
        data_source: &DataSourceConfiguration,
        lock: Arc<RwLock<InnerLock>>,
    ) -> Result<(), MachineLearningError> {
        match lock.write() {
            Ok(_) => {
                let conn = duckdb::Connection::open_with_flags(
                    config.db_path.clone(),
                    Config::default().access_mode(AccessMode::ReadWrite)?,
                )?;

                conn.execute(
                    format!(
                        "DELETE FROM {} WHERE {} < NOW() - '{} day'::INTERVAL",
                        data_source.table_name,
                        data_source.timestamp_column_name,
                        config.data_retention_days
                    )
                    .as_str(),
                    [],
                )?;

                Ok(())
            }
            Err(_) => Err(MachineLearningError::Database),
        }
    }

    // if data or table already exists for a data source, then we fetch continuously
    fn continuous_fetch_and_load(
        config: Configuration,
        data_source: &DataSourceConfiguration,
        client: &mut DeepLynxAPI,
        lock: Arc<RwLock<InnerLock>>,
    ) -> Result<(), MachineLearningError> {
        // only a read connection initially, so we don't need to consult the lock
        let conn = duckdb::Connection::open_with_flags(
            config.db_path.clone(),
            Config::default().access_mode(AccessMode::ReadOnly)?,
        )?;

        // we need to fetch the last record in the table, but the sort isn't guaranteed so we'll do that
        // manually
        let mut check_query = format!(
            "SELECT {} FROM {} ORDER BY {} DESC LIMIT 1",
            data_source.timestamp_column_name,
            data_source.table_name,
            data_source.timestamp_column_name
        );

        // sort by secondary index as well if it exists, get the latest value
        if data_source.secondary_index.is_some() {
            let secondary_index = data_source
                .secondary_index
                .clone()
                .ok_or(MachineLearningError::UnwrapOption)?;

            check_query = format!(
                "SELECT {},{} FROM {} ORDER BY {} DESC,{} DESC LIMIT 1",
                data_source.timestamp_column_name,
                secondary_index,
                data_source.table_name,
                data_source.timestamp_column_name,
                secondary_index
            );
        }

        // simple struct representing the record from the DB
        struct Record {
            timestamp_or_index: duckdb::types::Value, // the api expects a string even if it's an index number
            secondary_index: Option<u64>,
        }

        // pull the last record if it exists
        let last_record = conn
            .query_row(check_query.as_str(), [], |row: &Row| {
                let mut secondary_index: Option<u64> = None;
                if data_source.secondary_index.is_some() {
                    secondary_index = Some(row.get(1)?);
                }

                Ok(Record {
                    timestamp_or_index: row.get(0)?,
                    secondary_index,
                })
            })
            .optional()?;

        // if we don't have a last record, we need to drop the table and run initial fetch and load again
        if last_record.is_none() {
            return DataLoader::initial_fetch_and_load(config, data_source, client, lock);
        }

        let last_record = last_record.ok_or(MachineLearningError::UnwrapOption)?;

        // because we need to handle either an index or timestamp we have to match through duckdb's type
        // and convert to what we need - super fun!
        let start_time: Option<String> = duck_time_to_string(last_record.timestamp_or_index)?;

        // first we fetch the file pointer for the download, this way we can check filesize against disk
        // passing in the elements provided by the user, if none provided will default to returning
        // the full table currently
        let file_pointer = client.initiate_data_source_download(
            data_source.container_id,
            data_source.data_source_id,
            InitiateDataSourceDownloadQuery {
                start_time,
                end_time: None, // deeplynx defaults to latest timestamp if no endpoint is provided
                secondary_index_name: data_source.secondary_index.clone(),
                secondary_index_start_value: match last_record.secondary_index {
                    None => Some(0),
                    Some(i) => Some(i),
                },
            },
        )?;

        // TODO: check file against disk size prior to downloading

        let mut file_stream =
            client.download_file(data_source.container_id, file_pointer.id.parse()?, true)?;

        // copy the file stream from the download to a temporary file
        let uuid = Uuid::new_v4();
        let mut file = File::create(format!("{uuid}.csv"))?;
        io::copy(&mut file_stream, &mut file)?;

        match conn.close() {
            Ok(_) => {
                match lock.write() {
                    Ok(_) => {
                        let conn = duckdb::Connection::open_with_flags(
                            config.db_path.clone(),
                            Config::default().access_mode(AccessMode::ReadWrite)?,
                        )?;

                        // create a table from that .csv and load it in duckdb
                        let inserted = conn.execute(
                            format!(
                                "COPY {} FROM '{uuid}.csv' (HEADER TRUE)",
                                data_source.table_name
                            )
                            .as_str(),
                            [],
                        )?;

                        if inserted == 0 {
                            info!(
            "no data inserted for data source {} on continuous fetch, continuing loop",
            data_source.data_source_id
        );
                        }
                    }
                    Err(e) => {
                        let error = format!("unable to lock for duckdb connection {}", e);
                        error!("{}", error);
                        return Err(MachineLearningError::Thread(error));
                    }
                }
            }
            Err(e) => {
                let error = format!("unable to close duckdb connection {}", e.1);
                error!("{}", error);
                return Err(MachineLearningError::Thread(error));
            }
        }

        fs::remove_file(format!("{uuid}.csv"))?;
        Ok(())
    }

    // the first loading call for a data source, ensures a table is created if data exists - this is a
    // DESTRUCTIVE operation as it will first wipe the table if it exists to insure that the table matches
    // the latest data from the source
    fn initial_fetch_and_load(
        config: Configuration,
        data_source: &DataSourceConfiguration,
        client: &mut DeepLynxAPI,
        lock: Arc<RwLock<InnerLock>>,
    ) -> Result<(), MachineLearningError> {
        match lock.write() {
            Ok(_) => {
                let conn = duckdb::Connection::open_with_flags(
                    config.db_path.clone(),
                    Config::default().access_mode(AccessMode::ReadWrite)?,
                )?;

                // first drop the table if it exists so that we can guarantee its the right structure
                conn.execute(
                    format!("DROP TABLE IF EXISTS {}", data_source.table_name).as_str(),
                    [],
                )?;
            }
            Err(e) => {
                let error = format!("unable to lock for duckdb connection {}", e);
                error!("{}", error);
                return Err(MachineLearningError::Thread(error));
            }
        }

        // first we fetch the file pointer for the download, this way we can check filesize against disk
        // passing in the elements provided by the user, if none provided will default to returning
        // the full table currently
        let file_pointer = client.initiate_data_source_download(
            data_source.container_id,
            data_source.data_source_id,
            InitiateDataSourceDownloadQuery {
                start_time: data_source.initial_timestamp.clone(),
                end_time: None, // deeplynx defaults to latest timestamp if no endpoint is provided
                secondary_index_name: data_source.secondary_index.clone(),
                secondary_index_start_value: Some(0),
            },
        )?;

        // TODO: check file against disk size prior to downloading

        let mut file_stream =
            client.download_file(data_source.container_id, file_pointer.id.parse()?, true)?;

        // copy the file stream from the download to a temporary file
        let uuid = Uuid::new_v4();
        let mut file = File::create(format!("{uuid}.csv"))?;
        io::copy(&mut file_stream, &mut file)?;

        match lock.write() {
            Ok(_) => {
                let conn = duckdb::Connection::open_with_flags(
                    config.db_path.clone(),
                    Config::default().access_mode(AccessMode::ReadWrite)?,
                )?;

                // create a table from that .csv and load it in duckdb
                let inserted = conn
                    .execute(
                        format!(
                            "CREATE TABLE {} AS SELECT * FROM '{uuid}.csv'",
                            data_source.table_name
                        )
                        .as_str(),
                        [],
                    )
                    .unwrap();

                // if no rows are inserted we need to remove the table as the inference of the data types might
                // be incorrect, when the process loops again it will attempt to create the table again if it
                // doesn't already exist - don't error out because lack of data doesn't constitute an error state
                // but do log it
                if inserted == 0 {
                    info!(
                        "no data fetched for data source {}, dropping temporary table",
                        data_source.data_source_id
                    );
                    conn.execute(
                        format!("DROP TABLE IF EXISTS {}", data_source.table_name).as_str(),
                        [],
                    )?;
                }
                fs::remove_file(format!("{uuid}.csv"))?;
                Ok(())
            }
            Err(e) => {
                let error = format!("unable to lock for duckdb connection {}", e);
                error!("{}", error);

                fs::remove_file(format!("{uuid}.csv"))?;
                return Err(MachineLearningError::Thread(error));
            }
        }
    }

    #[cfg(test)]
    fn initial_process_functionality() -> Result<(), MachineLearningError> {
        // double check we're starting from scratch - ignore possible not existing error
        let _ = fs::remove_file("./test.db");
        let config_file = File::open("./.config.yml")?;
        let config: Configuration = from_reader(config_file)?;

        let conn = duckdb::Connection::open_with_flags(
            config.db_path.clone(),
            Config::default().access_mode(AccessMode::ReadWrite)?,
        )?;

        let mut api = DeepLynxAPI::new(
            config.deep_lynx_url.clone(),
            config.api_key.clone(),
            config.api_secret.clone(),
        )?;

        let inner_lock = InnerLock {
            _data: Default::default(),
        };

        let lock = Arc::new(RwLock::new(inner_lock));

        DataLoader::initial_fetch_and_load(
            config.clone(),
            &config.data_sources[0],
            &mut api,
            lock,
        )?;

        // checking if the table now exists should be enough
        let table_exists: Option<String> = conn
            .query_row(
                "SELECT table_name FROM duckdb_tables() WHERE table_name = ?",
                [config.data_sources[0].table_name.clone()],
                |row| row.get(0),
            )
            .optional()?;

        assert!(table_exists.is_some());
        fs::remove_file("./test.db")?;
        Ok(())
    }

    #[cfg(test)]
    fn continuous_process_functionality() -> Result<(), MachineLearningError> {
        // double check we're starting from scratch - ignore possible not existing error
        let _ = fs::remove_file("./test.db");
        let config_file = File::open("./.config.yml")?;
        let config: Configuration = from_reader(config_file)?;

        let conn = duckdb::Connection::open_with_flags(
            config.db_path.clone(),
            Config::default().access_mode(AccessMode::ReadWrite)?,
        )?;

        let inner_lock = InnerLock {
            _data: Default::default(),
        };

        let mut api = DeepLynxAPI::new(
            config.deep_lynx_url.clone(),
            config.api_key.clone(),
            config.api_secret.clone(),
        )?;

        let lock = Arc::new(RwLock::new(inner_lock));

        DataLoader::initial_fetch_and_load(
            config.clone(),
            &config.data_sources[0],
            &mut api,
            lock.clone(),
        )?;

        // checking if the table now exists should be enough
        let table_exists: Option<String> = conn
            .query_row(
                "SELECT table_name FROM duckdb_tables() WHERE table_name = ?",
                [config.data_sources[0].table_name.clone()],
                |row| row.get(0),
            )
            .optional()?;

        assert!(table_exists.is_some());

        // now lets run the continuous test - because we can't load DL with more data this test
        // should not be completely trusted, but we're getting as close as we can without mocking
        // a crap ton of data
        DataLoader::continuous_fetch_and_load(
            config.clone(),
            &config.data_sources[0],
            &mut api,
            lock,
        )?;

        fs::remove_file("./test.db")?;
        Ok(())
    }
}

fn duck_time_to_string(val: duckdb::types::Value) -> Result<Option<String>, MachineLearningError> {
    let val = match val {
        Value::Null => None,
        Value::Boolean(b) => Some(b.to_string()),
        Value::TinyInt(t) => Some(t.to_string()),
        Value::SmallInt(s) => Some(s.to_string()),
        Value::Int(i) => Some(i.to_string()),
        Value::BigInt(b) => Some(b.to_string()),
        Value::HugeInt(h) => Some(h.to_string()),
        Value::UTinyInt(u) => Some(u.to_string()),
        Value::USmallInt(s) => Some(s.to_string()),
        Value::UInt(u) => Some(u.to_string()),
        Value::UBigInt(b) => Some(b.to_string()),
        Value::Float(f) => Some(f.to_string()),
        Value::Double(d) => Some(d.to_string()),
        Value::Decimal(d) => Some(d.to_string()),
        Value::Timestamp(unit, v) => match unit {
            TimeUnit::Second => Some(
                NaiveDateTime::from_timestamp_opt(v, 0)
                    .ok_or(MachineLearningError::Database)?
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string(),
            ),
            TimeUnit::Millisecond => Some(
                NaiveDateTime::from_timestamp_millis(v)
                    .ok_or(MachineLearningError::Database)?
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string(),
            ),

            TimeUnit::Microsecond => Some(
                NaiveDateTime::from_timestamp_micros(v)
                    .ok_or(MachineLearningError::Database)?
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string(),
            ),
            TimeUnit::Nanosecond => None, // we can't parse out nanos - thankfully they shouldn't come this way
        },
        Value::Text(s) => Some(s),
        Value::Blob(v) => Some(std::str::from_utf8(v.as_slice())?.to_string()),
        Value::Date32(d) => Some(d.to_string()),
        Value::Time64(unit, v) => match unit {
            TimeUnit::Second => Some(
                NaiveDateTime::from_timestamp_opt(v, 0)
                    .ok_or(MachineLearningError::Database)?
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string(),
            ),
            TimeUnit::Millisecond => Some(
                NaiveDateTime::from_timestamp_millis(v)
                    .ok_or(MachineLearningError::Database)?
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string(),
            ),

            TimeUnit::Microsecond => Some(
                NaiveDateTime::from_timestamp_micros(v)
                    .ok_or(MachineLearningError::Database)?
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string(),
            ),
            TimeUnit::Nanosecond => None, // we can't parse out nanos - thankfully they shouldn't come this way
        },
    };

    Ok(val)
}

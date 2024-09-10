pub mod data_loader;
mod deep_lynx;
mod errors;
mod notebook;

use crate::data_loader::{DataLoader, DataSourceConfiguration, Timestamps};
use crate::errors::MachineLearningError;
use crate::notebook::run_notebook;
use clap::Parser;
use log::{error, info};
use serde::{Deserialize, Serialize};
use serde_yaml::from_reader;
use std::fs::File;
use std::path::PathBuf;
use tokio::join;
use tokio::time::{sleep, Duration};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Arguments {
    #[clap(short, long, value_parser, value_name = "FILE")]
    config_file: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Configuration {
    api_key: Option<String>,
    api_secret: Option<String>,
    deep_lynx_url: String,
    db_path: String,
    period_interval: u64,
    data_retention_days: u32,
    target_data_source_id: Option<u64>,
    linear_notebook_path: Option<String>,
    anomaly_notebook_path: Option<String>,
    neutronics_notebook_path: Option<String>,
    target_container_id: Option<u64>,
    debug: Option<bool>,
    data_sources: Vec<DataSourceConfiguration>,
}

#[tokio::main]
async fn main() -> Result<(), MachineLearningError> {
    let cli: Arguments = Arguments::parse();

    let config_file_path = match cli.config_file {
        None => {
            let mut p = PathBuf::new();
            p.push(".config.yml");

            p
        }
        Some(p) => p,
    };

    let config_file = File::open(config_file_path.clone())?;
    let config: Configuration = from_reader(config_file)?;

    let mut log_level = log::LevelFilter::Info;

    if config.debug.is_some() {
        log_level = log::LevelFilter::Debug
    }

    // set the log to output to file and stdout
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log_level)
        .chain(std::io::stdout())
        .chain(fern::log_file("deeplynx_loader.log")?)
        .apply()?;

    let mut data_loader = DataLoader::new(config.clone());
    let mut previous_timestamps: Option<Timestamps> = None;

    loop {
        // timestamps first so we have the starting position we should run the notebooks on
        let timestamps = match data_loader.current_timestamps().await {
            Ok(mut t) => {
                // check the previous timestamp so we can reset indexes to zero if it's a new start
                match &previous_timestamps {
                    None => {
                        t.ch3_time = 0;
                        t.eng_time = 0;
                        t.temp_time = 0;
                        Some(t)
                    }
                    Some(p) => {
                        if p.experiment_start_time.as_str() != t.experiment_start_time.as_str() {
                            t.ch3_time = 0;
                            t.eng_time = 0;
                            t.temp_time = 0;
                        }

                        Some(t)
                    }
                }
            }
            // if we've errored out it's usually because we've not fetched the data yet
            Err(_) => None,
        };

        // run the fetch of each data source
        data_loader.load_data().await?;

        // if we didn't get timestamps earlier, fetch a modified version that defaults index to zero
        // because it most likely indicated we started from scratch so we should have tables now
        let timestamps = match timestamps {
            None => Timestamps {
                experiment_start_time: data_loader.current_date_time().await?,
                temp_time: 0,
                eng_time: 0,
                ch3_time: 0,
            },
            Some(t) => t,
        };

        previous_timestamps = Some(timestamps.clone());
        // The separate notebooks we ran.
        info!("running notebooks with timestamps {:?}", timestamps);
        // now that we've run the data fetching, run each notebook in its own thread and then wait
        // for them to finish so we can get a result out of them
        let linear_path = config.linear_notebook_path.clone();
        let file_path = config_file_path.clone();
        let inner_timestamps = timestamps.clone();
        let linear = tokio::spawn(async move {
            run_notebook(
                linear_path.unwrap_or("./python/notebooks/REPLACE ME.ipynb".to_string()),
                file_path.to_str(),
                inner_timestamps,
            )
            .await
        });

        let neutronics_path = config.neutronics_notebook_path.clone();
        let file_path = config_file_path.clone();
        let inner_timestamps = timestamps.clone();
        let neutronics = tokio::spawn(async move {
            run_notebook(
                neutronics_path
                    .unwrap_or("./python/notebooks/REPLACE ME.ipynb".to_string()),
                file_path.clone().to_str(),
                inner_timestamps,
            )
            .await
        });

        let anomaly_path = config.anomaly_notebook_path.clone();
        let file_path = config_file_path.clone();
        let inner_timestamps = timestamps.clone();
        let anomaly = tokio::spawn(async move {
            run_notebook(
                anomaly_path.unwrap_or("./python/notebooks/REPLACE ME.ipynb".to_string()),
                file_path.clone().to_str(),
                inner_timestamps,
            )
            .await
        });

        let (linear, neutronics, anomaly) = join!(linear, neutronics, anomaly);

        match linear? {
            Ok(status) => {
                if !status.success() {
                    error!("linear prediction notebook exited abnormally")
                }
            }
            Err(e) => {
                error!("linear prediction notebook exited with error {}", e)
            }
        }

        match neutronics? {
            Ok(status) => {
                if !status.success() {
                    error!("neutronics prediction notebook exited abnormally")
                }
            }
            Err(e) => {
                error!("neutronics prediction notebook exited with error {}", e)
            }
        }

        match anomaly? {
            Ok(status) => {
                if !status.success() {
                    error!("anomaly prediction notebook exited abnormally")
                }
            }
            Err(e) => {
                error!("anomaly prediction notebook exited with error {}", e)
            }
        }

        sleep(Duration::from_secs(config.period_interval)).await;
    }
}

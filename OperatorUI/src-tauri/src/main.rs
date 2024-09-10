// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod deep_lynx;
mod deeplynx_loader;
mod errors;

use crate::deeplynx_loader::{
    FinalizedData, FinalizedDataMap, FormattedPredictionData, Loader, RawPredictionData,
};
use crate::errors::LoaderError;
use std::env;
use std::time::Duration;

struct LoaderState(Loader);

#[tauri::command]
async fn manual_load(
    loader: tauri::State<'_, LoaderState>,
) -> Result<(), String> {
    loader.0.load_data().map_err(|err| err.to_string())?;
    Ok(())
}

#[tauri::command]
fn fetch_data(
    loader: tauri::State<LoaderState>,
    last_datetime: String,
    last_time: f32,
    limit: Option<i32>
) -> Result<FinalizedDataMap, String> {
    let data: Result<FinalizedDataMap, LoaderError> = loader.0.fetch_data(last_datetime, last_time, limit);

    if !data.is_ok() {
        data.map_err(|err| err.to_string())?;
        return Err("unable to complete".to_string());
    }

    Ok(data.unwrap())
}


#[tauri::command]
fn fetch_run_dates(
    loader: tauri::State<LoaderState>,
) -> Result<Vec<String>, String> {
    let data: Result<Vec<String>, LoaderError> = loader.0.fetch_run_dates();

    if !data.is_ok() {
        data.map_err(|err| err.to_string())?;
        return Err("unable to complete".to_string());
    }

    Ok(data.unwrap())
}

fn main() {
    let path = env::var("AGN_CONFIG_PATH").unwrap_or(".config.yml".to_string());

    let loader = match Loader::new(path.as_str(), true) {
        Ok(l) => l,
        Err(e) => {
            panic!("Cannot load configuration file: {}", e)
        }
    };

    tauri::Builder::default()
        .manage(LoaderState(loader))
        .invoke_handler(tauri::generate_handler![manual_load, fetch_data, fetch_run_dates])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

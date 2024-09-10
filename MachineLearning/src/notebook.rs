use crate::data_loader::Timestamps;
use crate::errors::MachineLearningError;
use std::process::ExitStatus;
use tokio::process::Command;

pub async fn run_notebook(
    path: String,
    config_path: Option<&str>,
    args: Timestamps,
) -> Result<ExitStatus, MachineLearningError> {
    let mut command = Command::new("papermill")
        .args([
            "-p",
            "config_file_path",
            format!("{}", config_path.unwrap_or("./.config.yml")).as_str(),
        ])
        .args([
            "-p",
            "experiment_start_time",
            format!("{}", args.experiment_start_time).as_str(),
        ])
        .args(["-p", "temp_time", format!("{}", args.temp_time).as_str()])
        .args(["-p", "eng_time", format!("{}", args.eng_time).as_str()])
        .args(["-p", "ch3_time", format!("{}", args.ch3_time).as_str()])
        .args(["-k", "python3"])
        .args(["--stdout-file", "out.txt"])
        .args(["--stderr-file", "out_err.txt"])
        .arg(path.clone())
        .arg("/dev/null")
        .spawn()?;

    Ok(command.wait().await?)
}

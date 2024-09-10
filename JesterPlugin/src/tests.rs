use jester_core::errors::ProcessorError;
/*
TEST DATA HAS BEEN REMOVED FOR OPEN SOURCING - CONTACT US IF YOU NEED TO TEST
 */

#[cfg(test)]
mod general_tests {
    use crate::{fetch_file, save_file, ISUFile, ISUProcessor};
    use adler::adler32;
    use jester_core::{DataSourceMessage, Processor};
    use sqlx::sqlite::SqliteConnectOptions;
    use sqlx::SqlitePool;
    use std::fs::{File, OpenOptions};
    use std::io::{BufReader, Write};
    use std::path::{Path, PathBuf};
    use std::str::FromStr;
    use tokio::fs;
    use tokio::sync::mpsc::unbounded_channel;

    #[tokio::test]
    async fn init_test() {
        fs::remove_file(".test.db").await.unwrap();

        let isu = match ISUProcessor::new() {
            Ok(p) => p,
            Err(e) => {
                panic!("init_test failed {e:?}")
            }
        };

        let options = match SqliteConnectOptions::from_str("sqlite://.test.db") {
            Ok(o) => o,
            Err(e) => {
                panic!("unable to create options for sqlite connection {e:?}",)
            }
        };

        let db = match SqlitePool::connect_with(options.create_if_missing(true)).await {
            Ok(d) => d,
            Err(e) => {
                panic!("unable to connect to sqlite database {e:?}",)
            }
        };

        let result = isu.init(db.clone());
        assert!(result.is_ok(), "{:?}", result.err());

        let mut row =
            sqlx::query("SELECT name FROM sqlite_master WHERE type='table' AND name='isu'")
                .fetch_one(&db)
                .await;
        // will be Ok if it returns a single row, which corresponds to the table existing
        assert!(row.is_ok())
    }

    #[tokio::test]
    async fn process_engineering_data_test() {
        fs::remove_file(".test.db").await;
        let (ts_tx, mut ts_rx) = unbounded_channel();
        let (g_tx, mut g_rx) = unbounded_channel();

        let options = match SqliteConnectOptions::from_str("sqlite://.test.db") {
            Ok(o) => o,
            Err(e) => {
                panic!("unable to create options for sqlite connection {e:?}")
            }
        };

        let db = match SqlitePool::connect_with(options.create_if_missing(true)).await {
            Ok(d) => d,
            Err(e) => {
                panic!("unable to connect to sqlite database {e:?}")
            }
        };

        let isu = match ISUProcessor::new() {
            Ok(p) => p,
            Err(e) => {
                panic!("init_test failed {e:?}")
            }
        };

        let result = isu.init(db.clone());
        assert!(result.is_ok(), "{:?}", result.err());

        std::fs::copy(
            "./test_data/Feb_13_2023_14_29/Most Engineering Data.txt",
            "./test_data/Feb_13_2023_14_29/tail.txt",
        )
        .unwrap();

        let result = isu.process(
            PathBuf::from_str("./test_data/Feb_13_2023_14_29/tail.txt").unwrap(),
            db.clone(),
            Some(ts_tx.clone()),
            Some(g_tx.clone()),
        );

        assert!(result.is_ok(), "Error during process call: {result:?}");

        // we should see a total of one message on the timeseries channel, should be a file
        let mut message = ts_rx.recv().await.unwrap();
        let generated = match message {
            DataSourceMessage::File(f) => {
                assert!(f.to_str().is_some());
                f
            }
            DataSourceMessage::Data(_) => {
                panic!("wrong message type received")
            }
            DataSourceMessage::Close => {
                panic!("wrong message type received")
            }
        };

        let generated_file = File::open(&generated).unwrap();
        let generated_checksum = adler32(BufReader::new(generated_file)).unwrap();

        let comparison = File::open(Path::new("./test_data/EngineeringResults.csv")).unwrap();
        let comparison_checksum = adler32(BufReader::new(comparison)).unwrap();

        assert_eq!(generated_checksum, comparison_checksum);
        fs::remove_file(generated).await.unwrap();

        // now we're testing the tailing
        let file = OpenOptions::new()
            .append(true)
            .open("./test_data/Feb_13_2023_14_29/tail.txt")
            .unwrap();

        // write a new record
        let mut csv_writer = csv::WriterBuilder::new().flexible(true).from_writer(file);
        csv_writer
            .write_record([
                "",
                "0.000000",
                "7123.633812",
                "9.472421E-10",
                "0.054466",
                "0.000000",
                "0.000000",
                "0.025790",
                "24.506584",
                "18.462020",
            ])
            .unwrap();

        csv_writer.flush().unwrap();

        let result = isu.process(
            PathBuf::from_str("./test_data/Feb_13_2023_14_29/tail.txt").unwrap(),
            db,
            Some(ts_tx),
            Some(g_tx),
        );

        assert!(result.is_ok(), "Error during process call: {result:?}");

        // we should see a total of one message on the timeseries channel, should be a file
        message = ts_rx.recv().await.unwrap();
        let generated = match message {
            DataSourceMessage::File(f) => {
                assert!(f.to_str().is_some());
                f
            }
            DataSourceMessage::Data(_) => {
                panic!("wrong message type received")
            }
            DataSourceMessage::Close => {
                panic!("wrong message type received")
            }
        };

        let generated_file = File::open(&generated).unwrap();
        let generated_checksum = adler32(BufReader::new(generated_file)).unwrap();

        let comparison = File::open(Path::new("./test_data/EngineeringResultsTail.csv")).unwrap();
        let comparison_checksum = adler32(BufReader::new(comparison)).unwrap();

        assert_eq!(generated_checksum, comparison_checksum);
        fs::remove_file(generated).await.unwrap();
    }

    #[tokio::test]
    async fn save_file_test() {
        fs::remove_file(".test.db").await.unwrap();
        let options = match SqliteConnectOptions::from_str("sqlite://.test.db") {
            Ok(o) => o,
            Err(e) => {
                panic!("unable to create options for sqlite connection {e:?}")
            }
        };

        let db = match SqlitePool::connect_with(options.create_if_missing(true)).await {
            Ok(d) => d,
            Err(e) => {
                panic!("unable to connect to sqlite database {e:?}")
            }
        };

        let isu = match ISUProcessor::new() {
            Ok(p) => p,
            Err(e) => {
                panic!("init_test failed {e:?}")
            }
        };

        let result = isu.init(db.clone());
        assert!(result.is_ok(), "{:?}", result.err());

        let result = save_file(
            ISUFile {
                path: String::from("test"),
                last_position_read: 0,
                last_index: 0,
                headers: String::from("header1, header2"),
                time: String::from(""),
            },
            db.clone(),
        );
        assert!(result.is_ok(), "{:?}", result.err());

        // saving the same path should result in an ok as well, as it will upsert the existing record
        let result = save_file(
            ISUFile {
                path: String::from("test"),
                last_position_read: 0,
                last_index: 0,
                headers: String::from("header1, header2"),
                time: String::from(""),
            },
            db,
        );
        assert!(result.is_ok(), "{:?}", result.err());
    }

    #[tokio::test]
    async fn fetch_file_test() {
        fs::remove_file(".test.db").await.unwrap();
        let options = match SqliteConnectOptions::from_str("sqlite://.test.db") {
            Ok(o) => o,
            Err(e) => {
                panic!("unable to create options for sqlite connection {e:?}")
            }
        };

        let db = match SqlitePool::connect_with(options.create_if_missing(true)).await {
            Ok(d) => d,
            Err(e) => {
                panic!("unable to connect to sqlite database {e:?}")
            }
        };

        let isu = match ISUProcessor::new() {
            Ok(p) => p,
            Err(e) => {
                panic!("init_test failed {e:?}")
            }
        };

        let result = isu.init(db.clone());
        assert!(result.is_ok(), "{:?}", result.err());

        let result = save_file(
            ISUFile {
                path: String::from("test"),
                last_position_read: 0,
                last_index: 0,
                headers: String::from("header1,header2"),
                time: String::from(""),
            },
            db.clone(),
        );
        assert!(result.is_ok(), "{:?}", result.err());

        let result = fetch_file("test", db.clone());
        assert!(result.is_ok(), "{:?}", result.err());
        let result = result.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().headers, String::from("header1,header2"));

        let result = save_file(
            ISUFile {
                path: String::from("test"),
                last_position_read: 0,
                last_index: 0,
                headers: String::from("header3,header4"),
                time: String::from(""),
            },
            db.clone(),
        );
        assert!(result.is_ok(), "{:?}", result.err());

        let result = fetch_file("test", db);
        assert!(result.is_ok(), "{:?}", result.err());
        let result = result.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().headers, String::from("header3,header4"))
    }

    #[tokio::test]
    async fn process_ch3_data_test() {
        fs::remove_file(".test.db").await;
        let (ts_tx, mut ts_rx) = unbounded_channel();
        let (g_tx, mut g_rx) = unbounded_channel();

        let options = match SqliteConnectOptions::from_str("sqlite://.test.db") {
            Ok(o) => o,
            Err(e) => {
                panic!("unable to create options for sqlite connection {e:?}")
            }
        };

        let db = match SqlitePool::connect_with(options.create_if_missing(true)).await {
            Ok(d) => d,
            Err(e) => {
                panic!("unable to connect to sqlite database {e:?}")
            }
        };

        let isu = match ISUProcessor::new() {
            Ok(p) => p,
            Err(e) => {
                panic!("init_test failed {e:?}")
            }
        };

        let result = isu.init(db.clone());
        assert!(result.is_ok(), "{:?}", result.err());

        let result = isu.process(
            PathBuf::from_str("./test_data/Feb_13_2023_14_29/Ch 3 Engineering Data.txt").unwrap(),
            db.clone(),
            Some(ts_tx.clone()),
            Some(g_tx.clone()),
        );

        assert!(result.is_ok(), "Error during process call: {result:?}");

        // we should see a total of one message on the timeseries channel, should be a file
        let mut message = ts_rx.recv().await.unwrap();
        let generated = match message {
            DataSourceMessage::File(f) => {
                assert!(f.to_str().is_some());
                f
            }
            DataSourceMessage::Data(_) => {
                panic!("wrong message type received")
            }
            DataSourceMessage::Close => {
                panic!("wrong message type received")
            }
        };

        let generated_file = File::open(&generated).unwrap();
        let generated_checksum = adler32(BufReader::new(generated_file)).unwrap();

        let comparison = File::open(Path::new("./test_data/Ch3EngineeringResults.csv")).unwrap();
        let comparison_checksum = adler32(BufReader::new(comparison)).unwrap();

        assert_eq!(generated_checksum, comparison_checksum);
        fs::remove_file(generated).await.unwrap();
    }

    #[tokio::test]
    async fn process_digitals_data_test() {
        fs::remove_file(".test.db").await;
        let (ts_tx, mut ts_rx) = unbounded_channel();
        let (g_tx, mut g_rx) = unbounded_channel();

        let options = match SqliteConnectOptions::from_str("sqlite://.test.db") {
            Ok(o) => o,
            Err(e) => {
                panic!("unable to create options for sqlite connection {e:?}")
            }
        };

        let db = match SqlitePool::connect_with(options.create_if_missing(true)).await {
            Ok(d) => d,
            Err(e) => {
                panic!("unable to connect to sqlite database {e:?}")
            }
        };

        let isu = match ISUProcessor::new() {
            Ok(p) => p,
            Err(e) => {
                panic!("init_test failed {e:?}")
            }
        };

        let result = isu.init(db.clone());
        assert!(result.is_ok(), "{:?}", result.err());

        let result = isu.process(
            PathBuf::from_str("./test_data/Feb_13_2023_14_29/Digitals.txt").unwrap(),
            db.clone(),
            Some(ts_tx.clone()),
            Some(g_tx.clone()),
        );

        assert!(result.is_ok(), "Error during process call: {result:?}");

        // we should see a total of one message on the timeseries channel, should be a file
        let mut message = ts_rx.recv().await.unwrap();
        let generated = match message {
            DataSourceMessage::File(f) => {
                assert!(f.to_str().is_some());
                f
            }
            DataSourceMessage::Data(_) => {
                panic!("wrong message type received")
            }
            DataSourceMessage::Close => {
                panic!("wrong message type received")
            }
        };

        let generated_file = File::open(&generated).unwrap();
        let generated_checksum = adler32(BufReader::new(generated_file)).unwrap();

        let comparison = File::open(Path::new("./test_data/DigitalsResults.csv")).unwrap();
        let comparison_checksum = adler32(BufReader::new(comparison)).unwrap();

        assert_eq!(generated_checksum, comparison_checksum);
        fs::remove_file(generated).await.unwrap();
    }

    #[tokio::test]
    async fn process_reduced_test() {
        fs::remove_file(".test.db").await;
        let (ts_tx, mut ts_rx) = unbounded_channel();
        let (g_tx, mut g_rx) = unbounded_channel();

        let options = match SqliteConnectOptions::from_str("sqlite://.test.db") {
            Ok(o) => o,
            Err(e) => {
                panic!("unable to create options for sqlite connection {e:?}")
            }
        };

        let db = match SqlitePool::connect_with(options.create_if_missing(true)).await {
            Ok(d) => d,
            Err(e) => {
                panic!("unable to connect to sqlite database {e:?}")
            }
        };

        let isu = match ISUProcessor::new() {
            Ok(p) => p,
            Err(e) => {
                panic!("init_test failed {e:?}")
            }
        };

        let result = isu.init(db.clone());
        assert!(result.is_ok(), "{:?}", result.err());

        let result = isu.process(
            PathBuf::from_str("./test_data/Feb_13_2023_14_29/Reduced Raw Data.txt").unwrap(),
            db.clone(),
            Some(ts_tx.clone()),
            Some(g_tx.clone()),
        );

        assert!(result.is_ok(), "Error during process call: {result:?}");

        // we should see a total of one message on the timeseries channel, should be a file
        let mut message = ts_rx.recv().await.unwrap();
        let generated = match message {
            DataSourceMessage::File(f) => {
                assert!(f.to_str().is_some());
                f
            }
            DataSourceMessage::Data(_) => {
                panic!("wrong message type received")
            }
            DataSourceMessage::Close => {
                panic!("wrong message type received")
            }
        };

        let generated_file = File::open(&generated).unwrap();
        let generated_checksum = adler32(BufReader::new(generated_file)).unwrap();

        let comparison = File::open(Path::new("./test_data/ReducedResults.csv")).unwrap();
        let comparison_checksum = adler32(BufReader::new(comparison)).unwrap();

        assert_eq!(generated_checksum, comparison_checksum);
        fs::remove_file(generated).await.unwrap();
    }

    #[tokio::test]
    async fn process_temperature_test() {
        fs::remove_file(".test.db").await;
        let (ts_tx, mut ts_rx) = unbounded_channel();
        let (g_tx, mut g_rx) = unbounded_channel();

        let options = match SqliteConnectOptions::from_str("sqlite://.test.db") {
            Ok(o) => o,
            Err(e) => {
                panic!("unable to create options for sqlite connection {e:?}")
            }
        };

        let db = match SqlitePool::connect_with(options.create_if_missing(true)).await {
            Ok(d) => d,
            Err(e) => {
                panic!("unable to connect to sqlite database {e:?}")
            }
        };

        let isu = match ISUProcessor::new() {
            Ok(p) => p,
            Err(e) => {
                panic!("init_test failed {e:?}")
            }
        };

        let result = isu.init(db.clone());
        assert!(result.is_ok(), "{:?}", result.err());

        let result = isu.process(
            PathBuf::from_str("./test_data/Feb_13_2023_14_29/Temperature.txt").unwrap(),
            db.clone(),
            Some(ts_tx.clone()),
            Some(g_tx.clone()),
        );

        assert!(result.is_ok(), "Error during process call: {result:?}");

        // we should see a total of one message on the timeseries channel, should be a file
        let mut message = ts_rx.recv().await.unwrap();
        let generated = match message {
            DataSourceMessage::File(f) => {
                assert!(f.to_str().is_some());
                f
            }
            DataSourceMessage::Data(_) => {
                panic!("wrong message type received")
            }
            DataSourceMessage::Close => {
                panic!("wrong message type received")
            }
        };

        let generated_file = File::open(&generated).unwrap();
        let generated_checksum = adler32(BufReader::new(generated_file)).unwrap();

        let comparison = File::open(Path::new("./test_data/TemperatureResults.csv")).unwrap();
        let comparison_checksum = adler32(BufReader::new(comparison)).unwrap();

        assert_eq!(generated_checksum, comparison_checksum);
        fs::remove_file(generated).await.unwrap();
    }

    #[tokio::test]
    async fn process_event_test() {
        fs::remove_file(".test.db").await;
        let (ts_tx, mut ts_rx) = unbounded_channel();
        let (g_tx, mut g_rx) = unbounded_channel();

        let options = match SqliteConnectOptions::from_str("sqlite://.test.db") {
            Ok(o) => o,
            Err(e) => {
                panic!("unable to create options for sqlite connection {e:?}")
            }
        };

        let db = match SqlitePool::connect_with(options.create_if_missing(true)).await {
            Ok(d) => d,
            Err(e) => {
                panic!("unable to connect to sqlite database {e:?}")
            }
        };

        let isu = match ISUProcessor::new() {
            Ok(p) => p,
            Err(e) => {
                panic!("init_test failed {e:?}")
            }
        };

        let result = isu.init(db.clone());
        assert!(result.is_ok(), "{:?}", result.err());

        std::fs::copy(
            "./test_data/Feb_13_2023_14_29/Events.txt",
            "./test_data/Feb_13_2023_14_29/EventsTail.txt",
        )
        .unwrap();

        let result = isu.process(
            PathBuf::from_str("./test_data/Feb_13_2023_14_29/EventsTail.txt").unwrap(),
            db.clone(),
            Some(ts_tx.clone()),
            Some(g_tx.clone()),
        );

        assert!(result.is_ok(), "Error during process call: {result:?}");

        // we should see a total of one message on the timeseries channel, should be a file
        let mut message = ts_rx.recv().await.unwrap();
        let generated = match message {
            DataSourceMessage::File(f) => {
                assert!(f.to_str().is_some());
                f
            }
            DataSourceMessage::Data(_) => {
                panic!("wrong message type received")
            }
            DataSourceMessage::Close => {
                panic!("wrong message type received")
            }
        };

        let generated_file = File::open(&generated).unwrap();
        let generated_checksum = adler32(BufReader::new(generated_file)).unwrap();

        let comparison = File::open(Path::new("./test_data/EventsResults.csv")).unwrap();
        let comparison_checksum = adler32(BufReader::new(comparison)).unwrap();

        assert_eq!(generated_checksum, comparison_checksum);
        fs::remove_file(generated).await.unwrap();

        // now we're testing the tailing
        let mut file = OpenOptions::new()
            .append(true)
            .open("./test_data/Feb_13_2023_14_29/EventsTail.txt")
            .unwrap();

        let written = file.write(String::from("Test Event\n").as_bytes()).unwrap();
        assert!(written > 0);

        file.flush().unwrap();
        let result = isu.process(
            PathBuf::from_str("./test_data/Feb_13_2023_14_29/EventsTail.txt").unwrap(),
            db,
            Some(ts_tx),
            Some(g_tx),
        );

        assert!(result.is_ok(), "Error during process call: {result:?}");

        // we should see a total of one message on the timeseries channel, should be a file
        message = ts_rx.recv().await.unwrap();
        let generated = match message {
            DataSourceMessage::File(f) => {
                assert!(f.to_str().is_some());
                f
            }
            DataSourceMessage::Data(_) => {
                panic!("wrong message type received")
            }
            DataSourceMessage::Close => {
                panic!("wrong message type received")
            }
        };

        let generated_file = File::open(&generated).unwrap();
        let generated_checksum = adler32(BufReader::new(generated_file)).unwrap();

        let comparison = File::open(Path::new("./test_data/EventsTailResults.csv")).unwrap();
        let comparison_checksum = adler32(BufReader::new(comparison)).unwrap();

        assert_eq!(generated_checksum, comparison_checksum);
        fs::remove_file(generated).await.unwrap();
    }
}

# MachineLearning
---------------

This is the machine learning portion of the AGN-201 Digital Twin. The purpose of this application is to pull down data from DeepLynx and run various Jupyter notebooks against it, then upload the results up to DeepLynx. This is designed to run either on an HPC cluster or in the cloud in a Docker image. In order to make this easier to understand we will go through how the application works when deployed as a Docker image.

1. The application starts.
2. Jester starts and begins monitoring a directory specified by the configuration file - once files appear in this directory they are uploaded to DeepLynx.
3. The application downloads the latest data from DeepLynx - controlled by the configuration file.
4. The application loads the data into a DuckDB instance and makes that instance available to the Python notebooks.
5. The application uses Papermill to run each one of the three included notebooks (Anomaly, Linear, Neutronics) - notebooks are run in parallel and should be configured to output their results to specific directories (in the Dockerfile they are hardcoded).
6. Jester uploads the notebook results up to DeepLynx.
7. The application wait for the configured period interval and then runs through the cycle again.


## Developing (Python Notebooks)

The easiest way to iterate on the jupyter notebooks and see if your updates function correctly is to use the included Dockerfile to generate a Docker image. By using Docker we streamline the Rust part of the application for you and try to make everything as easy as possible to get up and running. If you are on the INL network you will most likely need to get off ZScaler to get this to build correctly. Here are the steps necessary to get this up and running.

1. Build the Docker image using the Dockerfile in this directory. If you're on the INL network you will need to disconnect or be offsite to get this to build correctly.
2. Create your configuration file - see the next section for how to build said file
3. Once the Dockerfile has been built into an image successfully, you will need to launch the image, mounting the following volumes:
   1. {your path}:/configs - this is the directory you will need to use to mount the configuration file for the main Rust application and Jester. See the next section for configuration file samples. If you do not want Jester to run, simply do not include a separate configuration file for it
   2. {your path}:/(linear_out,neutronics_out,anomaly_out) - these are the output directorys for the notebooks that Jester should be watching if configured correctly (you can also disable Jester for development purposes)
   3. {your python notebook path, or the notebook path in this directory}:/program/machine_learning/python/notebooks - these are the notebooks that the application will run - currently they must maintain the same naming structure, or they will not run unless you provide a custom configuration (see below)


## Configuration File
The good news is that even though Jester and the application both need a configuration file, you should be able to use the same configuration file for both of them. For local development I recommend that you simply not include a configuration for Jester unless you need to test the upload capability.

Configuration file should be a YAML file = you can pass it as an argument to the application `application --config-file=path` or it will default to finding a file in the same directory called `.config.yml`.

### Jester Configuration File

You can find all the information on what files are required [here](https://github.inl.gov/Digital-Engineering/Jester#configuration-file).


### Rust Application Configuration File

Here is a sample of the configuration file you'll need to build and mount in the Docker container in order to make the application work correctly. The application is fairly good at telling you when you get something in here wrong.

```yaml
api_key: Your DeepLynx API key
api_secret: Your DeepLynx API secret
deep_lynx_url: The DeepLynx URL - without the trailing slash
db_path: The path the DuckDB file should be stored - make special note of what this is as you'll commonly need to know for your Python notebooks to be able to access. No default here
period_interval: 64-bit number indicating number of seconds between each iteration of the loop described at the start of this document
data_retention_days: How many days worth of data the DuckDB instance should hold in it. I suggest a low number in anything other than development so that you do not balloon the size of your Docker container 
linear_notebook_path: Defaults to ./python/notebooks/LinearPrediction.ipynb (jupyter notebook path)
anomaly_notebook_path: Defaults to ./python/notebooks/AnomalyPrediction.ipynb (jupyter notebook path)
neutronics_notebook_path: Defaults to ./python/notebooks/NeutronicsPrediction.ipynb (jupyter notebook path)
debug: boolean - highly recommended that you set this to true (note lowercase not Case) when developing so you can see exactly what's happening and share logs
data_sources: (this is the most important bit, this is a YAML array of each data source you wish to pull from DeepLynx and load into DuckDB)
  - table_name: DuckDB table name - recommended lower case and snake_case
    container_id: DeepLynx container ID of the data-source
    data_source_id: DeepLynx DataSource ID
    timestamp_column_name: The _column_ name of the column of data which should be treated as the timestamp. Typically this is the first column in the DeepLynx data source but you can also point it to whatever column you choose as long as its a valid timestamp
    secondary_index: This is the secondary index - how the rows should be ordered after they are ordered by timestamp. This is essential for this project as the secondary index is relative to experiment start time
    initial_timestamp: The earliest data that should be loaded into the the DuckDB instance when the application first runs
    initial_index: Initial vaule of the secondary index (recommend 0 in most cases)
```

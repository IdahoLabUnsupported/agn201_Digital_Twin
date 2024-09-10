# AGN-201-Digital-Twin

------------

This is the repository for all code related to the AGN-201 Nuclear Reactor Digital Twin at Idaho State University. The goal of this code repository is to consolidate all pieces required to run the AGN-201 Digital Twin in the [DeepLynx](https://github.com/idaholab/Deep-Lynx) ecosystem. This is the first successfully launched digital twin of a fissile nuclear reactor that we are aware of. While the code is not complex, the problems of networking, policy, and initial groundwork were significant to overcome.

We will now describe each section of this repository.

## JesterPlugin
_______
This is an AGN-201 ISU specific [Jester](https://github.com/idaholab/Jester) plugin. This plugin is responsible for working with Jester to inform it how the AGN-201's DAS outputs the sensor readings and how to take those readings and ingest them into DeepLynx. This plugin is written in Rust and is designed to tail various CSV files to read new data and send them on an interval to the DeepLynx data lake. There are tests to ensure that the current data structure works with DeepLynx. If that data structure changes you will need to update this code.

## MachineLearning

--------

This is as Rust program designed to utilize [Papermill](https://papermill.readthedocs.io/en/latest/index.html) to run various Jupyter notebooks designed by the INL AGN-201 Digital Twin team. These notebooks employ various machine learning algorithms to highlight anomalies and differences between expected values and actual values produced by the reactor. This repository also contains the models used for these machine learning pieces. More information can be found in the readme of this repository. This is designed to run on the cloud and pull data live from DeepLynx for analysis


## OperatorUI

-----

The operator UI is as [Tauri](https://tauri.app/) desktop application designed to give insights into reactor function to a reactor operator and monitor. This system incorporates the machine learning data ingested by DeepLynx from the program listed previously and displays it to the end user in an easy to digest format. We are continually developing this UI to provide more functionality and threshold monitoring so that it can eventually become an essential tool for monitoring and diagnosing reactor issues.


## AGN-MR

--------

This is the Unity project for the AGN 201 reactor.
# Overview

This repository contains the final project for the CS-E4780 Scalable Systems and Data Management course offered at Aalto university. For quick startup instructions, check [Startup](#startup). 

The goal of the project was to develop a high-performance application for processing and visualizing stock market data from three different stock exchanges. In addition to this, the application aims to find key metrics from the data that are then displayed to a user. 

The application consists of the following components:
- A Gateway simulator for simulating the constant flow of new stock marked datapoints by sending the data to a Redis broker. 
- A Redis pub/sub broker forward stock datapoints to the corresponding worker node.
- Worker nodes, which received stock events, calculate the important metrics such as EMA, bullish/bearish breakout pattern, detect invalid event and send these informatino to the database.  
- A Database, which stores data produced by the worker nodes.
- A Dashboard for retrieving information from the database and visualizing the data.

# Startup

Starting up the application requires
- Download the test [dataset](https://zenodo.org/records/6382482) and place the csv file in the /data folder.
- Configure important variables for the system in .env file
- Start the system with docker compose 


## Dataset

You can download the daily trading dataset from: https://zenodo.org/records/6382482

The csv files should be put in the data folder. The structure of the folder is as follows

```
project directory
|__ data
|   |__ debs2022-gc-trading-day-08-11-21.csv
|   |__ debs2022-gc-trading-day-09-11-21.csv
|   |__ ...
|__ ...
|__ .env
|__ README.md

```

## Template variables
The configuration of the system can be found in the .env files consists of the following important variables

| Variable                   | Details |
| --------                   | ------- |
| SIMULATOR_COUNT            | Number of stock exchange gateway replicas    |
| WORKER_COUNT               | Number of worker nodes     |
| WORKER_WINDOW_SIZE_SECONDS | Worker window size in seconds (Default value 300 seconds as in the project requirement, can handle 10s window)   |
| CSV_FILE                   | The path of the mounted csv file. The csv file should be in data folder, and the path is in form /data/{csv_filename}     |
| FILE_OFFSET_BYTES          | The gateway simulator also supports reading the csv file from a specific byte offset (Default value is 1) |

For the above folder structure, the environment can be set as follows:
 Variable                    | Value                                          |
| --------                   | -------                                        |
| SIMULATOR_COUNT            | 1                                              |
| WORKER_COUNT               | 1                                              |
| WORKER_WINDOW_SIZE_SECONDS | 10                                             |
| CSV_FILE                   | /data/debs2022-gc-trading-day-08-11-21.csv     |
| FILE_OFFSET_BYTES          | 1                                              |


## Start the system
The application can be started with running docker compose at the project base directory
```shell
docker compose up
```
After this, the visualization of the system can be found at [here](http://localhost:9999/d/ee39jei2kml1cc/my-dashboard?orgId=1&from=now-5m&to=now&timezone=browser&var-stock_id=MT.NL&var-WORKER_WINDOW_SIZE_SECONDS=10&var-SHOW_MOST_RECENT=10&refresh=10s) (http://localhost:9999/). Please wait a bit for the system to start. 


# Stop/Restart
if you want to stop/restart with running
The application can be started with running
```shell
docker compose down
```
In restart, try start again after this command.


# Repository Structure


## data/
The folder is used to stored the test data, and is later mounted to the gateway simulator container. The test csv file could be found [here](https://zenodo.org/records/6382482)

The **generated_csv_data.py** script is useful for generating the mock data for benchmarking by specifying the incoming event rate and duration.

## data_analysis/
The folder is used to store analysis script the the data incoming rate result. The information includes event rate per second, per 10 seconds and per 300 seconds (include and exclude invalid events)

## reports/
Contains material for latex report.

## scripts/
Contains some ultility scripts used during the development process.

## src/

The src directory contains the source code for the main components of the project.

### dashboard/
The folder consists of files used for Grafana dashboard and datasource provisioning. 

Technology: Grafana

### flyway/
The folder consists of sql script for database migration with flyway.

Technology: Flyway, PostgreSQL

### ingress-redis-streams
The folder contains the redis configuration file.

Technology: Redis

### gateway/

The Gateway Simulator is responsible for simulating a stock exchange gateway that receives data from a stock exchange and forward the data to the corresponding worker. In this project, to simulate that process, the gateway simulator read the csv file and send the event to the corresponding Redis queue.

Technology: Python

### worker/
The Worker is responsible for receive the event information periodically calculate the EMA indices, detect the bearish/bullish breakout pattern and send all the data to the database.

Technology: Rust

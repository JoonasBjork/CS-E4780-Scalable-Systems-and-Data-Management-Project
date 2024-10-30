# Overview

This repository contains the final project for the CS-E4780 Scalable Systems and Data Management course offered at Aalto university. For quick startup instructions, check [Startup](#startup). 

The goal of the project was to develop a high-performance application for processing and visualizing stock market data from three different stock exchanges. In addition to this, the application aims to find key metrics from the data that are then displayed to a user. 

The application consists of the following components:
- A Simulator for simulating the constant flow of new stock marked datapoints by sending the data to a Kafka cluster. 
- A Kafka cluster, which distributes the data sent by the Simulator into multiple Kafka topics. 
- Worker nodes, which query the Kafka cluster for data and output the data into a database 
- A Manager node, which is responsible for setting up and monitoring the system
- A Database, which stores data produced by the worker nodes
- A Frontend and a Backend for retrieving information from the database and visualizing the data

# Startup

Starting up the application requires
- Docker compose (?)
- ...

The application can be started with running
```shell
docker compose up
```
After this, the visualization of the system can be found at http://localhost:7800. 

# Repository Structure

## src/

The src directory contains the source code for the main components of the project.

### Simulator

The Simulator is responsible for simulating a stock exchange server that publishes data to clients. In the application, 
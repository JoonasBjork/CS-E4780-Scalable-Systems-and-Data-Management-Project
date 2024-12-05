-- turn on extension of timescalaledb
CREATE EXTENSION IF NOT EXISTS timescaledb CASCADE;

CREATE TABLE alerts (
    id SERIAL NOT NULL,--PRIMARY KEY, -- The alerts can be queried based on the id (just always query for alerts that have come after the last retrieved id)
    symbol VARCHAR(32),
    sectype VARCHAR(1),
    last FLOAT,
    trading_timestamp TIMESTAMP,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE indicators (
  symbol VARCHAR(32) NOT NULL,
  ema_38 FLOAT NOT NULL,
  ema_100 FLOAT NOT NULL,
  bullish BOOLEAN NOT NULL,
  bearish BOOLEAN NOT NULL,
  last_price FLOAT,
  last_trade_timestamp TIMESTAMP, -- May be null as there can be windows without any new values
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  message_count INT NOT NULL, -- Number of messages received in the window
  average_latency_ms INT-- Average latency from simulator to being processed by worker in milliseconds
  --PRIMARY KEY (symbol, created_at)
);

CREATE TABLE symbols (
  symbol VARCHAR(32) PRIMARY KEY
);

--convert it into a hypertable:
SELECT create_hypertable('alerts', 'created_at');
CREATE UNIQUE INDEX alerts_unique_idx ON alerts (id, created_at);

SELECT create_hypertable('indicators', 'created_at');
CREATE UNIQUE INDEX indicators_unique_idx ON indicators (symbol, created_at);
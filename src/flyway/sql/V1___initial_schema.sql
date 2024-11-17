CREATE TABLE alerts (
    id SERIAL PRIMARY KEY, -- The alerts can be queries based on the id (just always query for alerts that have come after the last retrieved id)
    symbol VARCHAR(32),
    sectype VARCHAR(1),
    last FLOAT,
    trading_timestamp TIMESTAMP,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE indicators (
  symbol VARCHAR(32),
  ema_38 FLOAT NOT NULL,
  ema_100 FLOAT NOT NULL,
  bullish BOOLEAN NOT NULL,
  bearish BOOLEAN NOT NULL,
  last_trade_timestamp TIMESTAMP, -- May be null as there can be windows without any new values
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (symbol, created_at)
);
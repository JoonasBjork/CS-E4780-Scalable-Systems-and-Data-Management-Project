CREATE TABLE IF NOT EXISTS environment_variables (
    key TEXT PRIMARY KEY,
    value TEXT
);

INSERT INTO environment_variables (key, value)
VALUES ('WORKER_WINDOW_SIZE_SECONDS', '${worker_window_size_seconds}');
CREATE TABLE IF NOT EXISTS namespaces (
    name TEXT PRIMARY KEY
);

CREATE TABLE IF NOT EXISTS apikeys (
    id INTEGER PRIMARY KEY,
    namespace TEXT,
    digest BLOB,
    salt BLOB,
    FOREIGN KEY (namespace) REFERENCES namespaces(name)
);
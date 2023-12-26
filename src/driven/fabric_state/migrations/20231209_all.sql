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

CREATE TABLE IF NOT EXISTS resources (
    id INTEGER PRIMARY KEY,
    namespace TEXT,
    name TEXT,
    uuid BLOB,
    kind TEXT,
    manifest BLOB,
    FOREIGN KEY (namespace) REFERENCES namespaces(name)
);
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
    uuid BLOB PRIMARY KEY,
    namespace TEXT,
    name TEXT,
    kind TEXT,
    manifest BLOB,
    FOREIGN KEY (namespace) REFERENCES namespaces(name)
);

CREATE TABLE IF NOT EXISTS accounting (
    id INTEGER PRIMARY KEY,
    epoch INTEGER,
    entry BLOB,
    cluster BLOB,
    namespace TEXT,
    resource BLOB DEFAULT NULL,
    account INTEGER,
    debit INTEGER NULL,
    credit INTEGER NULL,
    FOREIGN KEY (namespace) REFERENCES namespaces(name)
    FOREIGN KEY (resource) REFERENCES resources(uuid)
);
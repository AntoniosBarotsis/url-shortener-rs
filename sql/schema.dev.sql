DROP TABLE IF EXISTS metadata;
DROP TABLE IF EXISTS urls;

CREATE TABLE urls (
  id VARCHAR(6) PRIMARY KEY,
  url VARCHAR NOT NULL
);

CREATE TABLE metadata (
  id VARCHAR(6) REFERENCES urls(id),
  url VARCHAR NOT NULL,
  hits INT NOT NULL
);

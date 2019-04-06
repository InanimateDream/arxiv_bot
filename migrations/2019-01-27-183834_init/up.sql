PRAGMA foreign_keys = ON;

CREATE TABLE papers (
  arxiv_id TEXT NOT NULL,
  title    TEXT NOT NULL,
  abstract TEXT NOT NULL,
  prim_sub TEXT NOT NULL,
  PRIMARY KEY(arxiv_id)
);

CREATE TABLE authors (
  arxiv_id TEXT NOT NULL,
  auth     TEXT NOT NULL,
  PRIMARY KEY(arxiv_id, auth),
  FOREIGN KEY(arxiv_id) REFERENCES papers(arxiv_id) ON DELETE CASCADE
);

CREATE TABLE subjects (
  arxiv_id TEXT NOT NULL,
  sub      TEXT NOT NULL,
  PRIMARY KEY(arxiv_id, sub),
  FOREIGN KEY(arxiv_id) REFERENCES papers(arxiv_id) ON DELETE CASCADE
);

CREATE TABLE update_time (
  subject TEXT NOT NULL,
  rss_time DATETIME NOT NULL,
  PRIMARY KEY(subject)
);

CREATE TABLE pins (
  id       TEXT NOT NULL,
  ref_id   TEXT,
  arxiv_id TEXT NOT NULL,
  pub_date DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY(id),
  FOREIGN KEY(arxiv_id) REFERENCES papers(arxiv_id)
);

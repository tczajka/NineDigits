CREATE TABLE player (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL,
  bot_name TEXT NOT NULL
);

CREATE TABLE competition (
  id INTEGER PRIMARY KEY,
  date TEXT NOT NULL
);

CREATE TABLE round (
  id INTEGER PRIMARY KEY,
  competition_id INTEGER NOT NULL
);

CREATE TABLE game (
  id INTEGER PRIMARY KEY,
  round_id INTEGER NOT NULL,
  player_first INTEGER NOT NULL,
  player_second INTEGER NOT NULL,
  moves TEXT NOT NULL
);

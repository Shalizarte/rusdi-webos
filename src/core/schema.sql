-- Schema SQLite para biblioteca Rusdi

CREATE TABLE IF NOT EXISTS movies (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL, year INTEGER, plot TEXT, poster TEXT,
    filepath TEXT NOT NULL UNIQUE, runtime INTEGER,
    date_added DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS tvshows (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL, year INTEGER, plot TEXT, poster TEXT,
    date_added DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS episodes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    show_id INTEGER NOT NULL, season INTEGER NOT NULL, episode INTEGER NOT NULL,
    title TEXT NOT NULL, plot TEXT, filepath TEXT NOT NULL,
    FOREIGN KEY(show_id) REFERENCES tvshows(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS music (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL, artist TEXT NOT NULL, album TEXT,
    year INTEGER, genre TEXT, filepath TEXT NOT NULL UNIQUE,
    duration INTEGER, date_added DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_movies_title ON movies(title);
CREATE INDEX IF NOT EXISTS idx_episodes_show ON episodes(show_id, season, episode);
CREATE INDEX IF NOT EXISTS idx_music_artist ON music(artist);

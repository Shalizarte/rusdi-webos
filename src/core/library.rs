//! Biblioteca de mídia com SQLite + scrapers.

use crate::error::{RusdiError, Result};
use sqlx::{SqlitePool, Row};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Movie {
    pub id: i64, pub title: String, pub year: Option<i32>,
    pub plot: Option<String>, pub poster: Option<String>,
    pub filepath: String, pub runtime: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TvShow {
    pub id: i64, pub title: String, pub year: Option<i32>,
    pub plot: Option<String>, pub poster: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Episode {
    pub id: i64, pub show_id: i64, pub season: i32, pub episode: i32,
    pub title: String, pub plot: Option<String>, pub filepath: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MusicTrack {
    pub id: i64, pub title: String, pub artist: String, pub album: String,
    pub year: Option<i32>, pub genre: Option<String>, pub filepath: String,
    pub duration: Option<i32>,
}

pub struct Library {
    pool: SqlitePool,
}

impl Library {
    pub async fn new(db_path: &str) -> Result<Self> {
        let pool = SqlitePool::connect(&format!("sqlite://{}", db_path)).await?;
        sqlx::query(include_str!("schema.sql")).execute(&pool).await?;
        Ok(Self { pool })
    }

    pub async fn add_movie(&self, m: &Movie) -> Result<i64> {
        let row = sqlx::query("INSERT INTO movies (title, year, plot, poster, filepath, runtime) VALUES (?, ?, ?, ?, ?, ?) RETURNING id")
            .bind(&m.title).bind(m.year).bind(&m.plot).bind(&m.poster).bind(&m.filepath).bind(m.runtime)
            .fetch_one(&self.pool).await?;
        Ok(row.get("id"))
    }

    pub async fn get_movies(&self) -> Result<Vec<Movie>> {
        let rows = sqlx::query_as::<_, Movie>("SELECT * FROM movies ORDER BY title").fetch_all(&self.pool).await?;
        Ok(rows)
    }

    pub async fn scan_directory(&self, path: &PathBuf, recursive: bool) -> Result<usize> {
        let mut count = 0;
        Ok(count)
    }

    pub async fn add_show(&self, s: &TvShow) -> Result<i64> {
        let row = sqlx::query("INSERT INTO tvshows (title, year, plot, poster) VALUES (?, ?, ?, ?) RETURNING id")
            .bind(&s.title).bind(s.year).bind(&s.plot).bind(&s.poster)
            .fetch_one(&self.pool).await?;
        Ok(row.get("id"))
    }

    pub async fn add_episode(&self, e: &Episode) -> Result<i64> {
        let row = sqlx::query("INSERT INTO episodes (show_id, season, episode, title, plot, filepath) VALUES (?, ?, ?, ?, ?, ?) RETURNING id")
            .bind(e.show_id).bind(e.season).bind(e.episode).bind(&e.title).bind(&e.plot).bind(&e.filepath)
            .fetch_one(&self.pool).await?;
        Ok(row.get("id"))
    }

    pub async fn add_track(&self, t: &MusicTrack) -> Result<i64> {
        let row = sqlx::query("INSERT INTO music (title, artist, album, year, genre, filepath, duration) VALUES (?, ?, ?, ?, ?, ?, ?) RETURNING id")
            .bind(&t.title).bind(&t.artist).bind(&t.album).bind(t.year).bind(&t.genre).bind(&t.filepath).bind(t.duration)
            .fetch_one(&self.pool).await?;
        Ok(row.get("id"))
    }
}

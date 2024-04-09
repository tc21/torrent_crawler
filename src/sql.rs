use std::path::Path;

use rusqlite::{params, Connection};

const DB_FILE: &str = "database.db";

#[derive(Debug)]
pub struct Show {
    pub title: String,
    pub search_string: Option<String>,
    pub next_episode: i32,
    pub total_episodes: i32
}

#[derive(Debug)]
pub struct Episode {
    pub title: String,
    pub episode: i32,
    pub url: String
}

fn initialize_db(path: &Path) -> Result<Connection, rusqlite::Error> {
    let conn = Connection::open(path)?;

    conn.execute_batch(
        "CREATE TABLE shows (
            title TEXT NOT NULL PRIMARY KEY,
            search_string TEXT,
            next_episode INTEGER NOT NULL DEFAULT 1,
            total_episodes INTEGER NOT NULL DEFAULT -1
        );

        CREATE TABLE episodes (
            title TEXT NOT NULL REFERENCES shows,
            episode INTEGER NOT NULL,
            url TEXT NOT NULL,
            PRIMARY KEY (title, episode)
        );
        "
    )?;

    Ok(conn)
}

fn connect(config_dir: &str) -> Result<Connection, rusqlite::Error> {
    let db_file_location = Path::new(config_dir).join(DB_FILE);
    if db_file_location.exists() {
        Connection::open(db_file_location)
    } else {
        initialize_db(&db_file_location)
    }
}

pub fn insert(config_dir: &str, show: &Show) -> Result<usize, rusqlite::Error> {
    let connection = connect(config_dir)?;

    connection.execute(
        "INSERT OR REPLACE INTO shows
            (title, search_string, next_episode, total_episodes)
        VALUES
            (?, ?, ?, ?)
        ",
        params![show.title, show.search_string, show.next_episode, show.total_episodes])
}

pub fn insert_episode(config_dir: &str, episode: &Episode) -> Result<usize, rusqlite::Error> {
    let connection = connect(config_dir)?;

    connection.execute(
        "INSERT INTO episodes
            (title, episode, url)
        VALUES
            (?, ?, ?)
        ",
        params![episode.title, episode.episode, episode.url])
}

pub fn search_title(config_dir: &str, title: &str, show_completed: bool) -> Result<Vec<Show>, rusqlite::Error> {
    let connection = connect(config_dir)?;

    let mut statement = if show_completed {
        connection.prepare(
            "SELECT * FROM shows WHERE title LIKE ?"
        )
    } else {
        connection.prepare(
            "SELECT * FROM shows
                WHERE title LIKE ?
                AND (total_episodes = -1 OR next_episode <= total_episodes)"
        )
    }?;

    let param = format!("%{}%", title);

    let shows = statement.query_map([param], |row| {
        Ok(Show {
            title: row.get(0)?,
            search_string: row.get(1)?,
            next_episode: row.get(2)?,
            total_episodes: row.get(3)?,
        })
    })?;

    shows.collect::<Result<Vec<_>, _>>()
}

pub fn complete(config_dir: &str, title: &str) -> Result<usize, rusqlite::Error> {
    let connection = connect(config_dir)?;

    connection.execute(
        "UPDATE shows SET total_episodes = next_episode - 1 WHERE title = ?",
        [title]
    )
}

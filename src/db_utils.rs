use std::fs;
use tera::Tera;
use tera::Context;
use serde_derive::Serialize;
use chrono::prelude::{DateTime, Utc};
use log::{debug, error, info};
use rusqlite::Connection;
use std::path::Path;

const DB_PATH: &'static str = "/tmp/cinema.db";

#[derive(Debug, Serialize)]
struct Movie {
    id: Option<i32>,
    director: String,
    timetable: String,
    title: String,
    plot: String,
    url: String,
    guid: String,
    pub_date: DateTime<Utc>,
    // a fake db field to store the date in RFC2822 format
    pub_date_rfc2822: Option<String>,
    read_date: Option<DateTime<Utc>>,
}

pub fn init_db(purge_db: bool) {
    let db_path = Path::new(DB_PATH);
    if db_path.exists() && db_path.is_file() {
        if !purge_db {
            return;
        }
        match fs::remove_file(db_path) {
            Ok(_) => {}
            Err(res) => error!("Error while deleting: {}", res),
        };
    }
    let conn = Connection::open(db_path).unwrap();
    match conn.execute(
        "CREATE TABLE movie (
                          id              INTEGER PRIMARY KEY,
                          director        TEXT NOT NULL,
                          timetable       TEXT NOT NULL,
                          title           TEXT NOT NULL,
                          plot            TEXT NOT NULL,
                          url             TEXT NOT NULL,
                          guid            TEXT NOT NULL,
                          pub_date        TEXT NOT NULL,
                          read_date       TEXT NULL)",
        rusqlite::NO_PARAMS,
    ) {
        Ok(_) => debug!("Table created"),
        Err(err) => error!("Table creation failed: {}", err),
    }
}

pub fn insert_movie(
    title: String,
    director: String,
    timetable: String,
    plot: String,
    url: String,
    pub_date: DateTime<Utc>,
) {
    let db_path = Path::new(DB_PATH);
    let conn = Connection::open(db_path).unwrap();
    match conn.execute(
        "INSERT INTO movie (title, director, timetable, \
         plot, url, guid, pub_date)
                      VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        &[&title, &director, &timetable, &plot, &url, &url, &pub_date.to_rfc2822()],
    ) {
        Ok(inserted) => debug!("{} row(s) were inserted", inserted),
        Err(err) => error!("INSERT failed: {}", err),
    }
}

pub fn get_movie(url: String) -> i32 {
    let db_path = Path::new(DB_PATH);
    let conn = Connection::open(db_path).unwrap();
    let count: i32 = conn
        .query_row(
            "SELECT count(*) FROM movie WHERE guid = (?)",
            &[&url],
            |row| row.get(0),
        )
        .ok()
        .unwrap();
    info!("{} Record found for {}", count, url);
    return count;
}

pub fn get_movies_xml(feed_path: String) {
    let db_path = Path::new(DB_PATH);
    let conn = Connection::open(db_path).unwrap();
    let mut stmt = conn
        .prepare(
            "SELECT id, title, director, timetable, \
             plot, url, guid, pub_date, read_date FROM movie",
        )
        .unwrap();
    let movie_iter = stmt
        .query_map(rusqlite::NO_PARAMS, |row| Movie {
            id: row.get(0),
            title: row.get(1),
            director: row.get(2),
            timetable: row.get(3),
            plot: row.get(4),
            url: row.get(5),
            guid: row.get(6),
            pub_date: row.get(7),
            pub_date_rfc2822: row.get(7),
            read_date: row.get(8),
        })
        .unwrap();

    let mut movie_list = vec![];
    for movie in movie_iter {
        let mut m = match movie {
            Ok(x) => x,
            Err(err) => {
                error!("Skipping movie: {}", err);
                continue;
            }
        };
        m.pub_date_rfc2822 = Some(m.pub_date.to_rfc2822());
        movie_list.push(m);
    }

    let mut ctx = Context::new();
    ctx.insert("items", &movie_list);
    let _tera = Tera::new("./*").unwrap();
    let result = _tera.render("feed.tera", &ctx).unwrap();
    let full_feed_path = format!("{}feed.xml", feed_path);
    fs::write(full_feed_path, result).unwrap();
}

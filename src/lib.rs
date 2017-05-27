#[macro_use]
extern crate log;
extern crate env_logger;

extern crate rusqlite;
extern crate chrono;

static DB_PATH: &'static str = "/tmp/cinema.db";

pub mod db_utils {

    use std::path::Path;
    use std::fs::{File, remove_file};
    use std::io::Write;
    use chrono::{DateTime, UTC};
    use rusqlite::Connection;
    use DB_PATH;

    #[derive(Debug)]
    struct Movie {
        id: Option<i32>,
        director: String,
        timetable: String,
        title: String,
        plot: String,
        url: String,
        guid: String,
        creation_date: DateTime<UTC>,
        read_date: Option<DateTime<UTC>>,
    }

    // TODO: use a Default to automatically set creation_date
    // impl Default for Movie {
    //     fn default() -> Movie {
    //         Movie {id: None,
    //                director: String::new(), timetable: String::new(),
    //                title: String::new(), plot: String::new(),
    //                url: String::new(),
    //                guid: String::new(),
    //                creation_date: UTC::now(),
    //                read_date: None}
    //     }
    // }

    pub fn init_db() {
        let db_path = Path::new(DB_PATH);
        if db_path.exists() && db_path.is_file() {
            match remove_file(db_path) {
                Ok(_) => {}
                Err(res) => error!("Error while deleting: {}", res),
            };
        }
        let conn = Connection::open(db_path).unwrap();
        match conn.execute("CREATE TABLE movie (
                          id              INTEGER PRIMARY KEY,
                          director        TEXT NOT NULL,
                          timetable       TEXT NOT NULL,
                          title           TEXT NOT NULL,
                          plot            TEXT NOT NULL,
                          url             TEXT NOT NULL,
                          guid            TEXT NOT NULL,
                          creation_date   TEXT NOT NULL,
                          read_date       TEXT NULL)",
                           &[]) {
            Ok(_) => debug!("Table created"),
            Err(err) => error!("Table creation failed: {}", err),
        }
    }

    pub fn insert_movie(title: String,
                        director: String,
                        timetable: String,
                        plot: String,
                        url: String) {
        let db_path = Path::new(DB_PATH);
        let conn = Connection::open(db_path).unwrap();
        let movie = Movie {
            id: None,
            title: title,
            director: director,
            timetable: timetable,
            plot: plot,
            url: String::from(""),
            guid: url,
            creation_date: UTC::now(),
            read_date: None,
        };

        // use chrono::prelude::*;
        // let dt = UTC.ymd(2014, 11, 28).and_hms(12, 0, 9);
        // println!("{:?}", dt.to_rfc2822());

        match conn.execute("INSERT INTO movie (title, director, timetable, \
                            plot, url, guid, creation_date)
                      VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                           &[&movie.title,
                             &movie.director,
                             &movie.timetable,
                             &movie.plot,
                             &movie.url,
                             &movie.guid,
                             &movie.creation_date]) {
            Ok(inserted) => debug!("{} row(s) were inserted", inserted),
            Err(err) => error!("INSERT failed: {}", err),
        }
    }

    pub fn get_movies_xml() {
        let db_path = Path::new(DB_PATH);
        let conn = Connection::open(db_path).unwrap();
        let mut stmt = conn.prepare("SELECT id, title, director, timetable, \
                                     plot, url, guid, creation_date, read_date FROM movie")
            .unwrap();
        let movie_iter = stmt.query_map(&[], |row| {
                Movie {
                    id: row.get(0),
                    title: row.get(1),
                    director: row.get(2),
                    timetable: row.get(3),
                    plot: row.get(4),
                    url: row.get(5),
                    guid: row.get(6),
                    creation_date: row.get(7),
                    read_date: row.get(8),
                }
            })
            .unwrap();
        let mut fp = File::create("feed.xml").expect("just die");
        match fp.write(b"<?xml version=\"1.0\" encoding=\"utf-8\" ?>
    <rss version=\"2.0\" xmlns:atom=\"http://www.w3.org/2005/Atom\">
        <channel>
           <atom:link href=\"http://www.storiepvtride.it/rss/feed.xml\" rel=\"self\" type=\"application/rss+xml\" />
            <title>Cinema feed</title>
            <link>http://www.storiepvtride.it/rss/feed.xml</link>
            <description>Cinema RSS</description>") {
            Ok(_) => debug!("Successfully written movie list header"),
            Err(err) => {
                error!("Error writing movie list header: {}", err.to_string());
            }
        };
        for movie in movie_iter {
            let m = match movie {
                Ok(x) => x,
                Err(err) => {
                    error!("Skipping movie: {}", err);
                    continue;
                }
            };
            match write!(fp,
                         "<item>
                <title>{}</title>
                <description>{}</description>
                <pubDate>{}</pubDate>
                <guid>{}</guid>
            </item>",
                         m.title,
                         m.plot.replace('&', "&amp;"),
                         m.creation_date.to_rfc2822(),
                         m.guid) {
                Ok(_) => {
                    debug!("Successfully written movie item");
                }
                Err(err) => {
                    error!("Error writing movie item: {}", err.to_string());
                }
            };
        }

        match write!(fp, "</channel></rss>") {
            Ok(_) => {
                debug!("Successfully written movie list tail");
            }
            Err(err) => {
                error!("Error writing movie list tail: {}", err.to_string());
            }
        };
        fp.flush().unwrap();
    }
}

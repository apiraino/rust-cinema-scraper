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
    use chrono::prelude::{DateTime, Local};
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
        pub_date: DateTime<Local>,
        read_date: Option<DateTime<Local>>,
    }

    pub fn init_db(purge_db: bool) {
        let db_path = Path::new(DB_PATH);
        if db_path.exists() && db_path.is_file() {
            if !purge_db {
                return;
            }
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
                          pub_date        TEXT NOT NULL,
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
                        url: String,
                        pub_date: DateTime<Local>) {
        let db_path = Path::new(DB_PATH);
        let conn = Connection::open(db_path).unwrap();
        match conn.execute("INSERT INTO movie (title, director, timetable, \
                            plot, url, guid, pub_date)
                      VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                           &[&title,
                             &director,
                             &timetable,
                             &plot,
                             &url,
                             &url,
                             &pub_date]) {
            Ok(inserted) => debug!("{} row(s) were inserted", inserted),
            Err(err) => error!("INSERT failed: {}", err),
        }
    }

    pub fn get_movie(url: String) -> i32 {
        let db_path = Path::new(DB_PATH);
        let conn = Connection::open(db_path).unwrap();
        let count : i32 = conn.query_row("SELECT count(*) FROM movie WHERE guid = (?)", &[&url], |row| {
            row.get(0)
        }).ok().unwrap();
        info!("{} Record found for {}", count, url);
        return count;
    }

    pub fn get_movies_xml(feed_path: String) {
        let db_path = Path::new(DB_PATH);
        let conn = Connection::open(db_path).unwrap();
        let mut stmt = conn.prepare("SELECT id, title, director, timetable, \
                                     plot, url, guid, pub_date, read_date FROM movie")
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
                    pub_date: row.get(7),
                    read_date: row.get(8),
                }
            })
            .unwrap();
        let full_feed_path = format!("{}feed.xml", feed_path);
        let mut fp = File::create(full_feed_path).expect("just die");
        match fp.write(b"<?xml version=\"1.0\" encoding=\"utf-8\" ?>
    <rss version=\"2.0\" xmlns:atom=\"http://www.w3.org/2005/Atom\">
        <channel>
           <atom:link href=\"http://www.storiepvtride.it/rss/feed.xml\" rel=\"self\" type=\"application/rss+xml\" />
            <title>Cinema RSS feed</title>
            <link>http://www.storiepvtride.it/rss/feed.xml</link>
            <description>Cinema RSS feed</description>") {
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
                         format!("{}&lt;br&gt;Orari: {}", m.plot.replace('&', "&amp;"), m.timetable),
                         m.pub_date.to_rfc2822(),
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

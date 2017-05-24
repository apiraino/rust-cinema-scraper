extern crate rusqlite;
extern crate chrono;

static DB_PATH : &'static str = "/tmp/cinema.db";

pub mod db_utils {

    use std::path::Path;
    use std::fs::{File,remove_file};
    use std::io::Write;
    use chrono::{DateTime,UTC};
    use rusqlite::Connection;
    use DB_PATH;

    // TODO: implement Trait to mangle a DateTime from a FromSql

    #[derive(Debug)]
    struct Movie {
        id: Option<i32>,
        director: String,
        timetable: String,
        title: String,
        plot: String,
        url: String,
        creation_date: DateTime<UTC>,
        read_date: Option<DateTime<UTC>>
    }

    impl Default for Movie {
        fn default() -> Movie {
            Movie {id: None,
                   director: String::new(), timetable: String::new(),
                   title: String::new(), plot: String::new(),
                   url: String::new(),
                   creation_date: UTC::now(),
                   read_date: None}
        }
    }

    pub fn init_db() {
        let db_path = Path::new(DB_PATH);
        if db_path.exists() && db_path.is_file() {
            match remove_file(db_path) {
                Ok(_) => {},
                Err(res) => { println!("Error while deleting: {}", res)}
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
                          creation_date   TEXT NOT NULL,
                          read_date       TEXT NULL)",
                           &[]) {
            Ok(_) => println!("Table created"),
            Err(err) => println!("Table creation failed: {}", err),
        }
    }

    pub fn insert_movie(title:String, director:String, timetable:String,
                        plot:String, url:String) {
        let db_path = Path::new(DB_PATH);
        let conn = Connection::open(db_path).unwrap();
        // TODO: use a Default to automatically set creation_date
        let movie = Movie {
            id: None,
            title: title,
            director: director,
            timetable: timetable,
            plot: plot,
            url: url,
            creation_date: UTC::now(),
            read_date: None
        };
        match conn.execute("INSERT INTO movie (title, director, timetable, \
                            plot, url, creation_date)
                      VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                           &[&movie.title, &movie.director, &movie.timetable,
                             &movie.plot, &movie.url, &movie.creation_date]) {
            Ok(inserted) => println!("{} row(s) were inserted", inserted),
            Err(err) => println!("INSERT failed: {}", err),
        }
    }

    pub fn get_movies_xml() {
        let db_path = Path::new(DB_PATH);
        let conn = Connection::open(db_path).unwrap();
        let mut stmt = conn.prepare("SELECT id, title, director, timetable, \
                                     plot, url, creation_date, read_date FROM movie").unwrap();
        let movie_iter = stmt.query_map(&[], |row| {
            Movie {
                id: row.get(0),
                title: row.get(1),
                director: row.get(2),
                timetable: row.get(3),
                plot: row.get(4),
                url: row.get(5),
                creation_date: row.get(6),
                read_date: row.get(7)
            }
        }).unwrap();
        let mut fp = File::create("feed.xml").expect("just die");
        match fp.write(b"<?xml version=\"1.0\" encoding=\"utf-8\" ?>
    <rss version=\"2.0\">
        <channel>
            <title>storiepvtride.it feed</title>
            <link>http://www.storiepvtride.it</link>
            <description>Free RSS Tutorial</description>") {
            Ok(_) => println!("Successfully written movie list header"),
            Err(err) => {println!("Error writing movie list header: {}", err.to_string());}
        };
        for movie in movie_iter {
            let m = match movie {
                Ok(x) => x,
                Err(err) => { println!("Skipping movie: {}", err); continue; }
            };
            match write!(fp, "<item>
            <title>{}</title>
                <link>{}</link>
                <description>{}</description>
                <pubDate>{}</pubDate>
            </item>",
                         m.title, m.url, m.plot, m.creation_date) {
                Ok(_) => {println!("Successfully written movie item");}
                Err(err) => {println!("Error writing movie item: {}", err.to_string());}
            };
        }

        match write!(fp, "</channel></rss>") {
            Ok(_) => {println!("Successfully written movie list tail");}
            Err(err) => {println!("Error writing movie list tail: {}", err.to_string());}
        };
        fp.flush().unwrap();
    }
}

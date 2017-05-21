extern crate rusqlite;
extern crate chrono;

pub mod db_utils {

    use std::path::Path;
    use chrono::{DateTime,UTC};
    use rusqlite::Connection;

    // TODO: implement Trait to mangle a DateTime from a FromSql
    extern crate time;
    use self::time::Timespec;

    #[derive(Debug)]
    struct Movie {
        id: i32,
        title: String,
        plot: String,
        creation_date: Timespec, // DateTime<UTC>,
        read_date: Option<Timespec>
    }

    impl Default for Movie {
        fn default() -> Movie {
            Movie {id : 0, title : String::new(), plot: String::new(),
                   creation_date : time::get_time(), // UTC::now(),
                   read_date: None}
        }
    }

    pub fn init_db() {
        let db_path = Path::new("/tmp/cinema.db");
        // TODO: if file exists, delete it
        let conn = Connection::open(db_path).unwrap();
        match conn.execute("CREATE TABLE movie (
                          id              INTEGER PRIMARY KEY,
                          title           TEXT NOT NULL,
                          plot            TEXT NOT NULL,
                          creation_date   TEXT NOT NULL,
                          read_date       TEXT NULL)",
                           &[]) {
            Ok(_) => println!("Table created"),
            Err(err) => println!("Table creation failed: {}", err),
        }
    }

    pub fn insert_movie() {
        let db_path = Path::new("/tmp/cinema.db");
        let conn = Connection::open(db_path).unwrap();
        // TODO: use a Default to automatically set creation_date
        let movie = Movie {
            id: 0,
            title: "TestTitle".to_string(),
            plot: "TestPlot".to_string(),
            creation_date: time::get_time(), // UTC::now(),
            read_date: None
        };
        match conn.execute("INSERT INTO movie (id, title, plot, creation_date, read_date)
                      VALUES (?1, ?2, ?3, ?4, ?5)",
                           &[&movie.id, &movie.title, &movie.plot, &movie.creation_date, &movie.read_date]) {
            Ok(inserted) => println!("{} row(s) were inserted", inserted),
            Err(err) => println!("INSERT failed: {}", err),
        }
    }

    pub fn get_movies() {
        let db_path = Path::new("/tmp/cinema.db");
        let conn = Connection::open(db_path).unwrap();
        let mut stmt = conn.prepare("SELECT id, title, plot, creation_date, read_date FROM movie").unwrap();
        let movie_iter = stmt.query_map(&[], |row| {
            Movie {
                id: row.get(0),
                title: row.get(1),
                plot: row.get(2),
                creation_date: row.get(3),
                read_date: row.get(4)
            }
        }).unwrap();
        for movie in movie_iter {
            println!("Found movie {:?}", movie.unwrap());
        }
    }
}

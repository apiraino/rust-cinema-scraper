#[macro_use]
extern crate log;
extern crate clap;
extern crate scraper;
extern crate hyper;

use std::process;
use std::io::Read;
// use std::io::Write;
use std::fs::File;

use log::{LogRecord, LogLevel, LogMetadata, SetLoggerError, LogLevelFilter};
use clap::{App, Arg};
use hyper::Client;
use hyper::header::Connection;
use scraper::{Html, Selector};

static CINEMA_URL : &'static str = "http://visionario.movie";
const LOCAL_DEBUG : bool = false;
struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &LogMetadata) -> bool {
        metadata.level() <= LogLevel::Trace
    }

    fn log(&self, record: &LogRecord) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
        }
    }
}

pub fn init_log() -> Result<(), SetLoggerError> {
    log::set_logger(|max_log_level| {
        max_log_level.set(LogLevelFilter::Info);
        Box::new(SimpleLogger)
    })
}


fn main() {

    match init_log() {
        Ok(_) => println!("logging started."),
        Err(err) => println!("error starting logger: {}", err)
    };

    let matches = App::new("Cinema feed crawler")
        .version("0.1")
        .about("Grab the weekly cinema feed and store into DB")
        .arg(Arg::with_name("date_from")
             .short("f")
             .long("date_from")
             .help("Date FROM to start crawling")
             .takes_value(true))
        .get_matches();
    debug!("Got param: {}", matches.value_of("date_from").unwrap());
    let date = matches.value_of("date_from").unwrap();

    // Get movies for the week
    // "client" is mutable otherwise each time it is used, ownership is moved
    let mut client = Client::new();
    let url = format!("{}/calendario-settimanale/?data={}", CINEMA_URL, date);
    let mut body = String::new();
    if LOCAL_DEBUG
    {
        // Open the file and read content
        let mut f = File::open(format!("{}.html", date)).unwrap();
        f.read_to_string(&mut body).unwrap();
    }
    else
    {
        make_request(&mut client, url, &mut body);
    }

    // "Now you have two problems" (cit. Jamie Zawinski)
    let document = Html::parse_document(&body);
    let mut document_detail;
    let movie_row_sel = Selector::parse("div.singoloFilmInfo").unwrap();
    let title_sel = Selector::parse("div.titoloGriglia").unwrap();
    let director_sel = Selector::parse("div.registaGriglia").unwrap();
    let timetable_sel = Selector::parse("div.orari").unwrap();
    let li_sel = Selector::parse("li").unwrap();
    let href_sel = Selector::parse("a").unwrap();
    for row in document.select(&movie_row_sel) {

        let title_el = row.select(&title_sel).next().unwrap();
        let title = title_el.text().next().unwrap().trim();
        let director_el = row.select(&director_sel).next().unwrap();
        let director = director_el.text().next().unwrap().trim();

        let timetable_el = row.select(&timetable_sel).next().unwrap();
        let mut timetable = String::new();
        for t in timetable_el.select(&li_sel) {
            timetable += t.text().next().unwrap();
            timetable += " ";
        }

        let href_el = row.select(&href_sel).next().unwrap();
        let movie_url = format!("{}", href_el.value().attr("href").unwrap());

        info!("Found title={} director={}, timetable={} link={}",
              title, director, timetable, movie_url);

        // Get movie detail page
        let url = format!("{}{}", CINEMA_URL, movie_url);
        if LOCAL_DEBUG
        {
            let mut f = File::open("movie_detail.html").unwrap();
            f.read_to_string(&mut body).unwrap();
        }
        else
        {
            debug!("Requesting URL: {}", url);
            body = String::from("");
            make_request(&mut client, url, &mut body);
        }
        document_detail = Html::parse_document(&body);

        // retrieve plot
        let movie_plot_sel = Selector::parse("div.plot").unwrap();
        let p_sel = Selector::parse("p").unwrap();
        // let movie_plot_el = document_detail.select(&movie_plot_sel).next().unwrap();
        let movie_plot_el = match document_detail.select(&movie_plot_sel).next() {
            Some(item) => item,
            None => { panic!("Could not retrieve plot from None object"); }
        };

        // retrieve all "plot" divs
        // let mut count = 0;
        // for item in movie_plot_el.select(&p_sel) {
        //     let plot = item.text().next().unwrap_or("");
        //     info!("[{}] plot: {}", count, plot);
        //     count += 1;
        // }

        // retrieve just one "plot" div
        let plot = movie_plot_el.select(&p_sel).nth(1)
            .unwrap()
            .text().next().unwrap_or("Error: could not parse plot");
        info!("plot: {}", plot);
    }
}

fn make_request(client:&mut Client, url:String, body:&mut String) {
    let mut response = client.get(&url)
        .header(Connection::close()).send().unwrap_or_else(|e| {
            error!("Error occurred: {}", e);
            process::exit(1);
        });
    response.read_to_string(body).unwrap();
}

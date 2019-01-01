use env_logger;
use log::{debug, info, warn};

use chrono;
use clap;
use reqwest;
use scraper;

use std::env;
use std::fs::File;
use std::io::Read;

use chrono::prelude::*;
use clap::{App, Arg};
use scraper::{Html, Selector};

mod db_utils;

const LOCAL_DEBUG: bool = false;

fn main() -> Result<(), String> {
    env_logger::init();

    let key = "CINEMA_URL";
    let cinema_url = match env::var(key) {
        Ok(val) => val,
        Err(_) => return Err(format!("Error: could not find CINEMA_URL env var"))
    };

    let matches = App::new("Cinema feed crawler")
        .author(&clap::crate_authors!()[..])
        .version(&clap::crate_version!()[..])
        .about(&clap::crate_description!()[..])
        .arg(
            Arg::with_name("date_from")
                .long("--date-from")
                .help("Date FROM to start crawling")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("feed_path")
                .long("--feed-path")
                .help("Where to save the feed (default: same dir)")
                .takes_value(true),
        )
        .arg_from_usage("--purge-db Delete DB before running")
        .get_matches();
    let date_from = matches.value_of("date_from").unwrap();
    let mut feed_path = format!("{}", matches.value_of("feed_path").unwrap_or_default());
    if feed_path != "" {
        feed_path = format!("{}/", feed_path);
    }
    let purge_db = if matches.is_present("purge-db") {
        true
    } else {
        false
    };

    db_utils::init_db(purge_db);

    // Get movies for this day
    // "client" is mutable otherwise each time it is used, ownership is moved
    // let mut client = Client::new();
    let mut url = format!("{}/calendario-settimanale/?data={}", cinema_url, date_from);
    let mut body = String::new();
    if LOCAL_DEBUG {
        // Open the file and read content
        let mut f = File::open(format!("{}.html", date_from)).unwrap();
        f.read_to_string(&mut body).unwrap();
    } else {
        if let Ok(mut response) = reqwest::get(&url) {
            response.read_to_string(&mut body).unwrap();
        }
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
            timetable += t.text().next().unwrap_or_default();
            timetable += " ";
        }

        let href_el = row.select(&href_sel).next().unwrap();
        let movie_url = format!("{}", href_el.value().attr("href").unwrap());

        info!(
            "Found title={} director={}, timetable={} link={}",
            title, director, timetable, movie_url
        );

        // Get movie detail page
        url = format!("{}{}", cinema_url, movie_url);
        if LOCAL_DEBUG {
            let mut f = File::open("movie_detail.html").unwrap();
            f.read_to_string(&mut body).unwrap();
        } else {
            debug!("Requesting URL: {}", url);
            body = String::from("");
            if let Ok(mut response) = reqwest::get(&url) {
                response.read_to_string(&mut body).unwrap();
            } else {
                warn!("Could not retrieve movie detail page: {}", url);
                continue;
            }
        }
        document_detail = Html::parse_document(&body);

        // retrieve plot
        let movie_plot_sel = Selector::parse("div.plot").unwrap();
        let p_sel = Selector::parse("p").unwrap();
        let movie_plot_el = match document_detail.select(&movie_plot_sel).next() {
            Some(item) => item,
            // if plot is not there, the HTML is wrong (e.g. wrong redirect).
            // Just skip title.
            None => {
                warn!("Could not parse HTML for plot for URL: {}", url);
                continue;
            }
        };

        /*
        let mut count = 0;
        for item in movie_plot_el.select(&p_sel) {
            let plot = item.text().next().unwrap_or("");
            info!("[{}] plot: {}", count, plot);
            count += 1;
        }
        */

        // retrieve just one "plot" div
        let plot = movie_plot_el
            .select(&p_sel)
            .nth(1)
            .unwrap()
            .text()
            .next()
            .unwrap_or("Error: could not parse plot");
        info!("plot: {}", plot);

        // retrieve date of movie publishing
        // ex.: <div class="inizio ">dal 18 maggio 2017</div>
        let movie_publish_sel = Selector::parse("div.inizio").unwrap();
        let movie_publish_el = match document_detail.select(&movie_publish_sel).next() {
            Some(item) => item,
            None => {
                return Err(format!("Could not retrieve publish date from None object"))
            }
        };
        let mut movie_publish =
            String::from(movie_publish_el.text().next().unwrap().trim());
        movie_publish = _fix_date(movie_publish);
        debug!("movie published on: '{}'", movie_publish);
        let pub_date = match Utc.datetime_from_str(movie_publish.as_str(), "%d %B %Y %H:%M:%S") {
            Ok(num) => num,
            Err(err) => return Err(format!("Error on parsing date '{}': {}", movie_publish, err)),
        };

        // add movie to list if not already existing
        let num = db_utils::get_movie(String::from(format!("{}{}", cinema_url, movie_url)));
        if num > 0 {
            continue;
        }
        db_utils::insert_movie(
            String::from(title),
            String::from(director),
            timetable,
            String::from(plot),
            String::from(format!("{}{}", cinema_url, movie_url)),
            pub_date,
        );
    }
    db_utils::get_movies_xml(feed_path);

    Ok(())
}

fn _fix_date(txt: String) -> String {
    // poor man's date translator for RFC2822 compliancy
    return txt
        .replace("dal", "")
        .replace("gennaio", "january")
        .replace("febbraio", "february")
        .replace("marzo", "march")
        .replace("aprile", "april")
        .replace("maggio", "may")
        .replace("giugno", "june")
        .replace("luglio", "july")
        .replace("agosto", "august")
        .replace("settembre", "september")
        .replace("ottobre", "october")
        .replace("novembre", "november")
        .replace("dicembre", "december")
        + " 00:00:00";
}

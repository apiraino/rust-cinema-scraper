### A cinema website scraper in Rust

#### A web crawler that gets the cinema movie list of a specific date, saves it into a DB and exposes an RSS feed for public consumption.

**WARNING: this is not a crate for general use, it serves a very specific usecase: mine. At most it can be some sample code for newbies.**

This is a little more than the usual `Hello, world!` project. It's a first contact with the [Rust programming language](https://www.rust-lang.org), a.k.a [_I don't know what I'm doing_](http://knowyourmeme.com/memes/i-have-no-idea-what-i-m-doing), I just pulled it together applying a lot of google-fu and copy and paste ;-)

However, as any didactical project, it was really useful to learn a lot of things about Rust, its building toolchain, packaging, where to look for help and so on.

#### How does it work
This little application basically scrapes a web page, extracts some data using regular expressions, save the results into a SQLite3 DB and outputs anm RSS 2.0/Atom compliant XML file (that I will instruct my RSS feed reader to retrieve).

#### Limitations

HTML + regex. [Enough said](https://stackoverflow.com/a/1732454).

#### Installation and requirements

* Tested with Rust 1.31 stable
* Install [cargo](https://crates.io) to make your life easier

#### Run

Example: `CINEMA_URL="http://cinema.website.url" cargo run -- --date-from 2017-04-19`
Optional parameters:

 - `--purge-db` delete (if any) local DB before starting
 - `--feed-path` custom RSS feed save path

#### Verbose debugging:

Application and every library:
`RUST_LOG=debug cargo run -- --date-from 2017-04-19`

Applicatiion only:
`RUST_LOG=cinema_feed=debug cargo run -- --date-from 2017-04-19`

DB module only:
`RUST_LOG=db=debug cargo run -- --date-from 2017-04-19`

#![deny(
    warnings,
    missing_debug_implementations,
    rust_2018_idioms,
    nonstandard_style,
    future_incompatible,
    clippy::all,
    clippy::suspicious,
    clippy::style,
    clippy::complexity,
    clippy::perf,
    clippy::pedantic
)]
#![forbid(unsafe_code)]

use std::sync::atomic::AtomicUsize;
use std::time::{Duration, Instant};

use clap::{Parser, ValueEnum};
use futures::{stream, StreamExt};
use reqwest::{Client, StatusCode};
use tokio;

static REQUESTS_MADE: AtomicUsize = AtomicUsize::new(0);

const LONG_ABOUT: &str = "rrlc - rust rate limit checker\n\
                          Spams HTTP requests on a given URL until time \
                          limit is reached or 429 (Too Many Requests) \
                          status code is returned.";

#[derive(Debug, Clone, ValueEnum)]
enum HttpMethod {
    GET,
    POST,
}

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about,
    long_about = LONG_ABOUT
)]
struct CliArgs {
    #[arg()]
    url: String,

    /// For how long to spam requests in seconds
    #[arg()]
    duration: u64,

    #[arg(value_enum)]
    method: HttpMethod,

    /// After what number of requests program will stop
    #[arg(short, default_value_t = 1000)]
    max_requests: usize,

    /// Size of buffer for requestes that are made at the same time
    #[arg(short, default_value_t = 15)]
    concurrent_requests_count: usize,

    /// Disable logging to stdout/stderr
    #[arg(short, long)]
    quiet: bool,
}

#[tokio::main]
async fn main() {
    let args = CliArgs::parse();
    let duration = Duration::from_secs(args.duration);

    let client = Client::new();
    let request_stream = stream::iter([args.url])
        .cycle()
        .take(args.max_requests)
        .map(|url| {
            let builder = match args.method {
                HttpMethod::GET => client.get(url),
                HttpMethod::POST => client.post(url),
            };
            async move {
                let resp = builder.send().await.unwrap();
                (resp.status(), resp.headers().clone())
            }
        })
        .buffer_unordered(args.concurrent_requests_count);

    let start = Instant::now();
    request_stream
        .for_each(move |(status, headers)| async move {
            let requests_count =
                REQUESTS_MADE.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
            println!("{}", status);

            if status == StatusCode::TOO_MANY_REQUESTS {
                println!("Response 429");
                println!("{:?}", headers);
                let stop = Instant::now();
                let duration = stop - start;
                println!("Took {}ms", duration.as_millis());
                let requests_count = REQUESTS_MADE.load(std::sync::atomic::Ordering::SeqCst);
                println!("Requests made = {}.", requests_count);
                std::process::exit(0)
            }
            if Instant::now() - start > duration || requests_count >= args.max_requests {
                println!("Time or request limit reached.");
                let stop = Instant::now();
                let duration = stop - start;
                println!("Took {}ms", duration.as_millis());
                let requests_count = REQUESTS_MADE.load(std::sync::atomic::Ordering::SeqCst);
                println!("Requests made = {}.", requests_count);
                std::process::exit(0)
            }
        })
        .await;
}

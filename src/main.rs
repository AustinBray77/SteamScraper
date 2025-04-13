use std::{env, error::Error};

use searcher::Searcher;
use steam_requester::test_account_build_info;

extern crate reqwest;
extern crate tl;
extern crate tokio;

mod error;
mod heap;
mod log;
mod searcher;
mod steam_requester;
mod util;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let searcher = Searcher::new(
        "https://steamcommunity.com/profiles/76561198258961896",
        "https://steamcommunity.com/id/eltia",
    )
    .await;

    let mut args = env::args();

    args.next();

    let chunk_size = match args.next() {
        Some(size) => size
            .parse::<usize>()
            .expect("Expects first argument to be usize!"),
        None => 1000_usize,
    };

    let max_depth = match args.next() {
        Some(depth) => depth
            .parse::<usize>()
            .expect("Expects second argument to be usize!"),
        None => 10_usize,
    };

    println!("Chunk size: {}, Max depth: {}", chunk_size, max_depth);

    let path = searcher.start_search(max_depth, chunk_size).await?;

    for val in path {
        print!("{} ->", val);
    }

    Ok(())
}

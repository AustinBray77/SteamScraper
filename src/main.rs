use std::error::Error;

use searcher::Searcher;

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
        "https://steamcommunity.com/profiles/76561198138683364",
        "https://steamcommunity.com/profiles/76561198043820228",
    )
    .await;

    let path = searcher.start_search(10).await?;

    for val in path {
        print!("{} ->", val);
    }

    Ok(())
}

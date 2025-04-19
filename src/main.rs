use msg::Message;
use std::{env, error::Error, os::windows::thread};
use tokio::{
    sync::mpsc::{self, Receiver, Sender},
    task::JoinSet,
};

use searcher::Searcher;
use steam_requester::test_account_build_info;

extern crate reqwest;
extern crate tl;
extern crate tokio;

mod error;
mod heap;
mod log;
mod msg;
mod process_runner;
mod searcher;
mod steam_requester;
mod util;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let searcher = Searcher::new(
        "https://steamcommunity.com/profiles/76561198258961896",
        "https://steamcommunity.com/id/Undeadnemesiss",
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

    let (sender, reciever) = mpsc::channel::<Message>(100);

    let mut thread_task: JoinSet<()> = JoinSet::new();

    thread_task.spawn(search(searcher, max_depth, chunk_size, reciever));

    thread_task.spawn(get_input(sender));

    thread_task.join_all().await;

    //search(searcher, max_depth, chunk_size, reciever).await;

    Ok(())
}

async fn get_input(sender: Sender<Message>) {
    use std::io::stdin;

    let input_reader = stdin();

    loop {
        let mut input = String::new();

        let _ = input_reader.read_line(&mut input);

        println!("Read input: {}", input);

        let message = match input.trim() {
            "q" => Message::Quit,
            "p" => Message::Pause,
            "c" => Message::Continue,
            _ => Message::None,
        };

        println!("Sending: {:?}", message);

        match sender.send(message.clone()).await {
            Err(err) => {
                println!("{}", err.to_string());
                println!("Exiting...");
                break;
            }
            _ => {}
        }

        match message {
            Message::Quit => {
                break;
            }
            _ => {}
        }
    }
}

async fn search(
    searcher: Searcher,
    max_depth: usize,
    chunk_size: usize,
    reciever: Receiver<Message>,
) {
    let path_result = searcher.start_search(max_depth, chunk_size, reciever).await;

    match path_result {
        Ok(mut path) => {
            let last = path.pop().unwrap();

            for val in path {
                print!("{} <-> ", val);
            }

            print!("{}\n", last);
        }
        Err(err) => {
            println!("{:?}", err);
        }
    }
}

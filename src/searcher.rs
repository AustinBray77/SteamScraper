use std::collections::HashMap;
use std::error::Error;
use std::sync::mpsc::SendError;
use std::sync::Arc;

use tokio::task::JoinSet;

use crate::heap::{MaxHeap, Order};
use crate::steam_requester::{build_account_info, get_friends, score_account_overlap, AccountInfo};
use crate::util::{time_fn, time_fn_async, unzip_tuple_lists};

type HeapItem = (String, String, f32);
type Heap = MaxHeap<HeapItem, String>;

pub struct Searcher {
    source: &'static str,
    dest_account_info: AccountInfo,
    target_link: &'static str,
}

impl Searcher {
    pub async fn new(source: &'static str, target_link: &'static str) -> Self {
        let account_info = build_account_info(target_link.to_string()).await.unwrap();

        Searcher {
            source: source,
            dest_account_info: account_info,
            target_link: target_link,
        }
    }

    fn cmp(item_a: &HeapItem, item_b: &HeapItem) -> Order {
        let score1 = (*item_a).2;
        let score2 = (*item_b).2;

        if score1 > score2 {
            Order::Greater
        } else if score1 == score2 {
            Order::Equal
        } else {
            Order::Smaller
        }
    }

    fn key(item: &HeapItem) -> &String {
        &item.1
    }

    async fn score_friend(
        name: String,
        link: String,
        dst_account_info: &AccountInfo,
    ) -> Option<(String, String, f32)> {
        let next_account = match build_account_info(link.clone()).await {
            Ok(val) => val,
            Err(_) => return None,
        };

        if next_account.private {
            return None;
        }

        let score = score_account_overlap(&dst_account_info, &next_account);

        Some((name, link, score))
    }

    async fn search_node_with_score(
        preds: &HashMap<String, String>,
        person: String,
        person_link: String,
        _max_depth: usize,
        dst_account_info: Arc<AccountInfo>,
    ) -> (Heap, HashMap<String, String>) {
        let mut new_queue = MaxHeap::new(Self::cmp, Self::key);
        let mut new_preds: HashMap<String, String> = HashMap::new();

        let f_names_and_links = match get_friends(person_link.clone()).await {
            Ok(val) => val,
            Err(_) => return (new_queue, new_preds),
        };

        let mut score_friends_tasks: JoinSet<Option<(String, String, f32)>> = JoinSet::new();

        for (name, link) in f_names_and_links {
            if let Some(_) = preds.get(&(name.clone() + "," + link.clone().as_str())) {
                //println!("Skipping: {}", name);
                continue;
            }

            let account_ref = Arc::clone(&dst_account_info);

            score_friends_tasks
                .spawn(async move { Self::score_friend(name, link, &account_ref).await });
        }

        let results = score_friends_tasks.join_all().await;

        results
            .into_iter()
            .filter_map(|x| x)
            .for_each(|(name, link, score)| {
                new_queue.insert((name.clone(), link.clone(), score));
                new_preds.insert(name + "," + link.as_str(), person.clone());
            });

        (new_queue, new_preds)
    }

    fn collect_batch(
        &self,
        queue: &Heap,
        preds: &HashMap<String, String>,
        batch_size: usize,
        max_depth: usize,
    ) -> (JoinSet<(Heap, HashMap<String, String>)>, Heap) {
        let mut task_set = JoinSet::new();
        let shared_path = Arc::new(preds.clone());
        let shared_account_info = Arc::new(self.dest_account_info.clone());

        let mut new_queue = queue.clone();

        for (person, friends_link, score) in new_queue.pop_many(batch_size) {
            if friends_link == self.target_link.to_string() {
                return (JoinSet::new(), new_queue);
            }

            println!("Examining: {}, {} : {}", person, friends_link, score);

            let person_owned = person.clone();
            let link_owned = friends_link.clone();
            let path_ref = Arc::clone(&shared_path);
            let account_ref = Arc::clone(&shared_account_info);

            task_set.spawn(async move {
                Self::search_node_with_score(
                    &path_ref,
                    person_owned,
                    link_owned + "/friends/",
                    max_depth,
                    account_ref,
                )
                .await
            });
        }

        (task_set, new_queue)
    }

    pub async fn start_search(
        &self,
        max_depth: usize,
        batch_size: usize,
    ) -> Result<Vec<String>, Box<dyn Error>> {
        let mut queue: Heap = MaxHeap::new(Self::cmp, Self::key);

        let mut preds: HashMap<String, String> = HashMap::new();

        queue.insert((String::from("START"), self.source.to_string(), 0_f32));

        loop {
            let (current_run, new_queue) = time_fn(
                || self.collect_batch(&queue, &preds, batch_size, max_depth),
                "Collecting Runs",
            );

            let (queues, paths) = time_fn_async(
                || async { unzip_tuple_lists(current_run.join_all().await) },
                "Running...",
            )
            .await;

            queue = new_queue;

            queue = time_fn(
                || {
                    queues
                        .iter()
                        .fold(queue.clone(), |acc, next| acc.combine_with(next))
                },
                "Combining Heaps",
            );

            println!("Heap Size: {}", queue.len());

            queue.truncate(100000);

            if queue.len() == 0 {
                let mut path = vec![self.target_link.to_string()];
                let mut cur = self.target_link.to_string();

                while let Some(pred) = preds.get(&cur) {
                    if pred.clone() == String::new() {
                        break;
                    }

                    path.push(pred.clone());
                    cur = pred.clone();
                }

                return Ok(path);
            }

            preds = time_fn(
                || {
                    paths
                        .clone()
                        .into_iter()
                        .fold(preds.clone(), |mut acc, next| {
                            acc.extend(next);
                            acc
                        })
                },
                "Combining Preds",
            );

            //println!("Preds: {:?}", preds);

            println!(
                "End pred: {:?}",
                preds.get(&String::from(
                    "Dr.Disrepect,https://steamcommunity.com/profiles/76561198043820228"
                ))
            );

            println!("Complete iteration");
        }
    }
}

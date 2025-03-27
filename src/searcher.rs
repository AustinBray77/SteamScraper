use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;

use tokio::task::JoinSet;

use crate::heap::{MaxHeap, Order};
use crate::steam_requester::{build_account_info, get_friends, score_account_overlap, AccountInfo};
use crate::util::{time_fn, time_fn_async, unzip_tuple_lists};

type HeapCompare = fn(&(String, String, f32), &(String, String, f32)) -> Order;

pub struct Searcher {
    source: &'static str,
    source_account_info: AccountInfo,
    target_link: &'static str,
    cmp_fn: fn(&(String, String, f32), &(String, String, f32)) -> Order,
}

impl Searcher {
    pub async fn new(source: &'static str, target_link: &'static str) -> Self {
        let account_info = build_account_info(source.to_string()).await.unwrap();

        Searcher {
            source: source,
            source_account_info: account_info,
            target_link: target_link,
            cmp_fn: |(_, _, score1ptr), (_, _, score2ptr)| {
                let score1 = *score1ptr;
                let score2 = *score2ptr;

                if score1 > score2 {
                    Order::Greater
                } else if score1 == score2 {
                    Order::Equal
                } else {
                    Order::Smaller
                }
            },
        }
    }

    async fn search_node_with_score(
        preds: &HashMap<String, String>,
        person: String,
        person_link: String,
        _max_depth: usize,
        cmp_fn: HeapCompare,
        source_account_info: &AccountInfo,
    ) -> (
        MaxHeap<(String, String, f32), HeapCompare>,
        HashMap<String, String>,
    ) {
        let mut new_queue = MaxHeap::new(cmp_fn);
        let mut new_preds: HashMap<String, String> = HashMap::new();

        let f_names_and_links = match get_friends(person_link.clone()).await {
            Ok(val) => val,
            Err(_) => return (new_queue, new_preds),
        };

        for (name, link) in f_names_and_links {
            if let Some(_) = preds.get(&name) {
                continue;
            }

            let next_account = match build_account_info(link.clone()).await {
                Ok(val) => val,
                Err(_) => return (MaxHeap::new(cmp_fn), HashMap::new()),
            };

            let score = score_account_overlap(&source_account_info, &next_account).await;

            new_queue.insert((name.clone(), link, score));
            new_preds.insert(name, person.clone());
        }

        (new_queue, new_preds)
    }

    fn collect_batch(
        &self,
        queue: &MaxHeap<
            (String, String, f32),
            fn(&(String, String, f32), &(String, String, f32)) -> Order,
        >,
        preds: &HashMap<String, String>,
        batch_size: usize,
        max_depth: usize,
    ) -> JoinSet<(
        MaxHeap<(String, String, f32), fn(&(String, String, f32), &(String, String, f32)) -> Order>,
        HashMap<String, String>,
    )> {
        let mut task_set = JoinSet::new();
        let shared_path = Arc::new(preds.clone());
        let shared_account_info = Arc::new(self.source_account_info.clone());

        for (person, friends_link, _) in queue.clone().pop_many(batch_size) {
            if person == self.target_link.to_string() {
                return JoinSet::new();
            }

            let person_owned = person.clone();
            let link_owned = friends_link.clone();
            let path_ref = Arc::clone(&shared_path);
            let account_ref = Arc::clone(&shared_account_info);
            let cloned_fn = (self.cmp_fn).clone();

            task_set.spawn(async move {
                Self::search_node_with_score(
                    &path_ref,
                    person_owned,
                    link_owned + "/friends/",
                    max_depth,
                    cloned_fn,
                    &account_ref,
                )
                .await
            });
        }

        task_set
    }

    pub async fn start_search(&self, max_depth: usize) -> Result<Vec<String>, Box<dyn Error>> {
        let mut queue: MaxHeap<
            (String, String, f32),
            fn(&(String, String, f32), &(String, String, f32)) -> Order,
        > = MaxHeap::new(self.cmp_fn);

        let mut preds: HashMap<String, String> = HashMap::new();

        queue.insert((String::from("START"), self.source.to_string(), 0_f32));

        loop {
            let current_run = self.collect_batch(&queue, &preds, 1000, max_depth);

            let (queues, paths) = unzip_tuple_lists(current_run.join_all().await);

            queue = queues
                .iter()
                .fold(queue.clone(), |acc, next| acc.combine_with(next));

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

            preds = paths
                .clone()
                .into_iter()
                .fold(HashMap::new(), |mut acc, next| {
                    acc.extend(next);
                    acc
                });

            println!("Complete iteration");
        }
    }
}

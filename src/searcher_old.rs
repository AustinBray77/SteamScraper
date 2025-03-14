async fn search_node(
    preds: &HashMap<String, String>,
    person: String,
    friends_link: String,
    max_depth: usize,
) -> (HashSet<(String, String)>, HashMap<String, String>) {
    //println!("{}, {}", person, friends_link);

    let f_names_and_links = match get_friends(friends_link).await {
        Ok(val) => val,
        Err(_) => return (HashSet::new(), HashMap::new()),
    };

    /*
    Fix this somehow with pred array
    if current_path.len() >= max_depth {
        return (HashSet::new(), HashMap::new()); //Err(SteamError::boxed_new("Max Depth Exceeded"));
    } */

    let mut new_queue = HashSet::new();
    let mut new_preds = HashMap::new();

    for (name, link) in f_names_and_links {
        if let Some(_) = preds.get(&name) {
            continue;
        }

        new_queue.insert((name.clone(), link));
        new_preds.insert(name, person.clone());
    }

    (new_queue, new_preds)
}

fn collect_tasks(
    queue: &HashSet<(String, String)>,
    preds: &HashMap<String, String>,
    max_depth: usize,
    to: &'static str,
) -> JoinSet<(HashSet<(String, String)>, HashMap<String, String>)> {
    let mut task_set = JoinSet::new();
    let shared_path = Arc::new(preds.clone());

    for (cur_person, friends_link) in queue {
        let person_owned = cur_person.clone();
        let link_owned = friends_link.clone();
        let path_ref = Arc::clone(&shared_path);

        if person_owned == to.to_string() {
            return JoinSet::new();
        }

        task_set.spawn(
            async move { search_node(&path_ref, person_owned, link_owned, max_depth).await },
        );
    }

    task_set
}

async fn find_shortest_path_between_people(
    from: &'static str,
    to: &'static str,
    max_depth: usize,
) -> Result<Vec<String>, Box<dyn Error>> {
    let mut queue: HashSet<(String, String)> = HashSet::new();
    let mut preds: HashMap<String, String> = HashMap::new();

    queue.insert((String::from("START"), from.to_string() + "/friends/"));
    preds.insert(String::from("START"), String::new());

    loop {
        let current_run = time_fn(
            || collect_tasks(&queue, &preds, max_depth, to),
            "Collect Runs",
        );

        println!("Links to check: {}", queue.len());

        let (queues, paths) = time_fn_async(
            || async { unzip_tuple_lists(current_run.join_all().await) },
            "Execute All",
        )
        .await;

        queue = time_fn(
            || {
                queues
                    .clone()
                    .into_iter()
                    .fold(HashSet::new(), |mut acc, next| {
                        acc.extend(next);
                        acc
                    })
            },
            "Merging Queues",
        );

        if queue.len() == 0 {
            let mut path = vec![to.to_string()];
            let mut cur = to.to_string();

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
                    .fold(HashMap::new(), |mut acc, next| {
                        acc.extend(next);
                        acc
                    })
            },
            "Merging Paths",
        );

        //println!("{:?}", path);

        println!("Complete iteration");
    }
}

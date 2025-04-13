use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::time::Instant;

pub fn combine_tuple_lists<T: Clone, U: Clone>(a: Vec<T>, b: Vec<U>) -> Vec<(T, U)> {
    a.clone()
        .into_iter()
        .zip(b.clone().into_iter())
        .collect::<Vec<(T, U)>>()
}

pub fn unzip_tuple_lists<T: Clone, U: Clone>(a: Vec<(T, U)>) -> (Vec<T>, Vec<U>) {
    a.clone().into_iter().unzip()
}

pub fn join_maps_on_shortest<T: Clone + Eq + Hash>(
    a: &HashMap<T, Vec<T>>,
    b: &HashMap<T, Vec<T>>,
) -> HashMap<T, Vec<T>> {
    let mut new_map: HashMap<T, Vec<T>> = a.clone();

    for (key, vec_b) in b.iter() {
        if let Some(vec_a) = a.get(key) {
            if vec_b.len() < vec_a.len() {
                new_map.insert(key.to_owned(), vec_b.to_vec());
            }
        } else {
            new_map.insert(key.to_owned(), vec_b.to_vec());
        }
    }

    new_map
}

pub fn maps_have_intersection<T: Clone + Eq + Hash, U: Clone>(
    a: &HashMap<T, U>,
    b: &HashMap<T, U>,
) -> bool {
    a.clone()
        .into_keys()
        .filter(|val| b.contains_key(val))
        .next()
        != None
}

pub fn time_fn<T, F: Fn() -> T>(function: F, title: &'static str) -> T {
    println!("Starting: {}", title);

    let start = Instant::now();
    let result = function();
    let duration = start.elapsed();

    println!("Time: {:?}", duration);

    result
}

pub async fn time_fn_async<T, F: FnOnce() -> Fut, Fut: std::future::Future<Output = T>>(
    function: F,
    title: &'static str,
) -> T {
    println!("Starting: {}", title);

    let start = Instant::now();
    let result = function().await;
    let duration = start.elapsed();

    println!("Time: {:?}", duration);

    result
}

pub fn round(val: f32, places: usize) -> f32 {
    let digit_mult = 10_f32.powi(places as i32);
    (val * digit_mult).round() / digit_mult
}

pub fn print_and_return<T: Debug>(val: T) -> T {
    println!("{:?}", val);
    val
}

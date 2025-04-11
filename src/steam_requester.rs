use core::num;
use std::default;
use std::{error::Error, fmt::Display};

use tl::{Node, NodeHandle, Parser};

use crate::error::SteamError;
use crate::util::{combine_tuple_lists, round};

use std::collections::HashSet;

#[derive(Debug, Clone, Default)]
pub struct AccountInfo {
    pub name: String,
    pub recent_games: HashSet<String>,
    pub favorite_game: String,
    pub country: String,
    pub num_friends: f32,
    pub private: bool,
}

pub async fn test_account_build_info() {
    let account = build_account_info(String::from(
        "https://steamcommunity.com/profiles/76561198043820228",
    ))
    .await
    .unwrap();

    println!("{:?}", account);
}

pub fn score_account_overlap(base_account: &AccountInfo, scored_account: &AccountInfo) -> f32 {
    let norm_size = base_account.recent_games.len();

    let inter_size = base_account
        .recent_games
        .intersection(&(scored_account.recent_games))
        .collect::<HashSet<&String>>()
        .len();

    let recent_games_score = inter_size as f32 / norm_size as f32;

    let country_score = if base_account.country == scored_account.country {
        1_f32
    } else {
        0_f32
    };

    let fav_game_score = if base_account.favorite_game == scored_account.favorite_game
        && base_account.favorite_game != String::new()
    {
        1_f32
    } else {
        0_f32
    };

    let friend_score = scored_account
        .num_friends
        .log(1000_f32)
        .clamp(0_f32, 1000_f32);

    round(
        0.3_f32 * country_score
            + 0.3_f32 * recent_games_score
            + 0.3 * fav_game_score
            + 0.5 * friend_score,
        2,
    )
}

fn extract_child_text<'a>(node: &Node<'a>, parser: &Parser<'a>) -> Option<String> {
    let children = node.children()?;

    let child_text_combined = children.top().iter().fold(String::new(), |acc, next| {
        if let Some(node) = next.get(parser) {
            acc + node.inner_text(parser).to_string().as_str()
        } else {
            acc
        }
    });

    Some(child_text_combined.trim().to_string())
}

pub async fn build_account_info(link: String) -> Result<AccountInfo, Box<dyn Error>> {
    let raw_page = profile_from_link(link).await?;

    let parse_options = tl::ParserOptions::default();

    let dom = tl::parse(raw_page.as_str(), parse_options)?;

    let parser = dom.parser();

    let private_field = dom
        .get_elements_by_class_name("profile_private_info")
        .filter_map(|node| node.get(parser))
        .last();

    if let Some(_) = private_field {
        return Ok(AccountInfo {
            private: true,
            ..Default::default()
        });
    }

    let recent_games = dom
        .get_elements_by_class_name("game_name")
        .filter_map(|node| node.get(parser))
        .filter_map(|node| extract_child_text(node, parser))
        .collect::<HashSet<String>>();

    let name = dom
        .get_elements_by_class_name("actual_persona_name")
        .filter_map(|node| node.get(parser))
        .map(|node| node.inner_text(parser).to_string())
        .next()
        .unwrap_or(String::from("NAME NOT FOUND"));

    let country = dom
        .get_elements_by_class_name("header_real_name")
        .filter_map(|node| node.get(parser))
        .map(|node| node.inner_text(parser).to_string())
        .next()
        .unwrap_or(String::from("COUNTRY NOT FOUND"))
        .trim()
        .split(',')
        .last()
        .unwrap_or("DID NOT WORK!")
        .trim()
        .to_string();

    let favorite_game = dom
        .get_elements_by_class_name("showcase_item_detail_title")
        .filter_map(|node| node.get(parser))
        .filter_map(|node| extract_child_text(node, parser))
        .next()
        .map(|x| x.clone().trim().to_string())
        .unwrap_or(String::new());

    let num_friends = dom
        .get_elements_by_class_name("profile_count_link")
        .filter_map(|node| node.get(parser))
        .filter_map(|node| node.children())
        .filter_map(|children| children.all(parser).get(1))
        .filter_map(|node| extract_child_text(node, parser))
        .filter(|text| text.contains("Friends"))
        .next()
        .map(|x| x.clone().trim().to_string())
        .map(|x| x.replace("\t", "").replace("\n", "").replace(" ", ""))
        .map(|x| x.split(';').last().unwrap_or("").to_string())
        .unwrap_or(String::new())
        .parse::<i32>()
        .unwrap_or(0);

    Ok(AccountInfo {
        name: name,
        recent_games: recent_games,
        favorite_game: favorite_game,
        country: country,
        num_friends: num_friends as f32,
        private: false,
    })
}

pub async fn get_profile_info(id: &'static str) -> Result<String, Box<dyn Error>> {
    let raw_page = profile_from_id(id).await?;

    let parse_options = tl::ParserOptions::default();

    let dom = tl::parse(raw_page.as_str(), parse_options)?;

    let parser = dom.parser();

    let elements: Vec<NodeHandle> = dom
        .get_elements_by_class_name("actual_persona_name")
        .collect();

    let user_name_node = match elements.first() {
        Some(node) => node,
        None => {
            return Err(SteamError::boxed_new("User name could not be parsed"));
        }
    };

    let inner_user_name = match user_name_node.get(parser) {
        Some(node) => node,
        None => return Err(SteamError::boxed_new("Inner parsing failed")),
    };

    Ok(inner_user_name.inner_text(parser).to_string())
}

pub fn get_href_from_node<'a>(node: &Node<'a>) -> Option<String> {
    let tag = node.as_tag()?;

    let link = tag.attributes().get("href")??;

    Some(link.as_utf8_str().to_string())
}

pub async fn get_friends(link: String) -> Result<Vec<(String, String)>, Box<dyn Error>> {
    let raw_friends = raw_friends_page(link + "/friends/").await?;

    let parse_options = tl::ParserOptions::default();

    let dom = tl::parse(raw_friends.as_str(), parse_options)?;

    let parser = dom.parser();

    let friend_links = dom
        .get_elements_by_class_name("selectable_overlay")
        .filter_map(|node| node.get(parser))
        .filter_map(get_href_from_node)
        .collect::<Vec<String>>();

    let friend_names = dom
        .get_elements_by_class_name("friend_block_content")
        .filter_map(|node| node.get(parser))
        .map(|node| node.inner_html(parser).to_string())
        .filter_map(|content| extract_friend_name(content))
        .collect::<Vec<String>>();

    Ok(combine_tuple_lists(friend_names, friend_links))
}

pub fn extract_friend_name<'a>(content: String) -> Option<String> {
    content.split('<').next().map(|str| str.trim().to_string())
}

pub async fn raw_friends_page(link: String) -> Result<String, Box<dyn Error>> {
    let friends_page = reqwest::get(link).await?.text().await?;

    Ok(friends_page)
}

async fn profile_from_link(link: String) -> Result<String, Box<dyn Error>> {
    let raw_page = reqwest::get(link).await?.text().await?;

    Ok(raw_page)
}

pub async fn profile_from_id(id: &'static str) -> Result<String, Box<dyn Error>> {
    let profile_link = format!("https://steamcommunity.com/profiles/{}", id);

    profile_from_link(profile_link).await
}

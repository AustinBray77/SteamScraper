use std::{error::Error, fmt::Display};

use tl::{Node, NodeHandle};

use crate::error::SteamError;
use crate::util::combine_tuple_lists;

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
    let raw_friends = raw_friends_page(link).await?;

    let parse_options = tl::ParserOptions::default();

    let dom = tl::parse(raw_friends.as_str(), parse_options)?;

    let parser = dom.parser();

    let friend_links = dom
        .get_elements_by_class_name("selectable_overlay")
        .filter_map(|node| node.get(parser))
        .filter_map(get_href_from_node)
        .map(|str| str + "/friends/")
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

pub fn get_friends_link<T: Display>(id: T) -> String {
    format!("https://steamcommunity.com/profiles/{}/friends", id)
}

pub async fn raw_friends_page(link: String) -> Result<String, Box<dyn Error>> {
    let friends_page = reqwest::get(link).await?.text().await?;

    Ok(friends_page)
}

pub async fn profile_from_id(id: &'static str) -> Result<String, Box<dyn Error>> {
    let profile_link = format!("https://steamcommunity.com/profiles/{}", id);

    let profile = reqwest::get(profile_link).await?.text().await?;

    Ok(profile)
}

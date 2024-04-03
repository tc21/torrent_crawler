use colored::Colorize;
use scraper::{Html, Selector, ElementRef};

use crate::Args;


#[derive(Debug)]
pub struct Entry {
    pub title: String,
    pub magnet_link: String,
}

const MAX_PAGES_TO_SEARCH: usize = 10;

pub async fn find_with_args(args: &Args) -> Result<Vec<Entry>, Box<dyn std::error::Error>> {
    for page_number in 1..=MAX_PAGES_TO_SEARCH {
        eprintln!("searching page {}", page_number);
        let page = get_page(&args.name, page_number)
            .await?;

        let rows = find_in_page(&args, &page);

        if rows.len() == 0 && has_next_page(&page) {
            continue
        }

        return Ok(rows)
    }

    Ok(vec![])
}

fn find_in_page(args: &Args, page: &Html) -> Vec<Entry> {
    let row_selector = Selector::parse(".torrent-list > tbody > tr").unwrap();

    page.select(&row_selector)
        .map(|row| parse_row(&row))
        .filter_map(|row| ok_or_print_error(row))
        .filter(|row| matches(&row.title, &args.episode))
        .collect::<Vec<_>>()
}

fn ok_or_print_error<V, E: AsRef<str>>(result: Result<V, E>) -> Option<V> {
    match result {
        Ok(v) => Some(v),
        Err(e) => {
            eprintln!("{}", e.as_ref().red());
            None
        },
    }
}

fn has_next_page(page: &Html) -> bool {
    let selector = Selector::parse(".pagination-page-info").unwrap();

    let text = match page.select(&selector).next() {
        Some(e) => e.html(),
        None => return false,
    };

    let filtered = text.chars().map(|c| if c.is_numeric() { c } else { ' ' }).collect::<String>();
    let numbers = filtered.trim()
        .split_ascii_whitespace()
        .take(3)
        .map(|s| s.parse::<i32>())
        .collect::<Result<Vec<_>, _>>();

    match numbers {
        Ok(numbers) => numbers.len() >= 3 && numbers[2] > numbers[1],
        Err(_) => false,
    }
}

fn matches(title: &str, episode: &Option<String>) -> bool {
    match episode {
        Some(e) => title.contains(&format!(" - {:0>2}", e)) || title.contains(&format!("E{:0>2}", e)),
        None => true,
    }
}

async fn get_page(search_string: &str, page: usize) -> Result<Html, Box<dyn std::error::Error>> {
    let url = format!("https://nyaa.si/?f=0&c=0_0&q={}&p={}", search_string, page);

    let page = reqwest::get(url)
        .await?
        .text()
        .await?;

    Ok(Html::parse_document(&page))
}

fn parse_row(e: &ElementRef) -> Result<Entry, &'static str> {
    let title_selector = Selector::parse("a:not(.comments)[href^='/view/']").unwrap();
    let link_selector = Selector::parse("a[href^='magnet:']").unwrap();

    let title = e.select(&title_selector).next()
        .and_then(|e| e.value().attr("title"))
        .ok_or("could not find title")?
        .to_string();

    let magnet_link = e.select(&link_selector).next()
        .and_then(|e| e.value().attr("href"))
        .ok_or("could not find link")?
        .to_string();

    Ok(Entry { title, magnet_link })
}

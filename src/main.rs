use clap::{Parser, Subcommand};
use sql::Show;

use crate::sql::Episode;

mod crawler;
mod config;
mod sql;
mod exec;

#[derive(Parser, Debug)]
/// Retrieve links to particular episodes of a particular show from nyaa.si
pub struct Args {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long)]
    run_in: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Refresh,
    Status {
        #[arg(short, long)]
        title: Option<String>
    },
    Insert {
        #[arg(short, long)]
        title: String,

        #[arg(short, long)]
        search: Option<String>,

        #[arg(short, long)]
        next: Option<i32>,

        #[arg(short, long, id = "episodes")]
        episodes: Option<i32>,

        #[arg(short, long, action)]
        update: bool
    },
    Complete {
        #[arg(short, long)]
        title: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let args = Args::parse();
    let config_dir = args.run_in.unwrap_or(".".to_string());
    let settings = config::Config::read(&config_dir)?;

    match args.command {
        Commands::Refresh => {
            let shows = sql::search_title(&config_dir, "", false)
                .map_err(|e| format!("failed to get titles: {}", e))?;

            for show in shows {
                println!("refreshing {}...", show.title);
                let search_string =  match &show.search_string {
                    Some(s) => s,
                    None => &show.title
                };

                let episodes = match crawler::find(&search_string, show.next_episode).await {
                    Ok(e) => e,
                    Err(e) => {
                        println!("an error occurred: {:?}", e);
                        continue;
                    }
                };

                if episodes.is_empty() {
                    continue;
                }

                println!("found the following episodes, selecting the first one: {:?}", episodes);
                let episode = episodes.into_iter().next().unwrap();
                let episode = Episode {
                    title: show.title.clone(),
                    episode: show.next_episode,
                    url: episode.magnet_link
                };
                exec::new_episode(&episode, &settings.on_new_episode);

                println!("updating db...");
                let mut updated = Show { ..show };
                updated.next_episode += 1;
                sql::insert(&config_dir, &updated)
                    .map_err(|e| format!("failed to update show: {}", e))?;
                sql::insert_episode(&config_dir, &episode)
                    .map_err(|e| format!("failed to insert episode: {}", e))?;
            }

        },
        Commands::Status { title } => {
            let episodes = if let Some(title) = title {
                sql::search_title(&config_dir, &title, true)
            } else {
                sql::search_title(&config_dir, "", true)
            }.map_err(|e| format!("{}", e))?;

            dbg!(episodes);
        },
        Commands::Insert { title, mut search, mut next, mut episodes, update } => {
            if update {
                let existing = sql::search_title(&config_dir, &title, true).unwrap()
                    .into_iter()
                    .filter(|s| s.title.eq_ignore_ascii_case(&title))
                    .next()
                    .expect("title not found");

                if search.is_none() {
                    search = existing.search_string
                }

                if next.is_none() {
                    next = Some(existing.next_episode)
                }

                if episodes.is_none() {
                    episodes = Some(existing.next_episode)
                }
            }

            let show = Show {
                title,
                search_string: search,
                next_episode: next.unwrap_or(1),
                total_episodes: episodes.unwrap_or(-1)
            };

            let updated = sql::insert(&config_dir, &show)
                .map_err(|e| format!("{}", e))?;

            println!("{} item(s) updated", updated)
        },
        Commands::Complete { title } => {
            let updated = sql::complete(&config_dir, &title)
                .map_err(|e| format!("{}", e))?;

                println!("{} item(s) updated", updated)
        }
    }

    Ok(())
}

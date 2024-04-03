use clap::Parser;

mod downloader;

#[derive(Parser, Debug)]
/// Retrieve links to particular episodes of a particular show from nyaa.si
pub struct Args {
    name: String,
    #[arg(short, long)]
    /// leave empty for any episode
    episode: Option<String>,
    #[arg(long)]
    /// if set, will try to encode magnet links as hyperlinks in your terminal
    encode_links: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let rows = downloader::find_with_args(&args).await?;

    if rows.len() > 0 {
        for row in rows {
            if args.encode_links {
                println!(
                    "\x1b]8;;{link}\x1b\\{text}\x1b]8;;\x1b\\",
                    link = row.magnet_link,
                    text = row.title
                )
            } else {
                println!("{}", row.title);
                println!("> {}", row.magnet_link);
            }
        }
    } else {
        println!("could not find anything")
    }

    Ok(())
}

use clap::Parser;
use transmission_rpc::{types::TorrentAddArgs, TransClient};

#[derive(Debug, Parser)]
pub struct Args {
    server: String,
    torrent: String
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let mut client = TransClient::new(
        args.server.parse().unwrap(),
    );

    let response = client.torrent_add(TorrentAddArgs {
        filename: Some(args.torrent.to_owned()),
        ..Default::default()
    }).await.unwrap();

    dbg!(response);
}

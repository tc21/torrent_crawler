use std::process;

use crate::{config::Command, sql::Episode};

pub fn new_episode(episode: &Episode, commands: &Vec<Command>) {
    for command in commands {
        let args = command.args.iter()
            .map(|arg| replace_strings(arg, episode))
            .collect::<Vec<_>>();

        println!("executing {} with args {:?}", command.command, command.args);

        match process::Command::new(&command.command)
            .args(args)

        .spawn() {
            Ok(child) => std::mem::forget(child),
            Err(e) => println!("failed to spawn child process: {}", e),
        }
    }
}

fn replace_strings(arg: &str, episode: &Episode) -> String {
    arg.replace("$title", &episode.title)
        .replace("$episode", &format!("{}", episode.episode))
        .replace("$url", &episode.url)
}

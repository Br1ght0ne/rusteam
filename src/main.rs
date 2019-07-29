extern crate dirs;
extern crate structopt;
use rusteam::Rusteam;
use structopt::StructOpt;

#[derive(StructOpt)]
enum CLI {
    #[structopt(name = "ls", about = "List your games")]
    List {
        #[structopt(help = "substring of game name")]
        pattern: Option<String>,
    },
    #[structopt(name = "play", about = "Run a game")]
    Play {
        #[structopt(help = "substring of game name")]
        pattern: String,
    },
}

fn main() {
    let cli = CLI::from_args();
    let root = dirs::home_dir().unwrap().join("Games");

    match cli {
        CLI::List { pattern } => {
            let games = Rusteam::list_games(&root, pattern);
            for game in games.iter() {
                println!("{}", game);
            }
        }
        CLI::Play { pattern } => Rusteam::play_game(&root, pattern),
    }
}

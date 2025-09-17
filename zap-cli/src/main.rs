use clap::Command;

mod cmd;
mod config;

fn create_clap_app() -> Command {
    cmd::build::add_build_args(
        Command::new("zap")
            .version(env!("CARGO_PKG_VERSION"))
            .about("Get a website for your project in seconds, with no configuration")
            .author("Javier Feliz <me@javierfeliz.com>")
    )
    .subcommand(cmd::build::make_subcommand())
    .subcommand(cmd::serve::make_subcommand())
    .subcommand(
        Command::new("version")
            .about("Show version information")
    )
}

#[tokio::main]
async fn main() {
    let matches = create_clap_app().get_matches();

    let result = match matches.subcommand() {
        Some(("build", sub_matches)) => cmd::build::execute(sub_matches),
        Some(("serve", sub_matches)) => cmd::serve::execute(sub_matches).await,
        Some(("version", _)) => {
            println!("zap {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        None => {
            // No subcommand provided, run build with the root arguments
            cmd::build::execute(&matches)
        }
        _ => unreachable!(),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

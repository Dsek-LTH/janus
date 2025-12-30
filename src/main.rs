use clap::Parser;
use std::io;

mod discord;
mod dsek;
mod env;
mod register;
mod server;
mod storage;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = false)]
    register: bool,
    #[arg(short, long, default_value_t = false)]
    setup_db: bool,
}

fn main() {
    let args = Args::parse();

    if args.register {
        register::start();
    } else if args.setup_db {
        let mut input = String::new();

        println!(
            "This will wipe the database at {}, are you sure you want to do this? (y/N)",
            env::var("DATABASE_URL")
        );

        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");

        let input = input.trim();

        if input.eq_ignore_ascii_case("y") {
            println!("Setting up database");
            let rt = tokio::runtime::Runtime::new().unwrap();
            match rt.block_on(storage::setup_db()) {
                Ok(_) => println!("Successfully set up database"),
                Err(e) => println!("Error setting up database: {}", e),
            }
        } else {
            println!("Exiting");
        }
    } else {
        server::start().ok();
    }
}

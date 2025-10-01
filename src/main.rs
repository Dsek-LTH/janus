use clap::Parser;

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
}

fn main() {
    let args = Args::parse();

    if args.register {
        register::start();
    } else {
        server::start().ok();
    }
}

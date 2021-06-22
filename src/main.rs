use std::error::Error;
use structopt::StructOpt;

fn main() -> Result<(), Box<dyn Error>> {
    let config = sync_github::Config::from_args();

    match config.run() {
        Ok(_) => Ok(()),
        Err(e) => {
            println!("Error: {}", e);
            Ok(())
        }
    }
}

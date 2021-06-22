use std::thread::{spawn, JoinHandle};
use std::{error::Error, path::Path};
use std::{fmt, fs};
use std::{io::Write, process::Command};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct Config {
    #[structopt(name = "path", required = true, max_values = 1)]
    pub path: String,
}

impl Config {
    pub fn run(self: Self) -> Result<(), Box<dyn Error>> {
        let root = Path::new(&self.path);
        let paths;
        if let Ok(path) = fs::read_dir(root) {
            paths = path;
            let handles = Config::get_handles(paths);

            Config::wait_handle(handles);

            Ok(())
        } else {
            Err(Box::new(SyncError::InvalidPath { path: self.path }))
        }
    }

    fn get_handles(paths: fs::ReadDir) -> Vec<JoinHandle<()>> {
        paths.fold(vec![], |mut handles, result| {
            if let Ok(path) = result {
                let mut current_path = path.path();
                current_path.push(".git");
                if let Ok(_) = fs::metadata(&current_path) {
                    current_path.pop();
                    handles.push(spawn(move || {
                        Config::update(&current_path);
                    }));
                }
            }
            handles
        })
    }

    fn wait_handle(handles: Vec<JoinHandle<()>>) {
        for t in handles {
            t.join().unwrap();
        }
    }

    fn update(path: &Path) {
        let mut fns: Vec<Box<dyn Fn() -> ()>> = vec![];
        let actions = vec![
            vec!["reset", "--hard"],
            vec!["clean", "-df"],
            vec!["pull"],
        ];

        for action in actions {
            let cb = Config::exec(path, "git", action);
            fns.push(cb);
        }

        println!("\nupdate path: {}", path.to_str().unwrap());

        for cb in fns {
            cb();
        }
    }

    fn exec(path: &Path, cmd: &str, args: Vec<&str>) -> Box<dyn Fn() -> ()> {
        let mut git = Command::new(cmd);
        git.args(&args);
        git.current_dir(path);
        let result = git.output();
        Box::new(move || {
            if let Ok(output) = &result {
                println!();
                std::io::stdout().write_all(&output.stdout).unwrap();
            }
        })
    }
}

#[derive(Debug)]
pub enum SyncError {
    InvalidPath { path: String },
}

impl fmt::Display for SyncError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SyncError::InvalidPath { path } => write!(f, "invalid path {}", path),
        }
    }
}

impl Error for SyncError {}

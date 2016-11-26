use std::collections::HashMap;

use std::io::Write;

use std::rc::Rc;

struct Commit {
    parent: Option<Rc<Commit>>,
    payload: String,
}

impl Drop for Commit {
    fn drop(&mut self) {
        println!("'{}' deleted", self.payload);
    }
}

const MASTER: &'static str = "master";

macro_rules! error_out {
    () => {
        println!("Error")
    };
}

type Branch = String;

enum Command {
    NewCommit(Branch, String),
    NewBranch(Branch, Branch, usize),
    DeleteBranch(Branch),
}

struct Repository {
    branches: HashMap<Branch, Rc<Commit>>,
}

impl Commit {
    fn new(payload: String, parent: Rc<Commit>) -> Self {
        Commit { payload: payload, parent: Some(parent) }
    }
    fn new_tree(payload: String) -> Self {
        Commit { payload: payload, parent: None }
    }
}

impl Repository {
    fn new(payload: String) -> Self {
        let mut this = Repository { branches: HashMap::new() };
        println!("{} -> '{}'", MASTER, payload);
        this.branches.insert(MASTER.to_string(), Rc::new(Commit::new_tree(payload)));
        this
    }
    fn do_command(&mut self, c: Command) {
        use Command::*;
        match c {
            NewCommit(branch, payload) => {
                match self.branches.get_mut(&branch) {
                    Some(current_commit) => {
                        println!("{} -> '{}'", branch, payload);
                        *current_commit = Rc::new(Commit::new(payload, current_commit.clone()));
                    },
                    None => error_out!(),
                }
            },
            // TODO: handle offset
            NewBranch(name, location, offset) => {
                let commit = {
                    let mut opt_commit = self.branches.get(&location);
                    for _ in 0..offset {
                        opt_commit = opt_commit.and_then(|c| c.parent.as_ref());
                    }
                    opt_commit.cloned()
                };
                match commit {
                    Some(commit) => {
                        println!("{} -> '{}'", name, commit.payload);
                        self.branches.insert(name, commit);
                    }
                    None => error_out!(),
                }
            },
            DeleteBranch(location) => {
                if self.branches.contains_key(&location) {
                    println!("Removed branch {}", location);
                    self.branches.remove(&location);
                } else {
                    error_out!()
                }
            },
        }
    }
}

mod keyword {
    pub const NEW: &'static str = "new";
    pub const COMMIT: &'static str = "commit";
    pub const BRANCH: &'static str = "branch";
    pub const DELETE: &'static str = "delete";
    pub const QUIT: &'static str = "quit";
}

fn parse_command(input: &str) -> Result<Command,&str> {
    let split = input.split_whitespace().collect::<Vec<_>>();
    let quoted = input.split('\'').nth(1);
    let pre_tilde = split.last().and_then(|t| t.split('~').nth(0));
    let post_tilde = split.last().and_then(|t| t.split('~').nth(1))
                                 .and_then(|s| s.parse::<usize>().ok());
    match split.get(0).cloned() {
        Some(keyword::NEW) => {
            match split.get(1).cloned() {
                Some(keyword::BRANCH) => {
                    let name = split.get(2).ok_or("Missing new branch")?.to_string();
                    let location = pre_tilde.ok_or("Missing location branch")?.to_string();
                    let back_steps = post_tilde.unwrap_or(0);
                    Ok(Command::NewBranch(name, location, back_steps))
                },
                Some(keyword::COMMIT) => {
                    let payload = quoted.ok_or("Missing payload")?.to_string();
                    let branch = split.last().ok_or("Missing commit branch")?.to_string();
                    Ok(Command::NewCommit(branch, payload))
                },
                _ => Err("Unexpected second token after `new`"),
            }
        },
        Some(keyword::DELETE) => {
            if split.get(1).cloned() == Some(keyword::BRANCH) {
                let branch = split.get(2).ok_or("Missing branch to delete")?.to_string();
                Ok(Command::DeleteBranch(branch))
            } else {
                Err("Unexpected second token after `new`")
            }
        },
        _ => Err("Unknown first token"),
    }
}

fn main() {
    let starting_payload = std::env::args().nth(1).expect("Missing starting payload!");
    let mut repository = Repository::new(starting_payload);
    let mut stdout = std::io::stdout();
    loop {
        let mut line = String::new();
        print!("\n> ");
        stdout.flush().unwrap();
        std::io::stdin().read_line(&mut line).unwrap();
        if line.trim() == keyword::QUIT {
            break;
        }
        let command = parse_command(line.as_str()).map_err(|err| {
            println!("Error: {}", err);
            return;
        }).unwrap();
        repository.do_command(command)
    }
}

use serenity::model::id::ChannelId;
use serenity::http::client::Http;
use git2::Repository;
use std::time::Duration;
use std::thread;
use std::env;
use dotenv;

pub struct Error {
    pub msg: String,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        return write!(f, "{}", self.msg);
    }
}
impl From<git2::Error> for Error {
    fn from(error: git2::Error) -> Self {
        return Error { msg: error.to_string() };
    }
}
impl From<env::VarError> for Error {
    fn from(error: env::VarError) -> Self {
        return Error { msg: error.to_string() };
    }
}
impl From<core::num::ParseIntError> for Error {
    fn from(error: core::num::ParseIntError) -> Self {
        return Error { msg: error.to_string() };
    }
}

pub struct Watcher {
    pub branches: Vec<String>,
    pub repo: Repository,
}
impl Watcher {
    pub fn new(path: &str) -> Watcher {
        let repo = match Repository::open(path) {
            Ok(repo) => repo,
            Err(e) => panic!("failed to open: {}", e),
        };
        let branches = repo.branches(Some(git2::BranchType::Local)).unwrap().map(|item| item.unwrap().0.name().unwrap().unwrap().to_string()).collect();
        return Watcher { branches: branches, repo: repo };
    }

    fn checkout(&self, branch: &str) -> Result<(), Error> {
        let (object, reference) = self.repo.revparse_ext(branch)?;

        self.repo.checkout_tree(&object, None)?;

        match reference {
            // gref is an actual reference like branches or tags
            Some(gref) => self.repo.set_head(gref.name().unwrap()),
            // this is a commit, not a reference
            None => self.repo.set_head_detached(object.id()),
        }?;

        return Ok(());
    }

    fn fast_forward(&self, branch: &str) -> Result<(), Error> {
        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.credentials(|_, _, _| {
            let user = env::var("GIT_USER").expect("user");
            let pass = env::var("GIT_PASSWORD").expect("pass");
            return git2::Cred::userpass_plaintext(&user, &pass);
        });

        let mut opts = git2::FetchOptions::new();
        opts.remote_callbacks(callbacks);

        let _ = self.repo.find_remote("origin")?.fetch(&[branch], Some(&mut opts), None)?;

        let fetch_head = self.repo.find_reference("FETCH_HEAD")?;
        let fetch_commit = self.repo.reference_to_annotated_commit(&fetch_head)?;
        let analysis = self.repo.merge_analysis(&[&fetch_commit])?;
        if analysis.0.is_up_to_date() {
            return Ok(());
        } else if analysis.0.is_fast_forward() {
            let refname = format!("refs/heads/{}", branch);
            let mut reference = self.repo.find_reference(&refname)?;
            reference.set_target(fetch_commit.id(), "Fast-Forward")?;
            self.repo.set_head(&refname)?;
            return Ok(self.repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?);
        } else {
            return Err(Error { msg: "Fast-forward only!".to_string() });
        }
    }

    pub async fn watch(self) -> Result<(), Error> {
        let token = env::var("DISCORD_TOKEN")?;
        let client = Http::new_with_token(&token);
        loop {
            thread::sleep(Duration::new(10, 0));

            for branch in &self.branches {
                self.checkout(branch)?;
                let id = self.repo.head()?.peel_to_commit()?.id();
                self.fast_forward(branch)?;
                let new_commit = self.repo.head()?.peel_to_commit()?;

                println!("[{}] before: {} - after: {}", branch, id, new_commit.id());

                if id != new_commit.id() {
                    let channel_id = env::var("CHANNEL_ID")?;
                    let _ = ChannelId::from(channel_id.parse::<u64>()?).say(&client, format!("{} pushed to {}: {}", new_commit.author().name().unwrap_or("Someone"), branch, new_commit.summary().unwrap_or("No message"))).await;
                    
                    return Ok(());
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let _ = dotenv::dotenv();

    loop {
        let watcher = Watcher::new("/home/pi/spacelines");
        match watcher.watch().await {
            Err(e) => println!("{}", e),
            _ => (),
        }
    }
}

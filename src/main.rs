use serenity::model::id::ChannelId;
use serenity::http::client::Http;
use git2::Repository;
use std::time::Duration;
use std::thread;
use std::env;
use dotenv;

pub struct Watcher {
    pub branch: String,
    pub repo: Repository,
}
impl Watcher {
    pub fn new(path: &str, branch: String) -> Watcher {
        let repo = match Repository::open(path) {
            Ok(repo) => repo,
            Err(e) => panic!("failed to open: {}", e),
        };
        return Watcher { branch: branch, repo: repo };
    }

    fn fast_forward(&self) -> Result<(), git2::Error> {
        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.credentials(|_, _, _| {
            let user = env::var("GIT_USER").expect("user");
            let pass = env::var("GIT_PASSWORD").expect("pass");
            return git2::Cred::userpass_plaintext(&user, &pass);
        });

        let mut opts = git2::FetchOptions::new();
        opts.remote_callbacks(callbacks);

        let _ = self.repo.find_remote("origin")?.fetch(&[&self.branch], Some(&mut opts), None)?;

        let fetch_head = self.repo.find_reference("FETCH_HEAD")?;
        let fetch_commit = self.repo.reference_to_annotated_commit(&fetch_head)?;
        let analysis = self.repo.merge_analysis(&[&fetch_commit])?;
        if analysis.0.is_up_to_date() {
            Ok(())
        } else if analysis.0.is_fast_forward() {
            let refname = format!("refs/heads/{}", self.branch);
            let mut reference = self.repo.find_reference(&refname)?;
            reference.set_target(fetch_commit.id(), "Fast-Forward")?;
            self.repo.set_head(&refname)?;
            self.repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))
        } else {
            Err(git2::Error::from_str("Fast-forward only!"))
        }
    }

    pub async fn watch(self) {
        let token = env::var("DISCORD_TOKEN").expect("token");
        let client = Http::new_with_token(&token);
        loop {
            thread::sleep(Duration::new(10, 0));

            let id = self.repo.head().unwrap().peel_to_commit().unwrap().id();
            let _ = self.fast_forward().map_err(|e| println!("{:?}", e));
            let new_commit = self.repo.head().unwrap().peel_to_commit().unwrap();

            if id != new_commit.id() {
                let _ = ChannelId::from(829837845916549165).say(&client, format!("{} pushed to develop! Now at {}", new_commit.author().name().unwrap_or("Someone"), new_commit.id())).await;
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let _ = dotenv::dotenv();

    let watcher = Watcher::new("/home/lucas/Documents/test", "master".to_string());
    watcher.watch().await;
}

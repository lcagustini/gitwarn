use serenity::model::id::ChannelId;
use serenity::http::client::Http;
use git2::Repository;
use std::time::Duration;
use std::thread;
use std::env;
use dotenv;

#[tokio::main]
async fn main() {
    let _ = dotenv::dotenv();
    let token = env::var("DISCORD_TOKEN").expect("token");

    let client = Http::new_with_token(&token);

    let repo = match Repository::open("/home/lucas/Documents/test/") {
        Ok(repo) => repo,
        Err(e) => panic!("failed to open: {}", e),
    };

    loop {
        thread::sleep(Duration::new(10, 0));

        let id = repo.head().unwrap().peel_to_commit().unwrap().id();
        let _ = fast_forward(&repo).map_err(|e| println!("{:?}", e));
        let new_id = repo.head().unwrap().peel_to_commit().unwrap().id();

        if id != new_id {
            let _ = ChannelId::from(829837845916549165).say(&client, format!("Someone pushed to develop! Now at {}", new_id)).await;
        }
    }
}

fn fast_forward(repo: &Repository) -> Result<(), git2::Error> {
    let mut callbacks = git2::RemoteCallbacks::new();
    callbacks.credentials(|_, _, _| {
        let user = env::var("GIT_USER").expect("user");
        let pass = env::var("GIT_PASSWORD").expect("pass");
        return git2::Cred::userpass_plaintext(&user, &pass);
    });

    let mut opts = git2::FetchOptions::new();
    opts.remote_callbacks(callbacks);

    let _ = repo.find_remote("origin")?.fetch(&["master"], Some(&mut opts), None)?;

    let fetch_head = repo.find_reference("FETCH_HEAD")?;
    let fetch_commit = repo.reference_to_annotated_commit(&fetch_head)?;
    let analysis = repo.merge_analysis(&[&fetch_commit])?;
    if analysis.0.is_up_to_date() {
        Ok(())
    } else if analysis.0.is_fast_forward() {
        let refname = format!("refs/heads/{}", "master");
        let mut reference = repo.find_reference(&refname)?;
        reference.set_target(fetch_commit.id(), "Fast-Forward")?;
        repo.set_head(&refname)?;
        repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))
    } else {
        Err(git2::Error::from_str("Fast-forward only!"))
    }
}


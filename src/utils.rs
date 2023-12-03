use std::env;

use gitlab::Gitlab;

pub fn get_client() -> Gitlab {
    let token = env::var("GITLAB_TOKEN").unwrap();
    let client = Gitlab::new("gitlab.com", token).unwrap();
    client
}

use std::env;
use std::io;
use std::io::Read;

use gitlab::api::projects::merge_requests::notes;
use gitlab::api::{self, users, Query};
use serde::Deserialize;

mod utils;

#[derive(Debug, Deserialize, PartialEq)]
struct User {
    id: u64,
}
#[derive(Debug, Deserialize)]
struct MergeRequestNote {
    id: u64,
    body: String,
    author: User,
}

fn main() {
    let project_id: u64 = env::var("CI_MERGE_REQUEST_PROJECT_ID")
        .unwrap()
        .parse()
        .unwrap();
    let comment_mark: String =
        env::var("COMMENTATOR_MARK").unwrap_or("gl-mr-commentator".to_string());
    let mr_id: u64 = env::var("CI_MERGE_REQUEST_IID").unwrap().parse().unwrap();
    let client = utils::get_client();

    let marker = format!("<!-- {} -->", comment_mark);
    println!("Merge request marker is '{}'", marker);

    let mut content = marker.clone();
    content.push('\n');
    io::stdin().read_to_string(&mut content).unwrap();
    content = content
        + r#"
---

This comment was made by the [GitLab Merge Request Commentator](https://github.com/MarkusH/gl-mr-commentator)."#;

    let endpoint = users::CurrentUser::builder().build().unwrap();
    let current_user: User = endpoint.query(&client).unwrap();

    let endpoint = notes::MergeRequestNotes::builder()
        .project(project_id)
        .merge_request(mr_id)
        .build()
        .unwrap();
    let notes: Vec<MergeRequestNote> = endpoint.query(&client).unwrap();

    let mut note: Option<MergeRequestNote> = None;
    for n in notes {
        if n.author != current_user {
            continue;
        }
        if !n.body.starts_with(&marker) {
            continue;
        }
        note = Some(n);
        break;
    }

    match note {
        Some(n) => {
            println!("Updating merge request comment {}", n.id);
            let endpoint = notes::EditMergeRequestNote::builder()
                .project(project_id)
                .merge_request(mr_id)
                .note(n.id)
                .body(content)
                .build()
                .unwrap();
            let _: () = api::ignore(endpoint).query(&client).unwrap();
        }
        None => {
            println!("Creating merge request comment ...");
            let endpoint = notes::CreateMergeRequestNote::builder()
                .project(project_id)
                .merge_request(mr_id)
                .body(content)
                .build()
                .unwrap();
            let _: () = api::ignore(endpoint).query(&client).unwrap();
        }
    }
}

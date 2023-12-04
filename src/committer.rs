use std::env;
use std::fs;

use gitlab::api::projects::merge_requests;
use gitlab::api::projects::repository::{branches, commits};
use gitlab::api::{self, Query};
use gitlab::Gitlab;
use serde::Deserialize;
mod utils;

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct MergeRequest {
    id: u64,
    title: String,
}

#[derive(Debug, Deserialize)]
struct Branch {}

fn ensure_branch_exists(client: &Gitlab, project_id: u64, branch_name: &String) -> Branch {
    println!(
        "Ensuring branch '{}' exists in project {} ...",
        branch_name, project_id
    );
    let endpoint = branches::Branch::builder()
        .project(project_id)
        .branch(branch_name)
        .build()
        .unwrap();
    let branch: Branch = match endpoint.query(client) {
        Ok(b) => {
            println!(
                "Branch '{}' exists in project {} ...",
                branch_name, project_id
            );
            b
        }
        Err(_) => {
            println!(
                "Creating branch '{}' in project {} ...",
                branch_name, project_id
            );
            let endpoint = branches::CreateBranch::builder()
                .project(project_id)
                .branch(branch_name)
                .ref_("main")
                .build()
                .unwrap();
            let _: () = api::ignore(endpoint).query(client).unwrap();
            let endpoint = branches::Branch::builder()
                .project(project_id)
                .branch(branch_name)
                .build()
                .unwrap();
            let branch: Branch = endpoint.query(client).unwrap();
            branch
        }
    };
    branch
}

fn update_file(
    client: &Gitlab,
    project_id: u64,
    branch_name: &String,
    file_path: &String,
    content: &[u8],
) {
    println!(
        "Updating file '{}' in branch '{}' in project {} ...",
        file_path, branch_name, project_id
    );

    let action = commits::CommitAction::builder()
        .action(commits::CommitActionType::Update)
        .file_path(file_path)
        .content(content)
        .build()
        .unwrap();
    let endpoint = commits::CreateCommit::builder()
        .project(project_id)
        .branch(branch_name)
        .action(action)
        .commit_message("chore: Update files")
        .build()
        .unwrap();
    let _: () = api::ignore(endpoint).query(client).unwrap();
    println!(
        "Updated file '{}' in branch '{}' in project {}.",
        file_path, branch_name, project_id
    );
}

fn ensure_merge_request_is_open(
    client: &Gitlab,
    project_id: u64,
    branch_name: &String,
    title: &String,
) -> MergeRequest {
    println!(
        "Ensuring merge request '{}' from branch {} exists in project {} ...",
        title, branch_name, project_id
    );
    let endpoint = merge_requests::MergeRequests::builder()
        .project(project_id)
        .state(merge_requests::MergeRequestState::Opened)
        .source_branch(branch_name)
        .build()
        .unwrap();
    let open_merge_requests: Vec<MergeRequest> = api::paged(endpoint, api::Pagination::All)
        .query(client)
        .unwrap();

    for merge_request in open_merge_requests.iter() {
        if merge_request.title.eq(title) {
            println!(
                "Found merge request '{}' from branch '{}' in project {} ...",
                merge_request.title, branch_name, project_id
            );
            return merge_request.clone();
        }
    }

    println!(
        "Creating merge request '{}' from branch '{}' in project {} ...",
        title, branch_name, project_id
    );
    let endpoint = merge_requests::CreateMergeRequest::builder()
        .project(project_id)
        .source_branch(branch_name)
        .target_branch("main")
        .title(title)
        .description(
            r#"
---

This merge request was opened by the [GitLab Merge Request Commentator](https://github.com/MarkusH/gl-mr-commentator)."#,
        )
        .remove_source_branch(true)
        .squash(true)
        .build().unwrap();
    let mr: MergeRequest = endpoint.query(client).unwrap();
    return mr;
}

fn main() {
    let target_project_id: u64 = env::var("COMMITTER_TARGET_PROJECT_ID")
        .unwrap()
        .parse()
        .unwrap();
    let target_branch_name: String = env::var("COMMITTER_TARGET_BRANCH").unwrap();
    let target_file_path: String = env::var("COMMITTER_TARGET_FILE_PATH").unwrap();
    let target_merge_request_title: String =
        env::var("COMMITTER_TARGET_MERGE_REQUEST_TITLE").unwrap();
    let source_file_path: String = env::var("COMMITTER_SOURCE_FILE_PATH").unwrap();
    let client = utils::get_client();

    ensure_branch_exists(&client, target_project_id, &target_branch_name);

    let content = fs::read(source_file_path).unwrap();

    update_file(
        &client,
        target_project_id,
        &target_branch_name,
        &target_file_path,
        &content,
    );

    ensure_merge_request_is_open(
        &client,
        target_project_id,
        &target_branch_name,
        &target_merge_request_title,
    );

    // let endpoint = users::CurrentUser::builder().build().unwrap();
    // let current_user: User = endpoint.query(&client).unwrap();

    // let endpoint = notes::MergeRequestNotes::builder()
    //     .project(project_id)
    //     .merge_request(mr_id)
    //     .build()
    //     .unwrap();
    // let notes: Vec<MergeRequestNote> = endpoint.query(&client).unwrap();

    // let mut note: Option<MergeRequestNote> = None;
    // for n in notes {
    //     if n.author != current_user {
    //         continue;
    //     }
    //     if !n.body.starts_with(&marker) {
    //         continue;
    //     }
    //     note = Some(n);
    //     break;
    // }

    // match note {
    //     Some(n) => {
    //         println!("Updating merge request comment {}", n.id);
    //         let endpoint = notes::EditMergeRequestNote::builder()
    //             .project(project_id)
    //             .merge_request(mr_id)
    //             .note(n.id)
    //             .body(content)
    //             .build()
    //             .unwrap();
    //         let _: () = api::ignore(endpoint).query(&client).unwrap();
    //     }
    //     None => {
    //         println!("Creating merge request comment ...");
    //         let endpoint = notes::CreateMergeRequestNote::builder()
    //             .project(project_id)
    //             .merge_request(mr_id)
    //             .body(content)
    //             .build()
    //             .unwrap();
    //         let _: () = api::ignore(endpoint).query(&client).unwrap();
    //     }
    // }
}

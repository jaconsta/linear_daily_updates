pub mod queries;

use std::env;

use dotenvy::dotenv;

use chrono::NaiveDateTime;
use cynic::{http::ReqwestExt, QueryBuilder};
use reqwest::header::{HeaderMap, AUTHORIZATION};

static MORE_THAN_ONE_DAY: i64 = 30;
static TEAM_SEARCH: &str = "AWA";

use crate::queries::structs::{
    active_issues::{
        ActiveIssuesQuery, ActiveIssuesVariables, DoneIssuesQuery, Issue, TodoIssuesQuery,
    },
    projects::ListProjectsQuery,
    schema::DateTime,
    teams::TeamsQuery,
};

#[derive(Debug, Clone, Default)]
struct Tasks {
    pub todo: Vec<String>,
    pub in_progress: Vec<String>,
    pub in_review: Vec<String>,
    pub in_testing: Vec<String>,
    pub done: Vec<String>,
}

async fn get_tasks(team_id: &str) -> Result<Vec<Issue>, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, env::var("LINEAR_TOKEN")?.parse()?);

    let done_issues_operation = DoneIssuesQuery::build(ActiveIssuesVariables {
        id: Some(team_id.into()),
        completed_at: Some(DateTime("-P4D".to_string())), // -P2W
    });

    let active_issues_operation = ActiveIssuesQuery::build(ActiveIssuesVariables {
        id: Some(team_id.into()),
        completed_at: None,
    });
    let todo_issues_operation = TodoIssuesQuery::build(ActiveIssuesVariables {
        id: Some(team_id.into()),
        completed_at: None,
    });

    let mut unclassified_tasks: Vec<Issue> = Vec::new();

    let issues_request_base = client
        .post("https://api.linear.app/graphql")
        .headers(headers.clone());

    let issues_request = issues_request_base.run_graphql(done_issues_operation);
    let issues_response = issues_request.await?;

    if let Some(err) = issues_response.errors {
        return Err(Box::new(simple_error::simple_error!(
            "There was an error loading issues_response {:?}",
            err
        )));
    }

    // let now = chrono::Local::now().naive_utc();
    if let Some(data_res) = &issues_response.data {
        if data_res.issues.page_info.has_next_page {
            println!(
                "done_issues_operation should be paginated, Got {} tasks",
                data_res.issues.nodes.len()
            );
        }

        for issue in &data_res.issues.nodes {
            unclassified_tasks.push(issue.clone());
        }
    }

    // ---

    let issues_request_base = client
        .post("https://api.linear.app/graphql")
        .headers(headers.clone());
    let issues_request = issues_request_base.run_graphql(active_issues_operation);
    let issues_response = issues_request.await?;

    if let Some(err) = issues_response.errors {
        return Err(Box::new(simple_error::simple_error!(
            "There was an error loading issues_response {:?}",
            err
        )));
    }

    if let Some(data_res) = &issues_response.data {
        if data_res.issues.page_info.has_next_page {
            println!(
                "active_issues_operation should be paginated, Got {} tasks",
                data_res.issues.nodes.len()
            );
        }

        for issue in &data_res.issues.nodes {
            unclassified_tasks.push(issue.clone());
        }
    }

    // ---

    let issues_request_base = client
        .post("https://api.linear.app/graphql")
        .headers(headers.clone());
    let issues_request = issues_request_base.run_graphql(todo_issues_operation);
    let issues_response = issues_request.await?;

    if let Some(err) = issues_response.errors {
        return Err(Box::new(simple_error::simple_error!(
            "There was an error loading issues_response {:?}",
            err
        )));
    }

    if let Some(data_res) = &issues_response.data {
        if data_res.issues.page_info.has_next_page {
            println!(
                "todo_issues_operation should be paginated, Got {} tasks",
                data_res.issues.nodes.len()
            );
        }

        for issue in &data_res.issues.nodes {
            if !unclassified_tasks
                .iter()
                .any(|i| i.identifier == issue.identifier)
            {
                unclassified_tasks.push(issue.clone());
            }
        }
    }

    Ok(unclassified_tasks)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv()?;
    let _projects_operation = ListProjectsQuery::build(());
    let teams_operation = TeamsQuery::build(());

    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, env::var("LINEAR_TOKEN")?.parse()?);

    // let projects_response = client
    //     .post("https://api.linear.app/graphql")
    //     .headers(headers)
    //     .run_graphql(projects_operation)
    //     .await?;

    println!("Getting teams information! ");
    let teams_response = client
        .post("https://api.linear.app/graphql")
        .headers(headers.clone())
        .run_graphql(teams_operation)
        .await?;

    if let Some(err) = teams_response.errors {
        println!("There was an error loading teams_response: {:?}", err);
        return Ok(());
    }

    let mut team_id = "";
    if let Some(data_res) = &teams_response.data {
        for team in &data_res.teams.nodes {
            if team.key == TEAM_SEARCH {
                team_id = team.id.inner();
            }
            println!(
                "Team name {}, prefix {}, members {}",
                team.name,
                team.key,
                team.members.nodes.len()
            );
        }
    }
    println!(
        "Getting issues information for {} {}! ",
        TEAM_SEARCH, &team_id
    );

    let mut current_tasks = Tasks::default();
    let unclassified_tasks = get_tasks(team_id).await?;

    let now = chrono::Local::now().naive_utc();
    for issue in unclassified_tasks.iter() {
        let issue_identifier = issue.identifier.to_string();
        let task_state: &mut Vec<String> = match issue.state.name.as_str() {
            "In Progress" => current_tasks.in_progress.as_mut(),
            "In Review" => current_tasks.in_review.as_mut(),
            "In Testing" => current_tasks.in_testing.as_mut(),
            "Done" => current_tasks.done.as_mut(),
            _ => current_tasks.todo.as_mut(),
        };

        let events_history: Vec<(String, NaiveDateTime)> = issue
            .history
            .nodes
            .iter()
            .filter(|node| node.to_state.is_some())
            .map(|node| {
                (
                    format!("{}", &node.to_state.as_ref().unwrap().name),
                    chrono::DateTime::parse_from_rfc3339(&node.updated_at.0)
                        .unwrap()
                        .naive_utc(),
                )
            })
            .collect();

        if let Some(first) = events_history.first() {
            let hours_ago = (now - first.1).num_hours();
            let state_string = format!("Happened {} hours ago.", hours_ago);

            if hours_ago < MORE_THAN_ONE_DAY {
                task_state.push(format!("{} {}", issue_identifier, state_string));
            }
        }
    }

    println!("---- || ----");
    println!("Todo: ");
    for task in current_tasks.todo {
        println!("{}", task);
    }
    println!("in progress:",);
    for task in current_tasks.in_progress {
        println!("{}", task);
    }
    println!("in review: ");
    for task in current_tasks.in_review {
        println!("{}", task);
    }
    println!("in testing:");
    for task in current_tasks.in_testing {
        println!("{}", task);
    }
    println!("done tasks: ");
    for task in current_tasks.done {
        println!("{}", task);
    }

    Ok(())
}

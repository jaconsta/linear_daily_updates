pub mod queries;

use std::env;

use dotenvy::dotenv;

use chrono::NaiveDateTime;
use cynic::{http::ReqwestExt, QueryBuilder};
use reqwest::header::{HeaderMap, AUTHORIZATION};

use crate::queries::structs::{
    active_issues::{ActiveIssuesQuery, ActiveIssuesVariables, DoneIssuesQuery, TodoIssuesQuery},
    projects::ListProjectsQuery,
    schema::DateTime,
    teams::TeamsQuery,
};

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
            if team.key == "AWA" {
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
    println!("Getting issues information for AWA {}! ", &team_id);

    let done_issues_operation = DoneIssuesQuery::build(ActiveIssuesVariables {
        id: Some(team_id.into()),
        completed_at: Some(DateTime("-P4D".to_string())), // -P2W //Some(chrono::offset::Local::now()),
    });

    let active_issues_operation = ActiveIssuesQuery::build(ActiveIssuesVariables {
        id: Some(team_id.into()),
        completed_at: None,
    });
    let todo_issues_operation = TodoIssuesQuery::build(ActiveIssuesVariables {
        id: Some(team_id.into()),
        completed_at: None,
    });

    let mut todo_tasks: Vec<String> = vec![];
    let mut in_progress_tasks: Vec<String> = vec![];
    let mut in_review_tasks: Vec<String> = vec![];
    let mut in_testing_tasks: Vec<String> = vec![];
    let mut done_tasks: Vec<String> = vec![];

    let issues_request = client
        .post("https://api.linear.app/graphql")
        .headers(headers.clone());

    let issues_request = issues_request.run_graphql(done_issues_operation);

    // let issues_request = match index {
    //     1 => issues_request.run_graphql(todo_issues_operation),
    //     2 => issues_request.run_graphql(done_issues_operation),
    //     _ => issues_request.run_graphql(active_issues_operation),
    // };
    // let issues_response = client
    //     .post("https://api.linear.app/graphql")
    //     .headers(headers.clone())
    //     .run_graphql(issue_operation)
    let issues_response = issues_request.await?;

    if let Some(err) = issues_response.errors {
        println!("There was an error loading issues_response: {:?}", err);
        return Ok(());
    }

    let now = chrono::Local::now().naive_utc(); // chrono::offset::Utc::now();
    let mut print_me_extra = "".to_string();
    if let Some(data_res) = &issues_response.data {
        println!("Pagination Info {:?}", &data_res.issues.page_info);
        println!("Issues Found! {:?}", data_res.issues.nodes.len());

        for issue in &data_res.issues.nodes {
            if issue.identifier == "AWA-3242" {
                print_me_extra = format!("{:?}", &issue);
            }

            println!(
                "Issue id {}, priority {}, state {}",
                issue.identifier, issue.priority_label, issue.state.name,
            );

            match issue.state.name.as_str() {
                "Todo" => todo_tasks.push(issue.identifier.to_string()),
                "In Progress" => in_progress_tasks.push(issue.identifier.to_string()),
                "In Review" => in_review_tasks.push(issue.identifier.to_string()),
                "In Testing" => in_testing_tasks.push(issue.identifier.to_string()),
                "Done" => done_tasks.push(issue.identifier.to_string()),
                _ => (),
            };
            let labels: Vec<String> = issue
                .labels
                .nodes
                .iter()
                .map(|label| label.name.clone())
                .collect();
            println!("  Labels {}", labels.join(", "));

            let events_history: Vec<(String, NaiveDateTime)> = issue
                .history
                .nodes
                .iter()
                .filter(|node| node.to_state.is_some())
                .map(
                    |node| {
                        (
                            format!("{}", &node.to_state.as_ref().unwrap().name),
                            chrono::DateTime::parse_from_rfc3339(&node.updated_at.0)
                                .unwrap()
                                .naive_utc(),
                        )
                    }, //format!(
                       //    "{} - {}",
                       //    &node.to_state.as_ref().unwrap().name,
                       //    &node.updated_at.0,
                       //)
                )
                .collect();

            if events_history.len() > 0 {
                let first = events_history.first().unwrap();
                if events_history.len() > 1 {
                    let second = &events_history[1];
                    let time_diff = first.1 - second.1;
                    println!("  times {} {}", first.1, second.1);
                    println!(
                        "  Event {}, changed after {} hours",
                        &first.0,
                        time_diff.num_hours()
                    );
                }
                println!("  And happened {} hours ago", (now - first.1).num_hours());
                // println!("  Events {:?}", events_history.join(", "));
                println!(
                    "  State {} on {:?}",
                    issue.state.name, issue.state.updated_at.0
                );
                println!("  - And last update {:?}", issue.updated_at.0);
            }
        }
    }

    println!("Issues sample! {}", print_me_extra);

    println!("---- || ----");
    println!("todo: {:?}", &todo_tasks);
    println!("in progress: {}", &in_progress_tasks.join(", "));
    println!("in review: {}", &in_review_tasks.join(", "));
    println!("in testing: {}", &in_testing_tasks.join(", "));
    println!("done tasks: {}", &done_tasks.join(", "));

    Ok(())
}

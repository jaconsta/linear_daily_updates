use super::schema::{schema, DateTime};

#[derive(cynic::QueryVariables)]
pub struct ActiveIssuesVariables {
    pub id: Option<cynic::Id>,
    pub completed_at: Option<DateTime>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "ActiveIssuesVariables")]
pub struct TodoIssuesQuery {
    #[arguments(filter: {canceledAt: { null: true},  state: {type: {in: ["started", "unstarted"] }}, team: { id: { eq: $id } } })]
    pub issues: IssueConnection,
}
#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "ActiveIssuesVariables")]
pub struct InTestingIssuesQuery {
    #[arguments(filter: {canceledAt: { null: true}, completedAt: { lt: $completed_at, null: false }, startedAt: { null: false}, state: {name: {neq:"Done" }}, team: { id: { eq: $id } } })]
    pub issues: IssueConnection,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "ActiveIssuesVariables")]
pub struct DoneIssuesQuery {
    #[arguments(filter: {canceledAt: { null: true}, completedAt: { gt: $completed_at}, state: {name: {eq:"Done" }}, team: { id: { eq: $id } } })]
    pub issues: IssueConnection,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "ActiveIssuesVariables")]
pub struct ActiveIssuesQuery {
    #[arguments(filter: {canceledAt: { null: true}, startedAt: { null: false}, state: {name: {neq:"Done" }}, team: { id: { eq: $id } } })]
    pub issues: IssueConnection,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct IssueConnection {
    pub page_info: PageInfo,
    pub nodes: Vec<Issue>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct PageInfo {
    pub start_cursor: Option<String>,
    pub end_cursor: Option<String>,
    pub has_next_page: bool,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Issue {
    pub description: Option<String>,
    pub history: IssueHistoryConnection,
    pub title: String,
    pub labels: IssueLabelConnection,
    pub priority: f64,
    pub updated_at: DateTime,
    pub number: f64,
    pub completed_at: Option<DateTime>,
    pub canceled_at: Option<DateTime>,
    pub state: WorkflowState,
    pub assignee: Option<User>,
    pub priority_label: String,
    pub identifier: String,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct User {
    pub display_name: String,
    pub name: String,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct IssueLabelConnection {
    pub nodes: Vec<IssueLabel>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct IssueLabel {
    pub name: String,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct IssueHistoryConnection {
    pub nodes: Vec<IssueHistory>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct IssueHistory {
    pub id: cynic::Id,
    pub from_title: Option<String>,
    pub from_state_id: Option<String>,
    // pub from_state: Option<WorkflowState>,
    // pub changes: Option<Jsonobject>,
    pub to_state: Option<WorkflowState>,
    pub updated_at: DateTime,
}

// #[derive(cynic::QueryFragment, Debug)]
// #[cynic(graphql_type = "WorkflowState")]
// pub struct WorkflowState2 {
//     pub name: String,
//     pub id: cynic::Id,
//     #[cynic(rename = "type")]
//     pub type_: String,
// }

#[derive(cynic::QueryFragment, Debug)]
pub struct WorkflowState {
    pub name: String,
    // #[cynic(rename = "type")]
    // pub type_: String,
    // pub description: Option<String>,
    pub id: cynic::Id,
    // pub created_at: DateTime,
    pub updated_at: DateTime,
}

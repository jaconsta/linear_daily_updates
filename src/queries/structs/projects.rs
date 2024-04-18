use super::schema::{schema, DateTime};

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query")]
pub struct ListProjectsQuery {
    pub projects: ProjectConnection,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct ProjectConnection {
    pub nodes: Vec<Project>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Project {
    pub name: String,
    pub id: cynic::Id,
    pub description: String,
    pub completed_at: Option<DateTime>,
    pub progress: f64,
    pub started_at: Option<DateTime>,
    pub state: String,
}

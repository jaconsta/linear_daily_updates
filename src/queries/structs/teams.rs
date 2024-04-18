use super::schema::schema;

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query")]
pub struct TeamsQuery {
    pub teams: TeamConnection,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct TeamConnection {
    pub nodes: Vec<Team>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Team {
    pub id: cynic::Id,
    pub name: String,
    pub description: Option<String>,
    pub issue_count: i32,
    pub key: String,
    pub members: UserConnection,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct UserConnection {
    pub nodes: Vec<User>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct User {
    pub email: String,
    pub id: cynic::Id,
    pub name: String,
    pub active: bool,
}

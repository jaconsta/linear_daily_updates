#[derive(cynic::Scalar, Debug, Clone)]
pub struct DateTime(pub String);

#[derive(cynic::Scalar, Debug, Clone)]
#[cynic(graphql_type = "JSONObject")]
pub struct Jsonobject(pub String);

#[cynic::schema("linear")]
pub mod schema {}

use super::Input;
use anyhow::{Context, Result};
use graphql_client::{GraphQLQuery, Response};

const GN_INDEX_URL: &str = "http://index.globalnames.org/api/graphql";

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/verif/schema.json",
    query_path = "src/verif/query.graphql",
    response_derives = "Debug"
)]
pub struct Resolver;

pub fn verify(inputs: &Vec<Input>, sources: &Option<Vec<i64>>) -> Result<resolver::ResponseData> {
    let mut names: Vec<resolver::name> = Vec::with_capacity(inputs.len());
    for input in inputs {
        names.push(resolver::name {
            supplied_id: input.id.to_owned(),
            value: input.name.to_owned(),
        });
    }
    let q = Resolver::build_query(resolver::Variables {
        names,
        sources: sources.to_owned(),
    });
    let client = reqwest::Client::new();

    let mut res = client.post(GN_INDEX_URL).json(&q).send()?;

    let response_body: Response<resolver::ResponseData> = res.json()?;

    if let Some(errors) = response_body.errors {
        println!("there are errors:");

        for error in &errors {
            println!("{:?}", error);
        }
    }
    let response_data: resolver::ResponseData =
        response_body.data.context("gnindex remote error")?;
    Ok(response_data)
}

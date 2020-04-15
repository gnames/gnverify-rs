pub mod output;
pub mod output_csv;
pub mod remote;

pub use output::{MatchType, Output};
pub use output_csv::OutputCSV;
pub type Verified = remote::resolver::ResolverNameResolverResponses;
pub type VerifiedData = remote::resolver::ResolverNameResolverResponsesResults;
pub type VerifiedPreferredData = remote::resolver::ResolverNameResolverResponsesPreferredResults;

#[derive(Debug, Default)]
pub struct Input {
    pub id: Option<String>,
    pub name: String,
}

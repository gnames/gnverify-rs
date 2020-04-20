pub mod output;
pub mod output_csv;
pub mod remote;

pub use output::{MatchType, Output};
pub use output_csv::OutputCSV;
pub type Verified = remote::resolver::ResolverNameResolverResponses;
pub type VerifiedData = remote::resolver::ResolverNameResolverResponsesResults;
pub type VerifiedPreferredData = remote::resolver::ResolverNameResolverResponsesPreferredResults;

/// The input format to send to gnindex server.
#[derive(Debug, Default)]
pub struct Input {
    /// Optional ID attached to a name-string.
    pub id: Option<String>,
    /// Name-string to verify.
    pub name: String,
}

pub mod output;
pub mod remote;

pub use output::Output;
pub type Verified = remote::resolver::ResolverNameResolverResponses;
pub type VerifiedData = remote::resolver::ResolverNameResolverResponsesResults;
pub type VerifiedPreferredData = remote::resolver::ResolverNameResolverResponsesPreferredResults;

#[derive(Debug, Default)]
pub struct Input {
    pub id: Option<String>,
    pub name: String,
}

use super::MatchType;
use serde::Serialize;

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct OutputCSV {
    pub kind: String,
    pub match_type: MatchType,
    pub edit_distance: Option<i64>,
    pub scientific_name: String,
    pub matched_name: Option<String>,
    pub matched_canonical: Option<String>,
    pub taxon_id: Option<String>,
    pub current_name: Option<String>,
    pub synonym: bool,
    pub data_source_id: Option<i64>,
    pub data_source_title: Option<String>,
    pub classification_path: Option<String>,
}

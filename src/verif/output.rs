use super::{Verified, VerifiedData, VerifiedPreferredData};
use serde::{Serialize, Serializer};

trait ToResultData {
    fn to_result_data(&self) -> ResultData;
}

#[derive(Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Output {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub name: String,
    pub match_type: MatchType,
    pub data_sources_num: i64,
    pub data_source_curation: CurationType,
    pub retries: i64,
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub best_result: Option<ResultData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred_results: Option<Vec<ResultData>>,
}

#[derive(Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResultData {
    match_type: MatchType,
    data_source_id: i64,
    data_source_title: String,
    taxon_id: String,
    matched_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    matched_canonical: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    current_name: Option<String>,
    synonym: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    classification_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    classification_rank: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    classification_ids: Option<String>,
    edit_distance: i64,
    stem_edit_distance: i64,
}

#[derive(Debug, Clone)]
pub enum MatchType {
    NoMatch,
    Exact,
    Fuzzy,
    PartialExact,
    PartialFuzzy,
}

impl Default for MatchType {
    fn default() -> Self {
        MatchType::NoMatch
    }
}

impl Serialize for MatchType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            MatchType::Exact => serializer.serialize_str("Exact"),
            MatchType::Fuzzy => serializer.serialize_str("Fuzzy"),
            MatchType::PartialExact => serializer.serialize_str("PartialExact"),
            MatchType::PartialFuzzy => serializer.serialize_str("PartialFuzzy"),
            _ => serializer.serialize_str("NoMatch"),
        }
    }
}

#[derive(Debug)]
pub enum CurationType {
    NotCurated,
    AutoCurated,
    Curated,
}

impl Default for CurationType {
    fn default() -> Self {
        CurationType::NotCurated
    }
}

impl Serialize for CurationType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            CurationType::NotCurated => serializer.serialize_str("not_curated"),
            CurationType::AutoCurated => serializer.serialize_str("auto_curated"),
            CurationType::Curated => serializer.serialize_str("curated"),
        }
    }
}

impl Output {
    pub fn new<'b>(item: Verified, retries: i64) -> Self {
        let mut best_result: Option<ResultData> = None;
        let mut match_type = MatchType::NoMatch;
        if item.results.len() > 0 {
            let best_match = item.results[0].to_result_data();
            match_type = best_match.match_type.clone();
            best_result = Some(best_match);
        };
        let mut preferred_results: Option<Vec<ResultData>> = None;
        let mut pref_res_tmp: Vec<ResultData> = Vec::with_capacity(item.preferred_results.len());
        for res in item.preferred_results {
            pref_res_tmp.push(res.to_result_data())
        }
        if pref_res_tmp.len() > 0 {
            preferred_results = Some(pref_res_tmp);
        };
        let curation_str = item.quality_summary.unwrap_or("".to_owned());

        Output {
            id: item.supplied_id,
            name: item.supplied_input.unwrap(),
            match_type,
            data_sources_num: item.matched_data_sources,
            data_source_curation: get_curation(&curation_str),
            retries,
            error: None,
            best_result,
            preferred_results,
        }
    }
}

fn get_curation(cur: &str) -> CurationType {
    match cur {
        "HasCuratedSources" => CurationType::Curated,
        "HasAutoCuratedSources" => CurationType::AutoCurated,
        _ => CurationType::NotCurated,
    }
}

impl ToResultData for VerifiedData {
    fn to_result_data(&self) -> ResultData {
        let matched_canonical = match &self.canonical_name {
            None => None,
            Some(can) => Some(can.value_ranked.to_owned()),
        };
        let current_name = match &self.accepted_name {
            None => None,
            Some(acc_name) => Some(acc_name.name.value.to_owned()),
        };
        ResultData {
            data_source_title: self.data_source.title.to_owned(),
            data_source_id: self.data_source.id,
            taxon_id: self.taxon_id.to_owned(),
            matched_name: self.name.value.to_owned(),
            matched_canonical,
            current_name,
            classification_path: self.classification.path.to_owned(),
            classification_rank: self.classification.path_ranks.to_owned(),
            classification_ids: self.classification.path_ids.to_owned(),
            edit_distance: self.match_type.verbatim_edit_distance.unwrap_or(0),
            stem_edit_distance: self.match_type.stem_edit_distance.unwrap_or(0),
            match_type: get_match_type(&self.match_type.kind),
            synonym: self.synonym,
            ..Default::default()
        }
    }
}

impl ToResultData for VerifiedPreferredData {
    fn to_result_data(&self) -> ResultData {
        let matched_canonical = match &self.canonical_name {
            None => None,
            Some(can) => Some(can.value_ranked.to_owned()),
        };
        let current_name = match &self.accepted_name {
            None => None,
            Some(acc_name) => Some(acc_name.name.value.to_owned()),
        };
        ResultData {
            data_source_title: self.data_source.title.to_owned(),
            data_source_id: self.data_source.id,
            taxon_id: self.taxon_id.to_owned(),
            matched_name: self.name.value.to_owned(),
            matched_canonical,
            current_name,
            classification_path: self.classification.path.to_owned(),
            classification_rank: self.classification.path_ranks.to_owned(),
            classification_ids: self.classification.path_ids.to_owned(),
            edit_distance: self.match_type.verbatim_edit_distance.unwrap_or(0),
            stem_edit_distance: self.match_type.stem_edit_distance.unwrap_or(0),
            match_type: get_match_type(&self.match_type.kind),
            synonym: self.synonym,
            ..Default::default()
        }
    }
}

fn get_match_type(match_type: &str) -> MatchType {
    match match_type {
        "ExactMatch" | "ExactCanonicalMatch" => MatchType::Exact,
        "FuzzyCanonicalMatch" => MatchType::Fuzzy,
        "ExactPartialMatch" => MatchType::PartialExact,
        "FuzzyPartialMatch" => MatchType::PartialFuzzy,
        _ => MatchType::NoMatch,
    }
}

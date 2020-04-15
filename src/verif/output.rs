use super::OutputCSV;
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
    #[serde(skip_serializing_if = "is_zero")]
    pub data_sources_num: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_source_curation: Option<CurationType>,
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

    pub fn to_csv(&self) -> Vec<OutputCSV> {
        let mut len = 1;
        if self.preferred_results.is_some() {
            len += self.preferred_results.as_ref().unwrap().len();
        }
        let mut res: Vec<OutputCSV> = Vec::with_capacity(len);
        let mut o_csv = OutputCSV {
            kind: "BestMatch".to_owned(),
            supplied_id: self.id.clone(),
            scientific_name: self.name.clone(),
            ..Default::default()
        };
        if let Some(best) = self.best_result.as_ref() {
            o_csv.matched_name = Some(best.matched_name.clone());
            o_csv.matched_canonical = best.matched_canonical.clone();
            o_csv.taxon_id = Some(best.taxon_id.clone());
            o_csv.current_name = best.current_name.clone();
            o_csv.edit_distance = Some(best.edit_distance);
            o_csv.data_source_id = Some(best.data_source_id);
            o_csv.data_source_title = Some(trim(best.data_source_title.clone()));
            o_csv.classification_path = best.classification_path.clone();
            o_csv.match_type = best.match_type.clone();
        };
        res.push(o_csv);
        if let Some(pref) = self.preferred_results.as_ref() {
            for p in pref {
                let o_csv = OutputCSV {
                    kind: "PreferredMatch".to_owned(),
                    supplied_id: self.id.clone(),
                    scientific_name: self.name.clone(),
                    matched_name: Some(p.matched_name.clone()),
                    matched_canonical: p.matched_canonical.clone(),
                    taxon_id: Some(p.taxon_id.clone()),
                    current_name: p.current_name.clone(),
                    edit_distance: Some(p.edit_distance),
                    data_source_id: Some(p.data_source_id),
                    data_source_title: Some(trim(p.data_source_title.clone())),
                    classification_path: p.classification_path.clone(),
                    match_type: p.match_type.clone(),
                };
                res.push(o_csv);
            }
        }
        res
    }
}

fn trim(s: String) -> String {
    let limit = 40;
    if s.len() <= limit {
        return s;
    }
    format!("{}...", s[0..limit].to_owned())
}

fn get_curation(cur: &str) -> Option<CurationType> {
    match cur {
        "HasCuratedSources" => Some(CurationType::Curated),
        "HasAutoCuratedSources" => Some(CurationType::AutoCurated),
        "Unknown" => Some(CurationType::NotCurated),
        _ => None,
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

fn is_zero(i: &i64) -> bool {
    *i == 0
}

use actix_web::{get, web, HttpResponse, Responder};
use indexa::{
    database::{Database, StatusKind},
    enum_map::EnumMap,
    mode::Mode,
    query::{CaseSensitivity, MatchPathMode, QueryBuilder, SortOrder},
};
use serde::{de::IntoDeserializer, Deserialize, Serialize};
use std::{path::PathBuf, time::SystemTime};

#[derive(Debug, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct SearchParams {
    query: String,
    limit: usize,
    statuses: String,
    #[serde(with = "MatchPathModeDef")]
    match_path: MatchPathMode,
    #[serde(with = "CaseSensitivityDef")]
    case_sensitivity: CaseSensitivity,
    regex: bool,
    sort_by: StatusKind,
    sort_order: SortOrder,
    sort_dirs_before_files: bool,
}

impl Default for SearchParams {
    fn default() -> Self {
        Self {
            query: String::new(),
            limit: 30,
            statuses: "basename".to_string(),
            match_path: MatchPathMode::Never,
            case_sensitivity: CaseSensitivity::Smart,
            regex: false,
            sort_by: StatusKind::Basename,
            sort_order: SortOrder::Ascending,
            sort_dirs_before_files: false,
        }
    }
}

#[derive(Deserialize)]
#[serde(remote = "MatchPathMode", rename_all = "camelCase")]
enum MatchPathModeDef {
    Always,
    Never,
    Auto,
}

#[derive(Deserialize)]
#[serde(remote = "CaseSensitivity", rename_all = "camelCase")]
enum CaseSensitivityDef {
    Sensitive,
    Insensitive,
    Smart,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SearchResponse {
    query: String,
    num_hits: usize,
    hits: Vec<Hit>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Hit {
    is_dir: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    basename: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<PathBuf>,

    #[serde(skip_serializing_if = "Option::is_none")]
    extension: Option<Option<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    size: Option<Option<u64>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    mode: Option<Option<Mode>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    created: Option<MaybeTimestamp>,

    #[serde(skip_serializing_if = "Option::is_none")]
    modified: Option<MaybeTimestamp>,

    #[serde(skip_serializing_if = "Option::is_none")]
    accessed: Option<MaybeTimestamp>,

    highlighted: Highlighted,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Highlighted {
    #[serde(skip_serializing_if = "Option::is_none")]
    basename: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<String>,
}

struct MaybeTimestamp(Option<SystemTime>);

impl From<indexa::Result<SystemTime>> for MaybeTimestamp {
    fn from(t: indexa::Result<SystemTime>) -> Self {
        Self(t.ok())
    }
}

impl Serialize for MaybeTimestamp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let duration = self
            .0
            .map(|t| t.duration_since(SystemTime::UNIX_EPOCH))
            .transpose()
            .map_err(serde::ser::Error::custom)?;
        if let Some(duration) = duration {
            serializer.serialize_some(&duration.as_millis())
        } else {
            serializer.serialize_none()
        }
    }
}

#[get("/search")]
pub async fn service(
    params: web::Query<SearchParams>,
    database: web::Data<Database>,
) -> impl Responder {
    let query = QueryBuilder::new(&params.query)
        .match_path_mode(params.match_path)
        .case_sensitivity(params.case_sensitivity)
        .regex(params.regex)
        .sort_by(params.sort_by)
        .sort_order(params.sort_order)
        .sort_dirs_before_files(params.sort_dirs_before_files)
        .build()
        .expect("Failed to build query");

    let database_cloned = database.clone();
    let queue_cloned = query.clone();
    let hits = tokio::task::spawn_blocking(move || {
        database_cloned
            .search(&queue_cloned)
            .expect("Failed to search")
    })
    .await
    .expect("Failed to search");
    let num_hits = hits.len();

    let status_flags = extract_status_flags(&params.statuses);

    let hits: Vec<Hit> = hits
        .into_iter()
        .take(params.limit)
        .map(|id| {
            let entry = database.entry(id);
            Hit {
                is_dir: entry.is_dir(),
                basename: status_flags[StatusKind::Basename].then(|| entry.basename().to_string()),
                path: status_flags[StatusKind::Path].then(|| entry.path()),
                extension: status_flags[StatusKind::Extension]
                    .then(|| entry.extension().map(str::to_string)),
                size: status_flags[StatusKind::Size].then(|| entry.size().ok()),
                mode: status_flags[StatusKind::Mode].then(|| entry.mode().ok()),
                created: status_flags[StatusKind::Created].then(|| entry.created().into()),
                modified: status_flags[StatusKind::Modified].then(|| entry.modified().into()),
                accessed: status_flags[StatusKind::Accessed].then(|| entry.accessed().into()),
                highlighted: Highlighted {
                    basename: status_flags[StatusKind::Basename].then(|| {
                        highlight_text(entry.basename(), &query.basename_matches(&entry).unwrap())
                    }),
                    path: status_flags[StatusKind::Path].then(|| {
                        highlight_text(
                            entry.path().to_str().unwrap(),
                            &query.path_matches(&entry).unwrap(),
                        )
                    }),
                },
            }
        })
        .collect();

    HttpResponse::Ok().json(SearchResponse {
        query: params.query.clone(),
        num_hits,
        hits,
    })
}

fn extract_status_flags(param: &str) -> EnumMap<StatusKind, bool> {
    let param = param.to_ascii_lowercase();

    let statuses = param.split(',').map(|x| {
        let deserializer: serde::de::value::StrDeserializer<'_, serde::de::value::Error> =
            x.trim().into_deserializer();
        StatusKind::deserialize(deserializer)
    });

    let mut status_flags = EnumMap::default();
    for kind in statuses.flatten() {
        status_flags[kind] = true;
    }

    status_flags
}

fn highlight_text(text: &str, matches: &[std::ops::Range<usize>]) -> String {
    let mut prev_end = 0;
    let mut highlighted = String::new();
    for m in matches {
        if m.start > prev_end {
            highlighted.push_str(&text[prev_end..m.start]);
        }
        if m.end > m.start {
            highlighted.push_str("<em>");
            highlighted.push_str(&text[m.start..m.end]);
            highlighted.push_str("</em>");
        }
        prev_end = m.end;
    }
    if prev_end < text.len() {
        highlighted.push_str(&text[prev_end..]);
    }
    highlighted
}

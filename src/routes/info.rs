use actix_web::{get, web, HttpResponse, Responder};
use indexa::{
    camino::Utf8PathBuf,
    database::{Database, StatusKind},
    strum::IntoEnumIterator,
};
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct InfoResponse {
    num_entries: usize,
    root_dirs: Vec<Utf8PathBuf>,
    indexed: Vec<StatusKind>,
    fast_sortable: Vec<StatusKind>,
}

#[get("/info")]
pub async fn service(database: web::Data<Database>) -> impl Responder {
    let root_dirs = database.root_entries().map(|e| e.path()).collect();

    let mut indexed = Vec::new();
    let mut fast_sortable = Vec::new();
    for kind in StatusKind::iter() {
        if database.is_indexed(kind) {
            indexed.push(kind);
        }
        if database.is_fast_sortable(kind) {
            fast_sortable.push(kind);
        }
    }

    HttpResponse::Ok().json(InfoResponse {
        num_entries: database.num_entries(),
        root_dirs,
        indexed,
        fast_sortable,
    })
}

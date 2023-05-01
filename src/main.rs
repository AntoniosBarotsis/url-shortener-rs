// This warning is thrown in the `axum` function because `PgPool` is considered expensive to drop.
// It would only be dropped if the app is configured improperly and is thus unable to run so
// ignoring the warning is fine. I couldn't get this to work with a local allow for some reason
// (probably because of the shuttle macro) and so I just used a global allow instead.
#![allow(clippy::significant_drop_tightening)]

use std::{fs::read_to_string, path::PathBuf};

use axum::{
  extract::{Path, State},
  http::StatusCode,
  response::{IntoResponse, Redirect},
  routing::{get, post},
  Json, Router,
};
use serde::Serialize;
use shuttle_secrets::SecretStore;
use sqlx::{Executor, FromRow, PgPool};
use tracing::error;
use url::Url;

static BASE_URL: &str = "https://url-shortener-rs.shuttleapp.rs";

#[shuttle_runtime::main]
async fn axum(
  #[shuttle_shared_db::Postgres] pool: PgPool,
  #[shuttle_secrets::Secrets] secret_store: SecretStore,
  #[shuttle_static_folder::StaticFolder(folder = "sql")] static_folder: PathBuf,
) -> shuttle_axum::ShuttleAxum {
  let schema = secret_store
    .get("SCHEMA")
    .expect("SCHEMA secret not found.");

  let schema =
    read_to_string(static_folder.as_path().join(schema)).expect("SCHEMA is read correctly.");

  if let Err(err) = pool.execute(schema.as_str()).await {
    error!("Failed to execute schema: {}", err);
    panic!("Failed to execute schema.");
  }

  let router = Router::new()
    .route("/:id", get(retrieve))
    .route("/shorten", post(shorten))
    .route("/metadata/:id", get(get_metadata))
    .route("/help", get(|| async { help() }))
    .with_state(AppState { pool });

  Ok(router.into())
}

async fn shorten(State(state): State<AppState>, url: String) -> impl IntoResponse {
  let id = &nanoid::nanoid!(6);

  // Extract url if valid, otherwise return error
  let p_url = match Url::parse(&url) {
    Ok(url) => url,
    Err(e) => return Err((StatusCode::BAD_REQUEST, e.to_string())),
  };

  // Insert row in the urls table.
  let res = sqlx::query("INSERT INTO urls(id, url) VALUES ($1, $2)")
    .bind(id)
    .bind(p_url.as_str())
    .execute(&state.pool)
    .await;

  // Insert row in the metadata table.
  let _meta: Metadata =
    sqlx::query_as("INSERT INTO metadata(id, url, hits) VALUES ($1, $2, 0) RETURNING *")
      .bind(id)
      .bind(url)
      .fetch_one(&state.pool)
      .await
      .expect("Creation did not fail.");

  match res {
    Ok(_) => Ok((
      StatusCode::CREATED,
      Json(CreatedURL {
        url: format!("{BASE_URL}/{id}"),
      }),
    )),
    Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
  }
}

async fn retrieve(id: Path<String>, State(state): State<AppState>) -> impl IntoResponse {
  let res: Result<StoredURL, sqlx::Error> = sqlx::query_as("SELECT * FROM urls WHERE id = $1")
    .bind(&id.0)
    .fetch_one(&state.pool)
    .await;

  match res {
    Ok(res) => {
      let _meta = update_metadata(&id.0, &state.pool).await;
      Ok(Redirect::to(&res.url))
    }
    Err(_) => Err((
      StatusCode::NOT_FOUND,
      format!("Entry \"{}\" not found.", &id.0),
    )),
  }
}

async fn get_metadata(id: Path<String>, State(state): State<AppState>) -> impl IntoResponse {
  let meta: Result<Metadata, sqlx::Error> = sqlx::query_as("SELECT * FROM metadata WHERE id = $1")
    .bind(&id.0)
    .fetch_one(&state.pool)
    .await;

  match meta {
    Ok(meta) => Ok((StatusCode::OK, Json(meta))),
    Err(_) => Err((
      StatusCode::NOT_FOUND,
      format!("Entry \"{}\" not found.", &id.0),
    )),
  }
}

async fn update_metadata(id: &str, pool: &PgPool) -> Metadata {
  let meta: Result<Metadata, sqlx::Error> =
    sqlx::query_as("UPDATE metadata SET hits = hits + 1 WHERE id = $1 RETURNING *")
      .bind(id)
      .fetch_one(pool)
      .await;

  meta.expect("Something went wrong with the metadata id, please open a bug report.")
}

fn help() -> impl IntoResponse {
  r#"
  [POST] /shorten      - Shortens a URL                  | Body should contain the URL in raw text.
  [GET]  /:id          - Redirects to the URL
  [GET]  /metadata/:id - Returns the metadata of the URL"#
}

#[derive(Clone)]
struct AppState {
  pub pool: PgPool,
}

#[derive(Serialize, FromRow)]
struct StoredURL {
  pub id: String,
  pub url: String,
}

#[derive(Serialize, FromRow, Debug)]
struct Metadata {
  pub id: String,
  pub url: String,
  pub hits: i32,
}

#[derive(Serialize)]
struct CreatedURL {
  pub url: String,
}

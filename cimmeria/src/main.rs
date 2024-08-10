use std::{env, future::Future, sync::Arc};

use axum::{Extension, Router};
use chrono::{DateTime, Utc};
use database::BasicRecipeRepository;
use tower_http::trace::TraceLayer;

mod api;
mod database;

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Hash, serde::Deserialize)]
struct RecipeIdentifier {
    name: String,
    version: String,
    user: String,
    channel: String,
}

impl std::fmt::Display for RecipeIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            name,
            version,
            user,
            channel,
        } = self;
        write!(f, "{name}/{version}@{user}/{channel}")
    }
}

#[derive(serde::Serialize, Clone)]
pub struct Recipe {
    pub revision: String,
    pub time: DateTime<Utc>,
}

#[derive(serde::Serialize, Clone)]
pub struct Package {
    pub revision: String,
    pub time: DateTime<Utc>,
}

#[derive(thiserror::Error, Debug)]
pub enum RecipeLookupError {
    #[error("not found")]
    NotFound,
    #[error("database error")]
    DatabaseError,
}

trait RecipeRepository {
    fn get_latest_recipe(
        &mut self,
        recipe: RecipeIdentifier,
    ) -> impl Future<Output = Result<Recipe, RecipeLookupError>> + Send;
    fn get_recipe(
        &mut self,
        recipe: RecipeIdentifier,
        revision: String,
    ) -> impl (Future<Output = Result<Recipe, RecipeLookupError>>) + Send;

    fn get_latest_package(
        &mut self,
        recipe_revision: String,
        package_reference: String,
    ) -> impl Future<Output = Result<Package, RecipeLookupError>> + Send;
    fn get_package(
        &mut self,
        package_revision: String,
    ) -> impl Future<Output = Result<Package, RecipeLookupError>> + Send;
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();
    let db_url = env::var("REPO_DB_URL").expect("REPO_DB_URL must be set in the environment");
    let static_base_url =
        env::var("STATIC_BASE_URL").expect("STATIC_BASE_URL must be set in the environment");
    let state = BasicRecipeRepository::new(&db_url).await;
    let app = Router::new()
        .route(
            "/v2/conans/:name/:version/:user/:channel/latest",
            axum::routing::get(api::recipe_latest::<BasicRecipeRepository>),
        )
        .route(
            "/v2/conans/:name/:version/:user/:channel/revisions/:revision/files",
            axum::routing::get(api::recipe_files::<BasicRecipeRepository>),
        )
        .route(
            "/v2/conans/:name/:version/:user/:channel/revisions/:revision/files/*filename",
            axum::routing::get(api::static_recipe_file),
        )
        .route(
            "/v2/conans/:name/:version/:user/:channel/revisions/:recipe_revision/packages/:package_reference/latest",
            axum::routing::get(api::latest_package::<BasicRecipeRepository>)
        )
        .route(
            "/v2/conans/:name/:version/:user/:channel/revisions/:recipe_revision/packages/:package_reference/revisions/:package_revision/files",
            axum::routing::get(api::package_files::<BasicRecipeRepository>)
        )
        .route(
            "/v2/conans/:name/:version/:user/:channel/revisions/:recipe_revision/packages/:package_reference/revisions/:package_revision/files/*filename",
            axum::routing::get(api::static_package_file),
        )
        .route("/v1/ping", axum::routing::get(api::ping))
        .with_state(state)
        .layer(Extension(Arc::new(static_base_url.to_string())))
        .layer(TraceLayer::new_for_http());
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

use std::collections::HashMap;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
    Json,
};

use crate::{RecipeIdentifier, RecipeLookupError, RecipeRepository};

#[derive(Debug)]
struct MissingParameterError(String);
impl std::error::Error for MissingParameterError {}
impl std::fmt::Display for MissingParameterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "missing parameter: {}", self.0)
    }
}

impl From<MissingParameterError> for StatusCode {
    fn from(_: MissingParameterError) -> Self {
        StatusCode::BAD_REQUEST
    }
}

trait FromParameters {
    fn from_parameters(parameters: &HashMap<String, String>) -> Result<Self, MissingParameterError>
    where
        Self: Sized;
}

impl FromParameters for RecipeIdentifier {
    fn from_parameters(parameters: &HashMap<String, String>) -> Result<Self, MissingParameterError>
    where
        Self: Sized,
    {
        Ok(RecipeIdentifier {
            name: parameters
                .get(&"name".to_string())
                .ok_or_else(|| MissingParameterError("name".to_string()))?
                .to_string(),
            version: parameters
                .get(&"version".to_string())
                .ok_or_else(|| MissingParameterError("version".to_string()))?
                .to_string(),
            user: parameters
                .get(&"user".to_string())
                .ok_or_else(|| MissingParameterError("user".to_string()))?
                .to_string(),
            channel: parameters
                .get(&"channel".to_string())
                .ok_or_else(|| MissingParameterError("channel".to_string()))?
                .to_string(),
        })
    }
}

pub async fn recipe_latest<R>(
    State(mut repository): State<R>,
    Path(identifier): Path<RecipeIdentifier>,
) -> impl IntoResponse
where
    R: RecipeRepository + Send + 'static,
{
    repository
        .get_latest_recipe(identifier)
        .await
        .map(Json)
        .map_err(Into::<StatusCode>::into)
}

#[derive(Clone, Copy, Default, serde::Serialize)]
pub struct FilePath {}

#[derive(Clone, Default, serde::Serialize)]
pub struct RecipeFiles {
    #[serde(rename = "conan_export.tgz")]
    export: FilePath,
    #[serde(rename = "conan_sources.tgz")]
    sources: FilePath,
    #[serde(rename = "conanmanifest.txt")]
    manifest: FilePath,
    #[serde(rename = "conanfile.py")]
    conanfile: FilePath,
}

#[derive(Default, serde::Serialize)]
pub struct RecipeRevisionFiles {
    files: RecipeFiles,
}

impl From<RecipeLookupError> for axum::http::StatusCode {
    fn from(value: RecipeLookupError) -> Self {
        match value {
            RecipeLookupError::NotFound => StatusCode::NOT_FOUND,
            RecipeLookupError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

pub async fn recipe_files<R>(
    State(mut repository): State<R>,
    Path(parameters): Path<HashMap<String, String>>,
) -> impl IntoResponse
where
    R: RecipeRepository + Send + 'static,
{
    let identifier =
        RecipeIdentifier::from_parameters(&parameters).map_err(Into::<StatusCode>::into)?;
    let revision = parameters
        .get("revision")
        .ok_or(StatusCode::BAD_REQUEST)?
        .clone();
    repository
        .get_recipe(identifier, revision)
        .await
        .map(|_| Json(RecipeRevisionFiles::default()))
        .map_err(Into::<StatusCode>::into)
}

pub async fn static_recipe_file(
    Path(parameters): Path<HashMap<String, String>>,
) -> impl IntoResponse {
    let revision = parameters.get("revision").ok_or(StatusCode::BAD_REQUEST)?;
    let filename = parameters.get("filename").ok_or(StatusCode::BAD_REQUEST)?;
    Ok::<_, StatusCode>(Redirect::temporary(&format!(
        "http://localhost:8000/{revision}/{filename}"
    )))
}

pub async fn latest_package<R>(
    State(mut repository): State<R>,
    Path(parameters): Path<HashMap<String, String>>,
) -> impl IntoResponse
where
    R: RecipeRepository + Send + 'static,
{
    let recipe_revision = parameters
        .get("recipe_revision")
        .ok_or(StatusCode::BAD_REQUEST)?
        .clone();
    let package_reference = parameters
        .get("package_reference")
        .ok_or(StatusCode::BAD_REQUEST)?
        .clone();
    repository
        .get_latest_package(recipe_revision, package_reference)
        .await
        .map(Json)
        .map_err(Into::<StatusCode>::into)
}

#[derive(Default, serde::Serialize)]
pub struct PackageFiles {
    #[serde(rename = "conaninfo.txt")]
    info: FilePath,
    #[serde(rename = "conan_package.tgz")]
    package: FilePath,
    #[serde(rename = "conanmanifest.txt")]
    manifest: FilePath,
}

#[derive(Default, serde::Serialize)]
pub struct PackageRevisionFiles {
    files: PackageFiles,
}

pub async fn package_files<R>(
    State(mut repository): State<R>,
    Path(parameters): Path<HashMap<String, String>>,
) -> impl IntoResponse
where
    R: RecipeRepository + Send + 'static,
{
    let package_revision = parameters
        .get("package_revision")
        .ok_or(StatusCode::BAD_REQUEST)?
        .clone();
    repository
        .get_package(package_revision)
        .await
        .map(|_| Json(PackageRevisionFiles::default()))
        .map_err(Into::<StatusCode>::into)
}

pub async fn static_package_file(
    Path(parameters): Path<HashMap<String, String>>,
) -> impl IntoResponse {
    let revision = parameters
        .get("package_revision")
        .ok_or(StatusCode::BAD_REQUEST)?;
    let filename = parameters.get("filename").ok_or(StatusCode::BAD_REQUEST)?;
    Ok::<_, StatusCode>(Redirect::temporary(&format!(
        "http://localhost:8000/{revision}/{filename}"
    )))
}

pub async fn ping() -> impl IntoResponse {
    (
        StatusCode::OK,
        [("X-Conan-Server-Capabilities", "revisions")],
    )
}

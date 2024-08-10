use chrono::{DateTime, NaiveDateTime, Utc};
use sqlx::SqlitePool;

use crate::{RecipeIdentifier, RecipeLookupError, RecipeRepository};

#[derive(sqlx::FromRow, Clone)]
struct Recipe {
    #[allow(dead_code)]
    reference: String,

    revision: String,
    created: String,
}

fn date(value: &str) -> DateTime<Utc> {
    NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S")
        .unwrap()
        .and_local_timezone(Utc)
        .unwrap()
}

impl From<Recipe> for crate::Recipe {
    fn from(value: Recipe) -> Self {
        let Recipe {
            revision, created, ..
        } = value;
        let time = date(&created);
        Self { revision, time }
    }
}

#[derive(sqlx::FromRow, Clone)]
struct Package {
    #[allow(dead_code)]
    reference: String,

    revision: String,

    #[allow(dead_code)]
    recipe: String,

    created: String,
}

impl From<Package> for crate::Package {
    fn from(value: Package) -> Self {
        let Package {
            revision, created, ..
        } = value;
        let time = date(&created);
        Self { revision, time }
    }
}

impl From<sqlx::Error> for RecipeLookupError {
    fn from(value: sqlx::Error) -> Self {
        match value {
            sqlx::Error::RowNotFound => RecipeLookupError::NotFound,
            _ => RecipeLookupError::DatabaseError,
        }
    }
}

#[derive(Clone)]
pub struct BasicRecipeRepository(SqlitePool);

impl BasicRecipeRepository {
    pub async fn new(db_path: &str) -> Self {
        Self(SqlitePool::connect(db_path).await.unwrap())
    }
}

impl RecipeRepository for BasicRecipeRepository {
    async fn get_latest_recipe(
        &mut self,
        identifier: RecipeIdentifier,
    ) -> Result<crate::Recipe, RecipeLookupError> {
        sqlx::query_as::<_, Recipe>("SELECT * FROM recipes WHERE reference=$1")
            .bind(identifier.to_string())
            .fetch_all(&self.0)
            .await?
            .iter()
            .reduce(|lhs, rhs| if lhs.created > rhs.created { lhs } else { rhs })
            .cloned()
            .map(Into::<crate::Recipe>::into)
            .ok_or(RecipeLookupError::NotFound)
    }

    async fn get_recipe(
        &mut self,
        identifier: RecipeIdentifier,
        revision: String,
    ) -> Result<crate::Recipe, RecipeLookupError> {
        sqlx::query_as::<_, Recipe>("SELECT * FROM recipes WHERE reference=$1 AND revision=$2")
            .bind(identifier.to_string())
            .bind(revision)
            .fetch_one(&self.0)
            .await
            .map(Into::<_>::into)
            .map_err(Into::<_>::into)
    }

    async fn get_latest_package(
        &mut self,
        recipe_revision: String,
        package_reference: String,
    ) -> Result<crate::Package, RecipeLookupError> {
        sqlx::query_as::<_, Package>(
            "SELECT * FROM packages WHERE reference=$1 AND recipe=$2 ORDER BY created",
        )
        .bind(package_reference)
        .bind(recipe_revision)
        .fetch_one(&self.0)
        .await
        .map(Into::<_>::into)
        .map_err(Into::<_>::into)
    }

    async fn get_package(
        &mut self,
        package_revision: String,
    ) -> Result<crate::Package, RecipeLookupError> {
        sqlx::query_as::<_, Package>("SELECT * FROM packages WHERE revision=$1")
            .bind(package_revision)
            .fetch_one(&self.0)
            .await
            .map(Into::<_>::into)
            .map_err(Into::<_>::into)
    }
}

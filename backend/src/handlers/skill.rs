use actix_web::{HttpResponse, web};
use chrono::{DateTime, Utc};
use sqlx::PgPool;

#[derive(serde::Deserialize)]
pub struct CreateSkill {
    pub title: String,
    pub description: String,
    pub category_id: i32,
}

#[derive(serde::Deserialize)]
pub struct UpdateSkill {
    pub title: Option<String>,
    pub description: Option<String>,
    pub category_id: Option<i32>,
}

#[derive(serde::Serialize)]
pub struct SkillResponse {
    pub skill_id: i32,
    pub title: String,
    pub description: String,
    pub category_id: i32,
    pub created_at: DateTime<Utc>,
}

pub async fn get_all_skills(pool: web::Data<PgPool>) -> HttpResponse {
    let result = sqlx::query!(
        "SELECT * FROM skills WHERE is_deleted = FALSE ORDER BY created_at DESC"
    )
        .fetch_all(pool.get_ref())
        .await;

    match result {
        Ok(skills) => {
            let skills = skills.into_iter().map(|skill| SkillResponse {
                skill_id: skill.skill_id,
                title: skill.title,
                description: skill.description.expect("No description"),
                category_id: skill.category_id.expect("No category_id"),
                created_at: skill.created_at.expect("1970-01-01T00:00:00Z"),
            }).collect::<Vec<_>>();
            HttpResponse::Ok().json(skills)
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Ошибка: {}", e)),
    }
}

pub async fn get_skill_by_id(
    pool: web::Data<PgPool>,
    skill_id: web::Path<i32>,
) -> HttpResponse {
    let result = sqlx::query!(
        "SELECT * FROM skills WHERE skill_id = $1 AND is_deleted = FALSE",
        *skill_id
    )
        .fetch_one(pool.get_ref())
        .await;

    match result {
        Ok(skill) => {
            let skill = SkillResponse {
                skill_id: skill.skill_id,
                title: skill.title,
                description: skill.description.expect("No description"),
                category_id: skill.category_id.expect("No category_id"),
                created_at: skill.created_at.expect("1970-01-01T00:00:00Z"),
            };
            HttpResponse::Ok().json(skill)
        }
        Err(_) => HttpResponse::NotFound().body("Навык не найден"),
    }
}

pub async fn create_skill(
    pool: web::Data<PgPool>,
    form: web::Json<CreateSkill>,
) -> HttpResponse {
    let result = sqlx::query!(
        r#"
        INSERT INTO skills (title, description, category_id)
        VALUES ($1, $2, $3)
        RETURNING *
        "#,
        form.title,
        form.description,
        form.category_id,
    )
        .fetch_one(pool.get_ref())
        .await;

    match result {
        Ok(skill) => {
            let skill = SkillResponse {
                skill_id: skill.skill_id,
                title: skill.title,
                description: skill.description.expect("No description"),
                category_id: skill.category_id.expect("No category_id"),
                created_at: skill.created_at.expect("1970-01-01T00:00:00Z"),
            };
            HttpResponse::Ok().json(skill)
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Ошибка: {}", e)),
    }
}

pub async fn update_skill(
    pool: web::Data<PgPool>,
    skill_id: web::Path<i32>,
    form: web::Json<UpdateSkill>,
) -> HttpResponse {
    let result = sqlx::query!(
        r#"
        UPDATE skills
        SET
            title = COALESCE($1, title),
            description = COALESCE($2, description),
            category_id = COALESCE($3, category_id)
        WHERE skill_id = $4 AND is_deleted = FALSE
        RETURNING *
        "#,
        form.title,
        form.description,
        form.category_id,
        *skill_id
    )
        .fetch_one(pool.get_ref())
        .await;

    match result {
        Ok(skill) => {
            let skill = SkillResponse {
                skill_id: skill.skill_id,
                title: skill.title,
                description: skill.description.expect("No description"),
                category_id: skill.category_id.expect("No category_id"),
                created_at: skill.created_at.expect("1970-01-01T00:00:00Z"),
            };
            HttpResponse::Ok().json(skill)
        }
        Err(_) => HttpResponse::NotFound().body("Навык не найден"),
    }
}

pub async fn delete_skill(
    pool: web::Data<PgPool>,
    skill_id: web::Path<i32>,
) -> HttpResponse {
    let result = sqlx::query!(
        "UPDATE skills SET is_deleted = TRUE WHERE skill_id = $1",
        *skill_id
    )
        .execute(pool.get_ref())
        .await;

    match result {
        Ok(_) => HttpResponse::Ok().body("Навык помечен как удалённый"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Ошибка: {}", e)),
    }
}
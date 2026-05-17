use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Click {
    pub id: Uuid,
    pub link_id: Uuid,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub referer: Option<String>,
    pub clicked_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct ClickStats {
    pub total_clicks: i64,
    pub clicks_today: i64,
    pub clicks_week: i64,
    pub recent_clicks: Vec<Click>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct DailyClicks {
    pub date: Option<DateTime<Utc>>,
    pub count: Option<i64>,
}

impl Click {
    pub async fn create(
        pool: &PgPool,
        link_id: Uuid,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
        referer: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO clicks (link_id, ip_address, user_agent, referer) VALUES ($1, $2, $3, $4)"
        )
        .bind(link_id)
        .bind(ip_address)
        .bind(user_agent)
        .bind(referer)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn get_stats(pool: &PgPool, link_id: Uuid) -> Result<ClickStats, sqlx::Error> {
        let total_clicks = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM clicks WHERE link_id = $1"
        )
        .bind(link_id)
        .fetch_one(pool)
        .await?;

        let clicks_today = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM clicks WHERE link_id = $1 AND clicked_at >= CURRENT_DATE"
        )
        .bind(link_id)
        .fetch_one(pool)
        .await?;

        let clicks_week = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM clicks WHERE link_id = $1 AND clicked_at >= NOW() - INTERVAL '7 days'"
        )
        .bind(link_id)
        .fetch_one(pool)
        .await?;

        let recent_clicks = sqlx::query_as::<_, Click>(
            "SELECT * FROM clicks WHERE link_id = $1 ORDER BY clicked_at DESC LIMIT 50"
        )
        .bind(link_id)
        .fetch_all(pool)
        .await?;

        Ok(ClickStats {
            total_clicks,
            clicks_today,
            clicks_week,
            recent_clicks,
        })
    }

    pub async fn daily_clicks(
        pool: &PgPool,
        link_id: Uuid,
        days: i32,
    ) -> Result<Vec<DailyClicks>, sqlx::Error> {
        sqlx::query_as::<_, DailyClicks>(
            "SELECT DATE_TRUNC('day', clicked_at) as date, COUNT(*) as count \
             FROM clicks WHERE link_id = $1 AND clicked_at >= NOW() - make_interval(days => $2) \
             GROUP BY date ORDER BY date"
        )
        .bind(link_id)
        .bind(days)
        .fetch_all(pool)
        .await
    }
}

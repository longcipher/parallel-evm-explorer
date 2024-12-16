#[derive(Clone, Debug)]
pub struct DB {
    pub db: sqlx::PgPool,
}

impl DB {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { db: pool }
    }
}

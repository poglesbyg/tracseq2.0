use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct QaqcService {
    pub pool: Arc<PgPool>,
}

impl QaqcService {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool: Arc::new(pool),
        }
    }
}

#[derive(Clone)]
pub struct QualityMetricsService {
    pub pool: Arc<PgPool>,
}

impl QualityMetricsService {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool: Arc::new(pool),
        }
    }
}

#[derive(Clone)]
pub struct ComplianceService {
    pub pool: Arc<PgPool>,
}

impl ComplianceService {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool: Arc::new(pool),
        }
    }
}
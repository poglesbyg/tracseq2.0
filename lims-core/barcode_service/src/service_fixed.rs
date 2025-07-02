// This file contains the fixed SQLx queries
// Copy the contents to service.rs to replace the compile-time macros

// For query_as! replacement:
// Replace this:
//     let stored_barcode = sqlx::query_as!(
//         StoredBarcode,
//         "SELECT * FROM barcodes WHERE barcode = $1",
//         barcode
//     )
// With this:
let stored_barcode = sqlx::query_as::<_, StoredBarcode>(
    "SELECT * FROM barcodes WHERE barcode = $1"
)
.bind(barcode)
.fetch_optional(&self.db_pool)
.await
.map_err(BarcodeError::DatabaseError)?;

// For query_scalar! replacements in get_stats():
// Replace:
//     let total_generated = sqlx::query_scalar!(
//         "SELECT COUNT(*) FROM barcodes"
//     )
// With:
let total_generated: Option<i64> = sqlx::query_scalar(
    "SELECT COUNT(*) FROM barcodes"
)
.fetch_one(&self.db_pool)
.await
.map_err(BarcodeError::DatabaseError)?;

let total_generated = total_generated.unwrap_or(0);

// Replace:
//     let total_reserved = sqlx::query_scalar!(
//         "SELECT COUNT(*) FROM barcodes WHERE is_reserved = true"
//     )
// With:
let total_reserved: Option<i64> = sqlx::query_scalar(
    "SELECT COUNT(*) FROM barcodes WHERE is_reserved = true"
)
.fetch_one(&self.db_pool)
.await
.map_err(BarcodeError::DatabaseError)?;

let total_reserved = total_reserved.unwrap_or(0);

// Replace:
//     let total_unique_prefixes = sqlx::query_scalar!(
//         "SELECT COUNT(DISTINCT prefix) FROM barcodes WHERE prefix IS NOT NULL"
//     )
// With:
let total_unique_prefixes: Option<i64> = sqlx::query_scalar(
    "SELECT COUNT(DISTINCT prefix) FROM barcodes WHERE prefix IS NOT NULL"
)
.fetch_one(&self.db_pool)
.await
.map_err(BarcodeError::DatabaseError)?;

let total_unique_prefixes = total_unique_prefixes.unwrap_or(0);

// Replace:
//     let most_recent_barcode = sqlx::query_scalar!(
//         "SELECT barcode FROM barcodes ORDER BY created_at DESC LIMIT 1"
//     )
// With:
let most_recent_barcode: Option<String> = sqlx::query_scalar(
    "SELECT barcode FROM barcodes ORDER BY created_at DESC LIMIT 1"
)
.fetch_optional(&self.db_pool)
.await
.map_err(BarcodeError::DatabaseError)?;

// Replace:
//     let recent_count = sqlx::query_scalar!(
//         "SELECT COUNT(*) FROM barcodes WHERE created_at >= $1",
//         thirty_days_ago
//     )
// With:
let recent_count: Option<i64> = sqlx::query_scalar(
    "SELECT COUNT(*) FROM barcodes WHERE created_at >= $1"
)
.bind(thirty_days_ago)
.fetch_one(&self.db_pool)
.await
.map_err(BarcodeError::DatabaseError)?;

let recent_count = recent_count.unwrap_or(0);

// For health_check():
// Replace:
//     let count = sqlx::query_scalar!("SELECT COUNT(*) FROM barcodes")
// With:
let count: Option<i64> = sqlx::query_scalar("SELECT COUNT(*) FROM barcodes")
    .fetch_one(&self.db_pool)
    .await
    .map_err(BarcodeError::DatabaseError)?;

Ok(count.unwrap_or(0)) 
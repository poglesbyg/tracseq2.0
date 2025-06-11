use argon2::password_hash::{rand_core::OsRng, PasswordHasher, SaltString};
use argon2::Argon2;
use lab_manager::config::AppConfig;
use sqlx::PgPool;
use std::env;
use std::io::{self, Write};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Lab Manager - Admin User Creation Tool");
    println!("==========================================");

    // Load configuration
    let config = AppConfig::from_env().expect("Failed to load configuration");

    // Connect to database
    let pool = PgPool::connect(&config.database.url)
        .await
        .expect("Failed to connect to database");

    // Get user input
    print!("Enter admin email: ");
    io::stdout().flush()?;
    let mut email = String::new();
    io::stdin().read_line(&mut email)?;
    let email = email.trim();

    print!("Enter admin password: ");
    io::stdout().flush()?;
    let mut password = String::new();
    io::stdin().read_line(&mut password)?;
    let password = password.trim();

    print!("Enter first name: ");
    io::stdout().flush()?;
    let mut first_name = String::new();
    io::stdin().read_line(&mut first_name)?;
    let first_name = first_name.trim();

    print!("Enter last name: ");
    io::stdout().flush()?;
    let mut last_name = String::new();
    io::stdin().read_line(&mut last_name)?;
    let last_name = last_name.trim();

    print!("Enter lab affiliation (optional): ");
    io::stdout().flush()?;
    let mut lab_affiliation = String::new();
    io::stdin().read_line(&mut lab_affiliation)?;
    let lab_affiliation = lab_affiliation.trim();
    let lab_affiliation = if lab_affiliation.is_empty() {
        None
    } else {
        Some(lab_affiliation)
    };

    print!("Enter department (optional): ");
    io::stdout().flush()?;
    let mut department = String::new();
    io::stdin().read_line(&mut department)?;
    let department = department.trim();
    let department = if department.is_empty() {
        None
    } else {
        Some(department)
    };

    // Hash password
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| format!("Password hashing failed: {}", e))?
        .to_string();

    // Insert user into database
    let user_id = Uuid::new_v4();

    let result = sqlx::query(
        r#"
        INSERT INTO users (
            id, email, password_hash, first_name, last_name, role, status,
            email_verified, lab_affiliation, department, position
        ) VALUES (
            $1, $2, $3, $4, $5, 'lab_administrator', 'active',
            TRUE, $6, $7, 'System Administrator'
        )
        "#,
    )
    .bind(user_id)
    .bind(email)
    .bind(password_hash)
    .bind(first_name)
    .bind(last_name)
    .bind(lab_affiliation)
    .bind(department)
    .execute(&pool)
    .await;

    match result {
        Ok(_) => {
            println!("✅ Admin user created successfully!");
            println!("📧 Email: {}", email);
            println!(
                "🔑 Password: {} (please change after first login)",
                password
            );
            println!("👤 Name: {} {}", first_name, last_name);
            println!("🆔 User ID: {}", user_id);

            if let Some(lab) = lab_affiliation {
                println!("🧪 Lab: {}", lab);
            }
            if let Some(dept) = department {
                println!("🏢 Department: {}", dept);
            }

            println!("\n🚀 You can now login at: http://localhost:5173");
        }
        Err(e) => {
            if e.to_string().contains("duplicate key") {
                println!("❌ Error: User with email '{}' already exists!", email);
            } else {
                println!("❌ Error creating user: {}", e);
            }
        }
    }

    Ok(())
}

//! # Pet Store Demo Application with Hodei Verified Permissions
//! 
//! This is a complete example application demonstrating:
//! - SQLite database integration with full CRUD operations
//! - HTML UI with authorization status display
//! - Middleware integration with Hodei Verified Permissions
//! - Role-based access control (pet-admin, pet-vet, pet-customer)
//! - Real-time authorization decision visualization

use anyhow::Result;
use axum::{
    extract::{Path, State, Query},
    routing::{get, post, put, delete},
    response::{Html, Redirect, IntoResponse, Response},
    http::{StatusCode, header},
    Json, Router, Form,
    middleware::{self, Next},
    RequestExt,
};
use dotenvy::dotenv;
use hodei_permissions_sdk::{
    middleware::VerifiedPermissionsLayer, AuthorizationClient, Decision,
    entities::{Entity, EntityBuilder},
    requests::IsAuthorizedRequest,
};
use serde::{Deserialize, Serialize};
use sqlx::{SqlitePool, FromRow, sqlite::SqlitePoolOptions};
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;
use tracing::{info, warn, error};
use tracing_subscriber::{fmt, EnvFilter};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use askama::Template;

// ============================================================================
// DATABASE MODELS
// ============================================================================

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
struct Pet {
    id: String,
    name: String,
    species: String,
    breed: String,
    owner_username: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
struct Appointment {
    id: String,
    pet_id: String,
    pet_name: String,
    vet_username: String,
    notes: String,
    appointment_date: DateTime<Utc>,
    created_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
struct User {
    username: String,
    role: String,
    email: String,
}

// ============================================================================
// REQUEST/RESPONSE DTOS
// ============================================================================

#[derive(Debug, Deserialize)]
struct CreatePetRequest {
    name: String,
    species: String,
    breed: String,
}

#[derive(Debug, Deserialize)]
struct UpdatePetRequest {
    name: Option<String>,
    species: Option<String>,
    breed: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CreateAppointmentRequest {
    pet_id: String,
    vet_username: String,
    notes: String,
    appointment_date: String, // ISO format
}

#[derive(Debug, Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

// ============================================================================
// APPLICATION STATE
// ============================================================================

#[derive(Clone)]
struct AppState {
    db: SqlitePool,
    avp_client: Arc<AuthorizationClient>,
    policy_store_id: String,
    identity_source_id: String,
}

// ============================================================================
// HTML TEMPLATES
// ============================================================================

#[derive(Template)]
#[template(path = "base.html")]
struct BaseTemplate {
    title: String,
    user: Option<User>,
    auth_status: Option<AuthStatus>,
    content: String,
}

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate;

#[derive(Template)]
#[template(path = "dashboard.html")]
struct DashboardTemplate {
    user: User,
    stats: DashboardStats,
}

#[derive(Template)]
#[template(path = "pets.html")]
struct PetsTemplate {
    user: User,
    pets: Vec<Pet>,
    can_create: bool,
    can_delete: bool,
    auth_status: AuthStatus,
}

#[derive(Template)]
#[template(path = "pet_detail.html")]
struct PetDetailTemplate {
    user: User,
    pet: Pet,
    appointments: Vec<Appointment>,
    can_edit: bool,
    can_delete: bool,
    can_create_appointment: bool,
    auth_status: AuthStatus,
}

#[derive(Template)]
#[template(path = "appointments.html")]
struct AppointmentsTemplate {
    user: User,
    appointments: Vec<Appointment>,
    can_create: bool,
    auth_status: AuthStatus,
}

// ============================================================================
// AUTHENTICATION & AUTHORIZATION
// ============================================================================

#[derive(Debug, Clone, Serialize)]
struct AuthStatus {
    decision: String,
    principal: String,
    action: String,
    resource: String,
    policies_evaluated: Vec<String>,
    processing_time_ms: u64,
    timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
struct AuthContext {
    user: User,
    token: String,
    auth_status: Option<AuthStatus>,
}

// Middleware para extraer y verificar el token
async fn auth_middleware(
    State(state): State<AppState>,
    mut req: axum::http::Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extraer token del header o cookie
    let token = extract_token(&req).ok_or(StatusCode::UNAUTHORIZED)?;
    
    // Verificar token con AVP
    let start = std::time::Instant::now();
    let auth_request = IsAuthorizedRequest {
        principal: Some(Entity::new("User", "current_user")),
        action: "Action::\"access\"".to_string(),
        resource: "Resource::\"application\"".to_string(),
        context: serde_json::json!({
            "jwt": token,
            "path": req.uri().path(),
            "method": req.method().to_string(),
        }),
    };
    
    let response = state.avp_client
        .is_authorized(&state.policy_store_id, auth_request)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let processing_time_ms = start.elapsed().as_millis() as u64;
    
    // Crear status de autorización para visualización
    let auth_status = AuthStatus {
        decision: format!("{:?}", response.decision),
        principal: "User::\"current_user\"".to_string(),
        action: "Action::\"access\"".to_string(),
        resource: "Resource::\"application\"".to_string(),
        policies_evaluated: vec!["policy1".to_string(), "policy2".to_string()],
        processing_time_ms,
        timestamp: Utc::now(),
    };
    
    // Extraer usuario del token (en producción, validar JWT)
    let user = extract_user_from_token(&token).ok_or(StatusCode::UNAUTHORIZED)?;
    
    // Añadir contexto a la request
    let context = AuthContext {
        user,
        token,
        auth_status: Some(auth_status),
    };
    
    req.extensions_mut().insert(context);
    
    Ok(next.run(req).await)
}

fn extract_token(req: &axum::http::Request<axum::body::Body>) -> Option<String> {
    // 1. Intentar desde header Authorization
    if let Some(auth_header) = req.headers().get(header::AUTHORIZATION) {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                return Some(token.to_string());
            }
        }
    }
    
    // 2. Intentar desde cookie
    if let Some(cookie_header) = req.headers().get(header::COOKIE) {
        if let Ok(cookie_str) = cookie_header.to_str() {
            for cookie in cookie_str.split(';') {
                let cookie = cookie.trim();
                if let Some(token) = cookie.strip_prefix("token=") {
                    return Some(token.to_string());
                }
            }
        }
    }
    
    None
}

fn extract_user_from_token(token: &str) -> Option<User> {
    // En producción: validar JWT y extraer claims
    // Para demo: mapear token a usuario basado en username
    
    let username = match token {
        "admin_token" => "pet_admin",
        "vet_token" => "pet_vet",
        "customer_token" => "pet_customer",
        _ => return None,
    };
    
    let role = match username {
        "pet_admin" => "pet-admin",
        "pet_vet" => "pet-vet",
        "pet_customer" => "pet-customer",
        _ => "pet-customer",
    };
    
    Some(User {
        username: username.to_string(),
        role: role.to_string(),
        email: format!("{}@petstore.com", username),
    })
}

// ============================================================================
// DATABASE OPERATIONS
// ============================================================================

async fn init_db(pool: &SqlitePool) -> Result<()> {
    // Crear tabla de pets
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS pets (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            species TEXT NOT NULL,
            breed TEXT NOT NULL,
            owner_username TEXT NOT NULL,
            created_at TIMESTAMP NOT NULL,
            updated_at TIMESTAMP NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Crear tabla de appointments
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS appointments (
            id TEXT PRIMARY KEY,
            pet_id TEXT NOT NULL,
            pet_name TEXT NOT NULL,
            vet_username TEXT NOT NULL,
            notes TEXT NOT NULL,
            appointment_date TIMESTAMP NOT NULL,
            created_at TIMESTAMP NOT NULL,
            FOREIGN KEY (pet_id) REFERENCES pets(id)
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Crear tabla de users (para demo)
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            username TEXT PRIMARY KEY,
            role TEXT NOT NULL,
            email TEXT NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

async fn seed_data(pool: &SqlitePool) -> Result<()> {
    // Insertar usuarios de demo
    sqlx::query(
        "INSERT OR IGNORE INTO users (username, role, email) VALUES (?, ?, ?)",
    )
    .bind("pet_admin")
    .bind("pet-admin")
    .bind("pet_admin@petstore.com")
    .execute(pool)
    .await?;

    sqlx::query(
        "INSERT OR IGNORE INTO users (username, role, email) VALUES (?, ?, ?)",
    )
    .bind("pet_vet")
    .bind("pet-vet")
    .bind("pet_vet@petstore.com")
    .execute(pool)
    .await?;

    sqlx::query(
        "INSERT OR IGNORE INTO users (username, role, email) VALUES (?, ?, ?)",
    )
    .bind("pet_customer")
    .bind("pet-customer")
    .bind("pet_customer@petstore.com")
    .execute(pool)
    .await?;

    // Insertar pets de demo
    let pets = vec![
        ("pet-1", "Fido", "Dog", "Labrador", "pet_customer"),
        ("pet-2", "Mittens", "Cat", "Siamese", "pet_customer"),
        ("pet-3", "Rex", "Dog", "German Shepherd", "pet_admin"),
    ];

    for (id, name, species, breed, owner) in pets {
        sqlx::query(
            "INSERT OR IGNORE INTO pets (id, name, species, breed, owner_username, created_at, updated_at) VALUES (?, ?, ?, ?, ?, datetime('now'), datetime('now'))",
        )
        .bind(id)
        .bind(name)
        .bind(species)
        .bind(breed)
        .bind(owner)
        .execute(pool)
        .await?;
    }

    Ok(())
}

// ============================================================================
// HANDLERS - AUTH
// ============================================================================

async fn login_page() -> Html<String> {
    let template = LoginTemplate;
    Html(template.render().unwrap())
}

async fn login_handler(
    State(state): State<AppState>,
    Form(credentials): Form<LoginRequest>,
) -> Result<Response, StatusCode> {
    // Validar credenciales (en producción: contra Keycloak)
    let token = match credentials.username.as_str() {
        "pet_admin" => "admin_token",
        "pet_vet" => "vet_token",
        "pet_customer" => "customer_token",
        _ => return Err(StatusCode::UNAUTHORIZED),
    };

    // Crear cookie con token
    let cookie = format!("token={}; Path=/; HttpOnly; SameSite=Strict", token);
    
    let response = Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header(header::LOCATION, "/dashboard")
        .header(header::SET_COOKIE, cookie)
        .body(axum::body::Body::empty())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(response)
}

async fn logout_handler() -> Result<Response, StatusCode> {
    // Eliminar cookie
    let cookie = "token=; Path=/; HttpOnly; SameSite=Strict; Max-Age=0";
    
    let response = Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header(header::LOCATION, "/login")
        .header(header::SET_COOKIE, cookie)
        .body(axum::body::Body::empty())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(response)
}

// ============================================================================
// HANDLERS - DASHBOARD
// ============================================================================

#[derive(Serialize)]
struct DashboardStats {
    total_pets: i64,
    total_appointments: i64,
    user_role: String,
}

async fn dashboard_handler(
    State(state): State<AppState>,
    Extension(auth_context): Extension<AuthContext>,
) -> Result<Html<String>, StatusCode> {
    // Obtener stats
    let total_pets: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM pets")
        .fetch_one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let total_appointments: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM appointments")
        .fetch_one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let stats = DashboardStats {
        total_pets: total_pets.0,
        total_appointments: total_appointments.0,
        user_role: auth_context.user.role.clone(),
    };

    let template = DashboardTemplate {
        user: auth_context.user,
        stats,
    };

    Ok(Html(template.render().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?))
}

// ============================================================================
// HANDLERS - PETS CRUD
// ============================================================================

async fn list_pets_handler(
    State(state): State<AppState>,
    Extension(auth_context): Extension<AuthContext>,
) -> Result<Html<String>, StatusCode> {
    let pets = sqlx::query_as::<_, Pet>("SELECT * FROM pets ORDER BY created_at DESC")
        .fetch_all(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Verificar permisos
    let can_create = auth_context.user.role == "pet-admin";
    let can_delete = auth_context.user.role == "pet-admin";

    let template = PetsTemplate {
        user: auth_context.user,
        pets,
        can_create,
        can_delete,
        auth_status: auth_context.auth_status.unwrap(),
    };

    Ok(Html(template.render().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?))
}

async fn get_pet_handler(
    State(state): State<AppState>,
    Path(pet_id): Path<String>,
    Extension(auth_context): Extension<AuthContext>,
) -> Result<Html<String>, StatusCode> {
    let pet = sqlx::query_as::<_, Pet>("SELECT * FROM pets WHERE id = ?")
        .bind(&pet_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let appointments = sqlx::query_as::<_, Appointment>(
        "SELECT * FROM appointments WHERE pet_id = ? ORDER BY appointment_date DESC"
    )
    .bind(&pet_id)
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Verificar permisos
    let can_edit = auth_context.user.role == "pet-admin";
    let can_delete = auth_context.user.role == "pet-admin";
    let can_create_appointment = auth_context.user.role == "pet-admin" || 
                                 auth_context.user.role == "pet-vet";

    let template = PetDetailTemplate {
        user: auth_context.user,
        pet,
        appointments,
        can_edit,
        can_delete,
        can_create_appointment,
        auth_status: auth_context.auth_status.unwrap(),
    };

    Ok(Html(template.render().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?))
}

async fn create_pet_handler(
    State(state): State<AppState>,
    Extension(auth_context): Extension<AuthContext>,
    Form(payload): Form<CreatePetRequest>,
) -> Result<Redirect, StatusCode> {
    // Verificar permiso
    if auth_context.user.role != "pet-admin" {
        return Err(StatusCode::FORBIDDEN);
    }

    let pet_id = Uuid::new_v4().to_string();
    
    sqlx::query(
        "INSERT INTO pets (id, name, species, breed, owner_username, created_at, updated_at) VALUES (?, ?, ?, ?, ?, datetime('now'), datetime('now'))",
    )
    .bind(&pet_id)
    .bind(&payload.name)
    .bind(&payload.species)
    .bind(&payload.breed)
    .bind(&auth_context.user.username)
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Redirect::to(&format!("/pets/{}", pet_id)))
}

async fn update_pet_handler(
    State(state): State<AppState>,
    Path(pet_id): Path<String>,
    Extension(auth_context): Extension<AuthContext>,
    Form(payload): Form<UpdatePetRequest>,
) -> Result<Redirect, StatusCode> {
    // Verificar permiso
    if auth_context.user.role != "pet-admin" {
        return Err(StatusCode::FORBIDDEN);
    }

    // Construir query dinámica
    let mut query = "UPDATE pets SET updated_at = datetime('now')".to_string();
    let mut bindings: Vec<String> = vec![];

    if let Some(name) = &payload.name {
        query.push_str(", name = ?");
        bindings.push(name.clone());
    }
    if let Some(species) = &payload.species {
        query.push_str(", species = ?");
        bindings.push(species.clone());
    }
    if let Some(breed) = &payload.breed {
        query.push_str(", breed = ?");
        bindings.push(breed.clone());
    }

    query.push_str(" WHERE id = ?");
    bindings.push(pet_id.clone());

    // Ejecutar query
    let mut query_obj = sqlx::query(&query);
    for binding in bindings {
        query_obj = query_obj.bind(binding);
    }

    query_obj.execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Redirect::to(&format!("/pets/{}", pet_id)))
}

async fn delete_pet_handler(
    State(state): State<AppState>,
    Path(pet_id): Path<String>,
    Extension(auth_context): Extension<AuthContext>,
) -> Result<Redirect, StatusCode> {
    // Verificar permiso
    if auth_context.user.role != "pet-admin" {
        return Err(StatusCode::FORBIDDEN);
    }

    // Eliminar citas primero (FK constraint)
    sqlx::query("DELETE FROM appointments WHERE pet_id = ?")
        .bind(&pet_id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Eliminar pet
    sqlx::query("DELETE FROM pets WHERE id = ?")
        .bind(&pet_id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Redirect::to("/pets"))
}

// ============================================================================
// HANDLERS - APPOINTMENTS CRUD
// ============================================================================

async fn list_appointments_handler(
    State(state): State<AppState>,
    Extension(auth_context): Extension<AuthContext>,
) -> Result<Html<String>, StatusCode> {
    let appointments = sqlx::query_as::<_, Appointment>(
        "SELECT * FROM appointments ORDER BY appointment_date DESC"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let can_create = auth_context.user.role == "pet-admin" || 
                    auth_context.user.role == "pet-vet";

    let template = AppointmentsTemplate {
        user: auth_context.user,
        appointments,
        can_create,
        auth_status: auth_context.auth_status.unwrap(),
    };

    Ok(Html(template.render().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?))
}

async fn create_appointment_handler(
    State(state): State<AppState>,
    Extension(auth_context): Extension<AuthContext>,
    Form(payload): Form<CreateAppointmentRequest>,
) -> Result<Redirect, StatusCode> {
    // Verificar permiso
    if auth_context.user.role != "pet-admin" && auth_context.user.role != "pet-vet" {
        return Err(StatusCode::FORBIDDEN);
    }

    // Verificar que el pet existe
    let pet_exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM pets WHERE id = ?)")
        .bind(&payload.pet_id)
        .fetch_one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !pet_exists {
        return Err(StatusCode::BAD_REQUEST);
    }

    let appointment_id = Uuid::new_v4().to_string();
    let pet_name: String = sqlx::query_scalar("SELECT name FROM pets WHERE id = ?")
        .bind(&payload.pet_id)
        .fetch_one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Parsear fecha
    let appointment_date = DateTime::parse_from_rfc3339(&payload.appointment_date)
        .map_err(|_| StatusCode::BAD_REQUEST)?
        .with_timezone(&Utc);

    sqlx::query(
        "INSERT INTO appointments (id, pet_id, pet_name, vet_username, notes, appointment_date, created_at) VALUES (?, ?, ?, ?, ?, ?, datetime('now'))",
    )
    .bind(&appointment_id)
    .bind(&payload.pet_id)
    .bind(&pet_name)
    .bind(&payload.vet_username)
    .bind(&payload.notes)
    .bind(&appointment_date)
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Redirect::to(&format!("/pets/{}", payload.pet_id)))
}

// ============================================================================
// MAIN
// ============================================================================

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    
    // Inicializar logging
    fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // Configuración
    let avp_endpoint = std::env::var("AVP_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:50051".to_string());
    let policy_store_id = std::env::var("POLICY_STORE_ID")
        .unwrap_or_else(|_| "demo-policy-store".to_string());
    let identity_source_id = std::env::var("IDENTITY_SOURCE_ID")
        .unwrap_or_else(|_| "demo-identity-source".to_string());
    let db_path = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:./petstore.db".to_string());
    let addr: SocketAddr = std::env::var("DEMO_HTTP_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:3000".to_string())
        .parse()?;

    info!("Conectando a base de datos: {}", db_path);
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_path)
        .await?;

    // Inicializar base de datos
    init_db(&pool).await?;
    seed_data(&pool).await?;

    info!("Conectando a AVP en {}", avp_endpoint);
    let client = AuthorizationClient::connect(&avp_endpoint).await?;
    let client_arc = Arc::new(client);

    let state = AppState {
        db: pool,
        avp_client: client_arc,
        policy_store_id,
        identity_source_id,
    };

    // Crear middleware de autorización
    let auth_layer = VerifiedPermissionsLayer::new(
        state.avp_client.clone(),
        state.policy_store_id.clone(),
        state.identity_source_id.clone(),
    );

    // Configurar router
    let app = Router::new()
        // Auth routes
        .route("/login", get(login_page).post(login_handler))
        .route("/logout", get(logout_handler))
        
        // Dashboard
        .route("/dashboard", get(dashboard_handler))
        
        // Pets CRUD
        .route("/pets", get(list_pets_handler).post(create_pet_handler))
        .route("/pets/:id", get(get_pet_handler))
        .route("/pets/:id/edit", post(update_pet_handler))
        .route("/pets/:id/delete", post(delete_pet_handler))
        
        // Appointments
        .route("/appointments", get(list_appointments_handler))
        .route("/appointments/create", post(create_appointment_handler))
        
        // Health check
        .route("/health", get(|| async { "OK" }))
        
        // Static files
        .route("/static/style.css", get(|| async {
            Html(include_str!("../templates/style.css"))
        }))
        
        .with_state(state)
        .layer(middleware::from_fn(auth_middleware))
        .layer(auth_layer);

    info!("Pet Store demo escuchando en {}", addr);
    info!("URLs disponibles:");
    info!("  - Login: http://{}/login", addr);
    info!("  - Dashboard: http://{}/dashboard", addr);
    info!("  - Pets: http://{}/pets", addr);
    info!("  - Appointments: http://{}/appointments", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

use anyhow::Result;
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use dotenvy::dotenv;
use hodei_permissions_sdk::{middleware::VerifiedPermissionsLayer, AuthorizationClient};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;
use tracing::{info, warn};
use tracing_subscriber::{fmt, EnvFilter};
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    pets: Arc<RwLock<HashMap<String, Pet>>>,
    appointments: Arc<RwLock<Vec<Appointment>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Pet {
    id: String,
    name: String,
    species: String,
    breed: String,
    owner_username: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Appointment {
    id: String,
    pet_id: String,
    vet_username: String,
    notes: String,
}

#[derive(Debug, Deserialize)]
struct NewPet {
    name: String,
    species: String,
    breed: String,
    owner_username: String,
}

#[derive(Debug, Deserialize)]
struct NewAppointment {
    vet_username: String,
    notes: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let avp_endpoint = std::env::var("AVP_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:50051".to_string());
    let policy_store_id = std::env::var("POLICY_STORE_ID")
        .unwrap_or_else(|_| "demo-policy-store".to_string());
    let identity_source_id = std::env::var("IDENTITY_SOURCE_ID")
        .unwrap_or_else(|_| "demo-identity-source".to_string());
    let addr: SocketAddr = std::env::var("DEMO_HTTP_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:3000".to_string())
        .parse()?;

    info!("Conectando a AVP en {}", avp_endpoint);
    let client = AuthorizationClient::connect(&avp_endpoint).await?;
    let layer = VerifiedPermissionsLayer::new(
        client,
        policy_store_id,
        identity_source_id,
    );

    let state = AppState {
        pets: Arc::new(RwLock::new(seed_pets())),
        appointments: Arc::new(RwLock::new(Vec::new())),
    };

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/pets", get(list_pets).post(create_pet))
        .route("/pets/:pet_id", get(get_pet))
        .route(
            "/pets/:pet_id/appointments",
            get(list_appointments_for_pet).post(schedule_appointment),
        )
        .with_state(state)
        .layer(layer);

    info!("Pet Store demo escuchando en {}", addr);
    axum::Server::bind(&addr).serve(app.into_make_service()).await?;
    Ok(())
}

async fn health_check() -> &'static str {
    "OK"
}

async fn list_pets(State(state): State<AppState>) -> Json<Vec<Pet>> {
    let pets = state.pets.read().await;
    Json(pets.values().cloned().collect())
}

async fn get_pet(State(state): State<AppState>, Path(pet_id): Path<String>) -> Option<Json<Pet>> {
    let pets = state.pets.read().await;
    pets.get(&pet_id).cloned().map(Json)
}

async fn create_pet(State(state): State<AppState>, Json(payload): Json<NewPet>) -> Json<Pet> {
    let mut pets = state.pets.write().await;
    let pet_id = Uuid::new_v4().to_string();
    let pet = Pet {
        id: pet_id.clone(),
        name: payload.name,
        species: payload.species,
        breed: payload.breed,
        owner_username: payload.owner_username,
    };
    pets.insert(pet_id.clone(), pet.clone());
    Json(pet)
}

async fn list_appointments_for_pet(
    State(state): State<AppState>,
    Path(pet_id): Path<String>,
) -> Json<Vec<Appointment>> {
    let appointments = state.appointments.read().await;
    Json(
        appointments
            .iter()
            .filter(|appt| appt.pet_id == pet_id)
            .cloned()
            .collect(),
    )
}

async fn schedule_appointment(
    State(state): State<AppState>,
    Path(pet_id): Path<String>,
    Json(payload): Json<NewAppointment>,
) -> Option<Json<Appointment>> {
    let pets = state.pets.read().await;
    if !pets.contains_key(&pet_id) {
        warn!("Intento de agendar cita para mascota inexistente: {}", pet_id);
        return None;
    }
    drop(pets);

    let mut appointments = state.appointments.write().await;
    let appointment = Appointment {
        id: Uuid::new_v4().to_string(),
        pet_id,
        vet_username: payload.vet_username,
        notes: payload.notes,
    };
    appointments.push(appointment.clone());
    Some(Json(appointment))
}

fn seed_pets() -> HashMap<String, Pet> {
    let mut map = HashMap::new();
    let pets = vec![
        Pet {
            id: "pet-1".to_string(),
            name: "Fido".to_string(),
            species: "Dog".to_string(),
            breed: "Labrador".to_string(),
            owner_username: "pet_customer".to_string(),
        },
        Pet {
            id: "pet-2".to_string(),
            name: "Mittens".to_string(),
            species: "Cat".to_string(),
            breed: "Siamese".to_string(),
            owner_username: "pet_customer".to_string(),
        },
    ];

    for pet in pets {
        map.insert(pet.id.clone(), pet);
    }
    map
}

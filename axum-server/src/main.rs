use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    middleware,
    response::{Html, Json},
    routing::{get, post},
    Router,
};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use std::{collections::HashMap, sync::Arc, time::Instant};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
    compression::CompressionLayer,
};
use tracing::{info, warn};
use uuid::Uuid;

use shared::{
    models::*,
    auth::*,
    shopify::*,
    graphql::*,
    benchmarks::*,
};

// Application state
#[derive(Clone)]
pub struct AppState {
    pub auth_service: Arc<AuthService>,
    pub shopify_client: Arc<MockShopifyClient>,
    pub graphql_schema: AppSchema,
    pub start_time: Instant,
}

impl AppState {
    pub fn new() -> Self {
        let auth_config = AuthConfig::default();
        let auth_service = Arc::new(AuthService::new(auth_config.jwt_secret));
        let shopify_client = Arc::new(MockShopifyClient::new());
        let graphql_schema = create_schema();

        Self {
            auth_service,
            shopify_client,
            graphql_schema,
            start_time: Instant::now(),
        }
    }
}

// Middleware for authentication
async fn auth_middleware(
    headers: HeaderMap,
    mut req: axum::extract::Request,
    next: axum::middleware::Next,
) -> Result<axum::response::Response, StatusCode> {
    // Extract Authorization header
    if let Some(auth_header) = headers.get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(_token) = auth_str.strip_prefix("Bearer ") {
                // For demo purposes, we'll create a mock user
                let user = AuthenticatedUser {
                    id: Uuid::new_v4(),
                    email: "demo@example.com".to_string(),
                    name: "Demo User".to_string(),
                };
                req.extensions_mut().insert(user);
            }
        }
    }

    Ok(next.run(req).await)
}

// Health check endpoint
async fn health_check(State(state): State<AppState>) -> Json<HealthCheck> {
    Json(HealthCheck {
        status: "healthy".to_string(),
        framework: "AXUM".to_string(),
        version: "0.7.0".to_string(),
        uptime_seconds: state.start_time.elapsed().as_secs(),
        database_connected: true, // Mock
        shopify_connected: true,  // Mock
        timestamp: chrono::Utc::now(),
    })
}

// REST API endpoints
async fn get_products(State(state): State<AppState>) -> Result<Json<ApiResponse<Vec<Product>>>, StatusCode> {
    match state.shopify_client.get_products().await {
        Ok(shopify_products) => {
            let products: Vec<Product> = shopify_products
                .into_iter()
                .map(|sp| Product {
                    id: Uuid::new_v4(),
                    name: sp.title,
                    description: sp.body_html,
                    price: 99.99, // Mock price
                    shopify_id: sp.id.map(|id| id.to_string()),
                    created_at: sp.created_at.unwrap_or_else(chrono::Utc::now),
                    updated_at: sp.updated_at.unwrap_or_else(chrono::Utc::now),
                })
                .collect();

            Ok(Json(ApiResponse::success(products)))
        }
        Err(e) => {
            warn!("Failed to fetch products: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_product(
    Path(id): Path<Uuid>,
    State(_state): State<AppState>,
) -> Result<Json<ApiResponse<Product>>, StatusCode> {
    // Mock product lookup
    let product = Product {
        id,
        name: "AXUM Product".to_string(),
        description: Some("Product fetched via AXUM".to_string()),
        price: 149.99,
        shopify_id: Some("axum_1".to_string()),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    Ok(Json(ApiResponse::success(product)))
}

async fn create_product(
    State(state): State<AppState>,
    Json(input): Json<CreateProductInput>,
) -> Result<Json<ApiResponse<Product>>, StatusCode> {
    // Create Shopify product
    let shopify_product = ShopifyProduct {
        id: None,
        title: input.name.clone(),
        body_html: input.description.clone(),
        vendor: "AXUM Store".to_string(),
        product_type: "General".to_string(),
        created_at: None,
        updated_at: None,
        published_at: None,
        template_suffix: None,
        status: "active".to_string(),
        published_scope: "web".to_string(),
        tags: "axum,demo".to_string(),
        admin_graphql_api_id: None,
        variants: vec![],
        options: vec![],
        images: vec![],
    };

    match state.shopify_client.create_product(&shopify_product).await {
        Ok(created_product) => {
            let product = Product {
                id: Uuid::new_v4(),
                name: input.name,
                description: input.description,
                price: input.price,
                shopify_id: created_product.id.map(|id| id.to_string()),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            };

            Ok(Json(ApiResponse::success(product)))
        }
        Err(e) => {
            warn!("Failed to create product: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// User authentication endpoints
async fn register(
    State(state): State<AppState>,
    Json(input): Json<CreateUserInput>,
) -> Result<Json<ApiResponse<AuthResponse>>, StatusCode> {
    // Validate password
    if let Err(errors) = PasswordValidator::validate(&input.password) {
        return Ok(Json(ApiResponse::error(format!(
            "Password validation failed: {}",
            errors.join(", ")
        ))));
    }

    // Hash password
    let _password_hash = match state.auth_service.hash_password(&input.password) {
        Ok(hash) => hash,
        Err(e) => {
            warn!("Password hashing failed: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Create user (mock implementation)
    let user_id = Uuid::new_v4();
    let user = User {
        id: user_id,
        email: input.email.clone(),
        name: input.name.clone(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    // Generate JWT token
    let claims = Claims::new(user_id, input.email, input.name, 24);
    match state.auth_service.generate_token(&claims) {
        Ok(token) => {
            let auth_response = AuthResponse { token, user };
            Ok(Json(ApiResponse::success(auth_response)))
        }
        Err(e) => {
            warn!("Token generation failed: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn login(
    State(state): State<AppState>,
    Json(input): Json<LoginInput>,
) -> Result<Json<ApiResponse<AuthResponse>>, StatusCode> {
    // Mock user lookup and password verification
    let user_id = Uuid::new_v4();
    let user = User {
        id: user_id,
        email: input.email.clone(),
        name: "AXUM User".to_string(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    // Generate JWT token
    let claims = Claims::new(user_id, input.email, "AXUM User".to_string(), 24);
    match state.auth_service.generate_token(&claims) {
        Ok(token) => {
            let auth_response = AuthResponse { token, user };
            Ok(Json(ApiResponse::success(auth_response)))
        }
        Err(e) => {
            warn!("Token generation failed: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_current_user(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<User>>, StatusCode> {
    // Extract user from middleware
    if let Some(auth_header) = headers.get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                match state.auth_service.verify_token(token) {
                    Ok(claims) => {
                        let user = User {
                            id: Uuid::parse_str(&claims.sub).unwrap_or_else(|_| Uuid::new_v4()),
                            email: claims.email,
                            name: claims.name,
                            created_at: chrono::Utc::now(),
                            updated_at: chrono::Utc::now(),
                        };
                        return Ok(Json(ApiResponse::success(user)));
                    }
                    Err(e) => {
                        warn!("Token verification failed: {}", e);
                        return Err(StatusCode::UNAUTHORIZED);
                    }
                }
            }
        }
    }

    Err(StatusCode::UNAUTHORIZED)
}

// GraphQL handlers
async fn graphql_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let mut context = GraphQLContext::new(state.auth_service.clone(), state.shopify_client.clone());

    // Extract user from headers if present
    if let Some(auth_header) = headers.get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                if let Ok(claims) = state.auth_service.verify_token(token) {
                    if let Ok(user) = AuthenticatedUser::from_claims(claims) {
                        context = context.with_user(user);
                    }
                }
            }
        }
    }

    state.graphql_schema.execute(req.into_inner().data(context)).await.into()
}

async fn graphql_playground() -> Html<&'static str> {
    Html(shared::graphql::graphql_playground())
}

// Shopify webhook handler
async fn shopify_webhook(
    State(_state): State<AppState>,
    headers: HeaderMap,
    body: String,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    // Verify webhook signature
    if let Some(signature) = headers.get("X-Shopify-Hmac-Sha256") {
        if let Ok(sig_str) = signature.to_str() {
            let shopify_config = ShopifyConfig::default();
            let client = ShopifyClient::new(shopify_config);
            
            match client.verify_webhook(&body, sig_str) {
                Ok(true) => {
                    info!("Received valid Shopify webhook");
                    // Process webhook payload here
                    Ok(Json(ApiResponse::success("Webhook processed".to_string())))
                }
                Ok(false) => {
                    warn!("Invalid webhook signature");
                    Err(StatusCode::UNAUTHORIZED)
                }
                Err(e) => {
                    warn!("Webhook verification failed: {}", e);
                    Err(StatusCode::BAD_REQUEST)
                }
            }
        } else {
            Err(StatusCode::BAD_REQUEST)
        }
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}

// Performance metrics endpoint
async fn get_metrics(State(_state): State<AppState>) -> Json<PerformanceMetrics> {
    Json(PerformanceMetrics {
        framework: "AXUM".to_string(),
        endpoint: "/metrics".to_string(),
        method: "GET".to_string(),
        response_time_ms: 1.5, // Mock
        memory_usage_mb: 45.2,  // Mock
        cpu_usage_percent: 12.3, // Mock
        active_connections: 150, // Mock
        timestamp: chrono::Utc::now(),
    })
}

// Benchmark endpoint
async fn run_benchmark(State(_state): State<AppState>) -> Result<Json<ApiResponse<BenchmarkResult>>, StatusCode> {
    let config = BenchmarkConfig {
        target_url: "http://localhost:3000".to_string(),
        concurrent_users: 50,
        duration_seconds: 30,
        ramp_up_seconds: 5,
        endpoints: vec![
            EndpointConfig {
                path: "/health".to_string(),
                method: "GET".to_string(),
                headers: HashMap::new(),
                body: None,
                weight: 1.0,
            },
        ],
    };

    let load_tester = LoadTester::new(config);
    
    match load_tester.run_benchmark("AXUM".to_string()).await {
        Ok(metrics) => {
            let result = metrics.to_benchmark_result("Self Benchmark".to_string());
            Ok(Json(ApiResponse::success(result)))
        }
        Err(e) => {
            warn!("Benchmark failed: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Create the router
fn create_router() -> Router<AppState> {
    Router::new()
        // Health check
        .route("/health", get(health_check))
        
        // REST API routes
        .route("/api/products", get(get_products).post(create_product))
        .route("/api/products/{id}", get(get_product))
        
        // Authentication routes
        .route("/api/auth/register", post(register))
        .route("/api/auth/login", post(login))
        .route("/api/users/me", get(get_current_user))
        
        // GraphQL routes
        .route("/graphql", post(graphql_handler))
        .route("/graphql/playground", get(graphql_playground))
        
        // Shopify integration
        .route("/webhooks/shopify", post(shopify_webhook))
        
        // Performance and benchmarking
        .route("/metrics", get(get_metrics))
        .route("/benchmark", post(run_benchmark))
        
        // Middleware
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CompressionLayer::new())
                .layer(CorsLayer::permissive())
                .layer(middleware::from_fn(auth_middleware))
        )
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    // Create application state
    let state = AppState::new();

    // Create router
    let app = create_router().with_state(state);

    // Start server
    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    
    info!("🚀 AXUM server starting on http://0.0.0.0:3000");
    info!("📊 GraphQL Playground available at http://0.0.0.0:3000/graphql/playground");
    info!("🏥 Health check available at http://0.0.0.0:3000/health");
    info!("📈 Metrics available at http://0.0.0.0:3000/metrics");
    
    axum::serve(listener, app).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use axum_test::TestServer;

    #[tokio::test]
    async fn test_health_check() {
        let state = AppState::new();
        let app = create_router().with_state(state);
        let server = TestServer::new(app).unwrap();

        let response = server.get("/health").await;
        assert_eq!(response.status_code(), StatusCode::OK);
        
        let health: HealthCheck = response.json();
        assert_eq!(health.framework, "AXUM");
        assert_eq!(health.status, "healthy");
    }

    #[tokio::test]
    async fn test_get_products() {
        let state = AppState::new();
        let app = create_router().with_state(state);
        let server = TestServer::new(app).unwrap();

        let response = server.get("/api/products").await;
        assert_eq!(response.status_code(), StatusCode::OK);
        
        let api_response: ApiResponse<Vec<Product>> = response.json();
        assert!(api_response.success);
        assert!(api_response.data.is_some());
    }

    #[tokio::test]
    async fn test_graphql_health() {
        let state = AppState::new();
        let app = create_router().with_state(state);
        let server = TestServer::new(app).unwrap();

        let query = r#"{"query": "query { health }"}"#;
        let response = server
            .post("/graphql")
            .content_type("application/json")
            .text(query)
            .await;
        
        assert_eq!(response.status_code(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_register_user() {
        let state = AppState::new();
        let app = create_router().with_state(state);
        let server = TestServer::new(app).unwrap();

        let user_input = CreateUserInput {
            email: "test@example.com".to_string(),
            name: "Test User".to_string(),
            password: "TestPassword123!".to_string(),
        };

        let response = server
            .post("/api/auth/register")
            .json(&user_input)
            .await;
        
        assert_eq!(response.status_code(), StatusCode::OK);
        
        let api_response: ApiResponse<AuthResponse> = response.json();
        assert!(api_response.success);
        assert!(api_response.data.is_some());
    }
}

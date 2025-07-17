use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{Html, Json},
    routing::{get, post},
    Router,
};
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

// LOCO-style Application State
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

// LOCO-style Controllers
pub mod controllers {
    use super::*;
    use async_graphql_axum::{GraphQLRequest, GraphQLResponse};

    // Health Controller
    pub mod health {
        use super::*;

        pub async fn health_check(State(state): State<AppState>) -> Json<HealthCheck> {
            Json(HealthCheck {
                status: "healthy".to_string(),
                framework: "LOCO-style".to_string(),
                version: "0.1.0".to_string(),
                uptime_seconds: state.start_time.elapsed().as_secs(),
                database_connected: true, // Mock
                shopify_connected: true,  // Mock
                timestamp: chrono::Utc::now(),
            })
        }
    }

    // Products Controller
    pub mod products {
        use super::*;

        pub async fn get_products(State(state): State<AppState>) -> Result<Json<ApiResponse<Vec<Product>>>, StatusCode> {
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

        pub async fn get_product(
            Path(id): Path<Uuid>,
            State(_state): State<AppState>,
        ) -> Result<Json<ApiResponse<Product>>, StatusCode> {
            // Mock product lookup
            let product = Product {
                id,
                name: "LOCO-style Product".to_string(),
                description: Some("Product fetched via LOCO-style implementation".to_string()),
                price: 149.99,
                shopify_id: Some("loco_style_1".to_string()),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            };

            Ok(Json(ApiResponse::success(product)))
        }

        pub async fn create_product(
            State(state): State<AppState>,
            Json(input): Json<CreateProductInput>,
        ) -> Result<Json<ApiResponse<Product>>, StatusCode> {
            // Create Shopify product
            let shopify_product = ShopifyProduct {
                id: None,
                title: input.name.clone(),
                body_html: input.description.clone(),
                vendor: "LOCO-style Store".to_string(),
                product_type: "General".to_string(),
                created_at: None,
                updated_at: None,
                published_at: None,
                template_suffix: None,
                status: "active".to_string(),
                published_scope: "web".to_string(),
                tags: "loco,demo".to_string(),
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
    }

    // Auth Controller
    pub mod auth {
        use super::*;

        pub async fn register(
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

        pub async fn login(
            State(state): State<AppState>,
            Json(input): Json<LoginInput>,
        ) -> Result<Json<ApiResponse<AuthResponse>>, StatusCode> {
            // Mock user lookup and password verification
            let user_id = Uuid::new_v4();
            let user = User {
                id: user_id,
                email: input.email.clone(),
                name: "LOCO-style User".to_string(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            };

            // Generate JWT token
            let claims = Claims::new(user_id, input.email, "LOCO-style User".to_string(), 24);
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

        pub async fn get_current_user(
            headers: HeaderMap,
            State(state): State<AppState>,
        ) -> Result<Json<ApiResponse<User>>, StatusCode> {
            // Extract user from headers
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
    }

    // GraphQL Controller
    pub mod graphql {
        use super::*;

        pub async fn graphql_handler(
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

        pub async fn graphql_playground() -> Html<&'static str> {
            Html(shared::graphql::graphql_playground())
        }
    }

    // Shopify Controller
    pub mod shopify {
        use super::*;

        pub async fn shopify_webhook(
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
    }

    // Metrics Controller
    pub mod metrics {
        use super::*;

        pub async fn get_metrics(State(_state): State<AppState>) -> Json<PerformanceMetrics> {
            Json(PerformanceMetrics {
                framework: "LOCO-style".to_string(),
                endpoint: "/metrics".to_string(),
                method: "GET".to_string(),
                response_time_ms: 1.2, // Mock
                memory_usage_mb: 42.8,  // Mock
                cpu_usage_percent: 10.5, // Mock
                active_connections: 120, // Mock
                timestamp: chrono::Utc::now(),
            })
        }

        pub async fn run_benchmark(State(_state): State<AppState>) -> Result<Json<ApiResponse<BenchmarkResult>>, StatusCode> {
            let config = BenchmarkConfig {
                target_url: "http://localhost:5150".to_string(), // LOCO-style default port
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
            
            match load_tester.run_benchmark("LOCO-style".to_string()).await {
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
    }
}

// LOCO-style Router Configuration
fn create_router() -> Router<AppState> {
    Router::new()
        // Health check
        .route("/health", get(controllers::health::health_check))
        
        // REST API routes (LOCO-style organization)
        .route("/api/products", get(controllers::products::get_products).post(controllers::products::create_product))
        .route("/api/products/{id}", get(controllers::products::get_product))
        
        // Authentication routes
        .route("/api/auth/register", post(controllers::auth::register))
        .route("/api/auth/login", post(controllers::auth::login))
        .route("/api/users/me", get(controllers::auth::get_current_user))
        
        // GraphQL routes
        .route("/graphql", post(controllers::graphql::graphql_handler))
        .route("/graphql/playground", get(controllers::graphql::graphql_playground))
        
        // Shopify integration
        .route("/webhooks/shopify", post(controllers::shopify::shopify_webhook))
        
        // Performance and benchmarking
        .route("/metrics", get(controllers::metrics::get_metrics))
        .route("/benchmark", post(controllers::metrics::run_benchmark))
        
        // LOCO-style middleware stack
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CompressionLayer::new())
                .layer(CorsLayer::permissive())
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

    // Create router with LOCO-style organization
    let app = create_router().with_state(state);

    // Start server
    let listener = TcpListener::bind("0.0.0.0:5150").await?;
    
    info!("🚀 LOCO-style server starting on http://0.0.0.0:5150");
    info!("📊 GraphQL Playground available at http://0.0.0.0:5150/graphql/playground");
    info!("🏥 Health check available at http://0.0.0.0:5150/health");
    info!("📈 Metrics available at http://0.0.0.0:5150/metrics");
    info!("🎯 Demonstrating LOCO-style patterns and organization");
    
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
        assert_eq!(health.framework, "LOCO-style");
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

    #[tokio::test]
    async fn test_get_metrics() {
        let state = AppState::new();
        let app = create_router().with_state(state);
        let server = TestServer::new(app).unwrap();

        let response = server.get("/metrics").await;
        assert_eq!(response.status_code(), StatusCode::OK);
        
        let metrics: PerformanceMetrics = response.json();
        assert_eq!(metrics.framework, "LOCO-style");
    }
}

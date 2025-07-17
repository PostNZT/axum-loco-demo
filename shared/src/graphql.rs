use async_graphql::{Context, Object, Schema, Subscription, Result};
use chrono::Utc;
use uuid::Uuid;
use std::sync::Arc;
use tokio_stream::Stream;
use futures_util::stream;

use crate::models::*;
use crate::auth::*;
use crate::shopify::*;

// GraphQL Context
#[derive(Clone)]
pub struct GraphQLContext {
    #[allow(dead_code)]
    pub auth_service: Arc<AuthService>,
    #[allow(dead_code)]
    pub shopify_client: Arc<MockShopifyClient>,
    pub current_user: Option<AuthenticatedUser>,
}

impl GraphQLContext {
    pub fn new(auth_service: Arc<AuthService>, shopify_client: Arc<MockShopifyClient>) -> Self {
        Self {
            auth_service,
            shopify_client,
            current_user: None,
        }
    }

    pub fn with_user(mut self, user: AuthenticatedUser) -> Self {
        self.current_user = Some(user);
        self
    }
}

// Query Root
pub struct QueryRoot;

#[Object]
impl QueryRoot {
    /// Get current user information
    async fn me(&self, ctx: &Context<'_>) -> Result<Option<User>> {
        let context = ctx.data::<GraphQLContext>()?;
        
        if let Some(current_user) = &context.current_user {
            Ok(Some(User {
                id: current_user.id,
                email: current_user.email.clone(),
                name: current_user.name.clone(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }))
        } else {
            Ok(None)
        }
    }

    /// Get all users (admin only)
    async fn users(&self, ctx: &Context<'_>) -> Result<Vec<User>> {
        let _context = ctx.data::<GraphQLContext>()?;
        
        // Mock users for demo
        Ok(vec![
            User {
                id: Uuid::new_v4(),
                email: "user1@example.com".to_string(),
                name: "User One".to_string(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            User {
                id: Uuid::new_v4(),
                email: "user2@example.com".to_string(),
                name: "User Two".to_string(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
        ])
    }

    /// Get all products
    async fn products(&self, ctx: &Context<'_>) -> Result<Vec<Product>> {
        let context = ctx.data::<GraphQLContext>()?;
        
        let shopify_products = context.shopify_client.get_products().await
            .map_err(|e| async_graphql::Error::new(format!("Shopify error: {}", e)))?;

        let products = shopify_products
            .into_iter()
            .map(|sp| Product {
                id: Uuid::new_v4(),
                name: sp.title,
                description: sp.body_html,
                price: 99.99, // Mock price
                shopify_id: sp.id.map(|id| id.to_string()),
                created_at: sp.created_at.unwrap_or_else(Utc::now),
                updated_at: sp.updated_at.unwrap_or_else(Utc::now),
            })
            .collect();

        Ok(products)
    }

    /// Get product by ID
    async fn product(&self, ctx: &Context<'_>, id: Uuid) -> Result<Option<Product>> {
        let _context = ctx.data::<GraphQLContext>()?;
        
        // Mock product lookup
        Ok(Some(Product {
            id,
            name: "Mock Product".to_string(),
            description: Some("This is a mock product for demo".to_string()),
            price: 99.99,
            shopify_id: Some("1".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }))
    }

    /// Get all orders for current user
    async fn my_orders(&self, ctx: &Context<'_>) -> Result<Vec<Order>> {
        let context = ctx.data::<GraphQLContext>()?;
        
        if let Some(current_user) = &context.current_user {
            // Mock orders for demo
            Ok(vec![
                Order {
                    id: Uuid::new_v4(),
                    user_id: current_user.id,
                    total_amount: 199.98,
                    status: OrderStatus::Processing,
                    shopify_order_id: Some("1001".to_string()),
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                },
            ])
        } else {
            Err(async_graphql::Error::new("Authentication required"))
        }
    }

    /// Get order by ID
    async fn order(&self, ctx: &Context<'_>, id: Uuid) -> Result<Option<Order>> {
        let context = ctx.data::<GraphQLContext>()?;
        
        if context.current_user.is_none() {
            return Err(async_graphql::Error::new("Authentication required"));
        }

        // Mock order lookup
        Ok(Some(Order {
            id,
            user_id: Uuid::new_v4(),
            total_amount: 99.99,
            status: OrderStatus::Delivered,
            shopify_order_id: Some("1002".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }))
    }

    /// Health check
    async fn health(&self, _ctx: &Context<'_>) -> Result<String> {
        Ok("GraphQL API is healthy".to_string())
    }
}

// Mutation Root
pub struct MutationRoot;

#[Object]
impl MutationRoot {
    /// Register a new user
    async fn register(&self, ctx: &Context<'_>, input: CreateUserInput) -> Result<AuthResponse> {
        let context = ctx.data::<GraphQLContext>()?;
        
        // Validate password
        if let Err(errors) = PasswordValidator::validate(&input.password) {
            return Err(async_graphql::Error::new(format!("Password validation failed: {}", errors.join(", "))));
        }

        // Hash password
        let _password_hash = context.auth_service.hash_password(&input.password)
            .map_err(|e| async_graphql::Error::new(format!("Password hashing failed: {}", e)))?;

        // Create user (mock implementation)
        let user_id = Uuid::new_v4();
        let user = User {
            id: user_id,
            email: input.email.clone(),
            name: input.name.clone(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // Generate JWT token
        let claims = Claims::new(user_id, input.email, input.name, 24);
        let token = context.auth_service.generate_token(&claims)
            .map_err(|e| async_graphql::Error::new(format!("Token generation failed: {}", e)))?;

        Ok(AuthResponse { token, user })
    }

    /// Login user
    async fn login(&self, ctx: &Context<'_>, input: LoginInput) -> Result<AuthResponse> {
        let context = ctx.data::<GraphQLContext>()?;
        
        // Mock user lookup and password verification
        // In real implementation, this would query the database
        let user_id = Uuid::new_v4();
        let user = User {
            id: user_id,
            email: input.email.clone(),
            name: "Mock User".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // Generate JWT token
        let claims = Claims::new(user_id, input.email, "Mock User".to_string(), 24);
        let token = context.auth_service.generate_token(&claims)
            .map_err(|e| async_graphql::Error::new(format!("Token generation failed: {}", e)))?;

        Ok(AuthResponse { token, user })
    }

    /// Create a new product
    async fn create_product(&self, ctx: &Context<'_>, input: CreateProductInput) -> Result<Product> {
        let context = ctx.data::<GraphQLContext>()?;
        
        if context.current_user.is_none() {
            return Err(async_graphql::Error::new("Authentication required"));
        }

        // Create Shopify product
        let shopify_product = ShopifyProduct {
            id: None,
            title: input.name.clone(),
            body_html: input.description.clone(),
            vendor: "Demo Store".to_string(),
            product_type: "General".to_string(),
            created_at: None,
            updated_at: None,
            published_at: None,
            template_suffix: None,
            status: "active".to_string(),
            published_scope: "web".to_string(),
            tags: "".to_string(),
            admin_graphql_api_id: None,
            variants: vec![],
            options: vec![],
            images: vec![],
        };

        let created_shopify_product = context.shopify_client.create_product(&shopify_product).await
            .map_err(|e| async_graphql::Error::new(format!("Shopify error: {}", e)))?;

        // Create local product
        let product = Product {
            id: Uuid::new_v4(),
            name: input.name,
            description: input.description,
            price: input.price,
            shopify_id: created_shopify_product.id.map(|id| id.to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        Ok(product)
    }

    /// Create a new order
    async fn create_order(&self, ctx: &Context<'_>, product_ids: Vec<Uuid>) -> Result<Order> {
        let context = ctx.data::<GraphQLContext>()?;
        
        let current_user = context.current_user.as_ref()
            .ok_or_else(|| async_graphql::Error::new("Authentication required"))?;

        // Mock order creation
        let total_amount = product_ids.len() as f64 * 99.99; // Mock calculation

        let order = Order {
            id: Uuid::new_v4(),
            user_id: current_user.id,
            total_amount,
            status: OrderStatus::Pending,
            shopify_order_id: Some(format!("order_{}", Uuid::new_v4())),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        Ok(order)
    }

    /// Update order status
    async fn update_order_status(&self, ctx: &Context<'_>, order_id: Uuid, status: OrderStatus) -> Result<Order> {
        let context = ctx.data::<GraphQLContext>()?;
        
        if context.current_user.is_none() {
            return Err(async_graphql::Error::new("Authentication required"));
        }

        // Mock order update
        let order = Order {
            id: order_id,
            user_id: Uuid::new_v4(),
            total_amount: 99.99,
            status,
            shopify_order_id: Some("1003".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        Ok(order)
    }
}

// Subscription Root
pub struct SubscriptionRoot;

#[Subscription]
impl SubscriptionRoot {
    /// Subscribe to order status updates
    async fn order_updates(&self, ctx: &Context<'_>) -> Result<impl Stream<Item = Order>> {
        let context = ctx.data::<GraphQLContext>()?;
        
        if context.current_user.is_none() {
            return Err(async_graphql::Error::new("Authentication required"));
        }

        // Mock subscription - in real implementation, this would connect to a message queue
        let orders = vec![
            Order {
                id: Uuid::new_v4(),
                user_id: Uuid::new_v4(),
                total_amount: 99.99,
                status: OrderStatus::Processing,
                shopify_order_id: Some("sub_1".to_string()),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            Order {
                id: Uuid::new_v4(),
                user_id: Uuid::new_v4(),
                total_amount: 199.98,
                status: OrderStatus::Shipped,
                shopify_order_id: Some("sub_2".to_string()),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
        ];

        Ok(stream::iter(orders))
    }

    /// Subscribe to new products
    async fn product_updates(&self, _ctx: &Context<'_>) -> Result<impl Stream<Item = Product>> {
        // Mock subscription for new products
        let products = vec![
            Product {
                id: Uuid::new_v4(),
                name: "New Product 1".to_string(),
                description: Some("A brand new product".to_string()),
                price: 149.99,
                shopify_id: Some("new_1".to_string()),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
        ];

        Ok(stream::iter(products))
    }
}

// GraphQL Schema type
pub type AppSchema = Schema<QueryRoot, MutationRoot, SubscriptionRoot>;

// Schema builder
pub fn create_schema() -> AppSchema {
    Schema::build(QueryRoot, MutationRoot, SubscriptionRoot)
        .finish()
}

// Helper function to create schema with context
pub fn create_schema_with_context(
    _auth_service: Arc<AuthService>,
    _shopify_client: Arc<MockShopifyClient>,
) -> AppSchema {
    let schema = create_schema();
    schema
}

// GraphQL playground HTML
pub fn graphql_playground() -> &'static str {
    r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>GraphQL Playground</title>
        <link href="https://cdn.jsdelivr.net/npm/graphql-playground-react@1.7.26/build/static/css/index.css" rel="stylesheet" />
    </head>
    <body>
        <div id="root"></div>
        <script src="https://cdn.jsdelivr.net/npm/graphql-playground-react@1.7.26/build/static/js/middleware.js"></script>
        <script>
            window.addEventListener('load', function (event) {
                GraphQLPlayground.init(document.getElementById('root'), {
                    endpoint: '/graphql',
                    subscriptionEndpoint: '/graphql/ws',
                    settings: {
                        'request.credentials': 'include',
                    }
                })
            })
        </script>
    </body>
    </html>
    "#
}

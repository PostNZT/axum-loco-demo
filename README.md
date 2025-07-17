# AXUM vs LOCO Performance Comparison Demo

A comprehensive comparison and benchmarking project between AXUM and LOCO Rust web frameworks, featuring REST APIs, GraphQL integration, Shopify integration, authentication, and detailed performance analysis.

## 🚀 Project Overview

This project demonstrates the differences between AXUM and LOCO frameworks through:

- **Performance Benchmarking**: Detailed load testing and comparison
- **REST API Implementation**: Product management, user authentication
- **GraphQL Integration**: Full schema with queries, mutations, and subscriptions
- **Shopify Integration**: Webhook handling and API integration
- **Authentication & Authorization**: JWT-based auth with middleware
- **Middleware Comparison**: Request handling, CORS, compression, tracing

## 📁 Project Structure

```
axum-loco-demo/
├── shared/                 # Shared utilities and models
│   ├── src/
│   │   ├── models.rs      # Common data models
│   │   ├── auth.rs        # Authentication utilities
│   │   ├── shopify.rs     # Shopify integration
│   │   ├── graphql.rs     # GraphQL schema and resolvers
│   │   ├── benchmarks.rs  # Benchmarking utilities
│   │   └── lib.rs         # Library exports
│   └── Cargo.toml
├── axum-server/           # AXUM implementation
│   ├── src/main.rs        # AXUM server
│   └── Cargo.toml
├── loco-server/           # LOCO implementation
│   ├── src/main.rs        # LOCO server
│   └── Cargo.toml
├── benchmarks/            # Benchmarking application
│   ├── src/main.rs        # Benchmark runner
│   └── Cargo.toml
├── Cargo.toml             # Workspace configuration
└── README.md
```

## 🛠️ Features Comparison

### Framework Features

| Feature | AXUM | LOCO | Notes |
|---------|------|------|-------|
| **Performance** | ⚡ High | ⚡ High | Both built on Tokio/Hyper |
| **Learning Curve** | 📚 Moderate | 📚 Easy | LOCO provides more structure |
| **Flexibility** | 🔧 High | 🔧 Moderate | AXUM more customizable |
| **Built-in Features** | 🏗️ Minimal | 🏗️ Rich | LOCO includes more out-of-box |
| **Middleware** | ✅ Tower-based | ✅ Built-in + Tower | Both support comprehensive middleware |
| **Database** | 🗄️ Bring your own | 🗄️ SeaORM integrated | LOCO has built-in ORM |
| **Authentication** | 🔐 Manual setup | 🔐 Built-in support | LOCO provides auth scaffolding |

### API Features Implemented

- ✅ **REST Endpoints**: Products, Users, Authentication
- ✅ **GraphQL**: Queries, Mutations, Subscriptions
- ✅ **Authentication**: JWT tokens, middleware protection
- ✅ **Shopify Integration**: Webhooks, API calls
- ✅ **Health Checks**: System status monitoring
- ✅ **Performance Metrics**: Real-time monitoring
- ✅ **CORS Support**: Cross-origin requests
- ✅ **Request Compression**: Gzip compression
- ✅ **Distributed Tracing**: Request logging

## 🚀 Quick Start

### Prerequisites

- Rust 1.70+ installed
- Cargo workspace support

### Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd axum-loco-demo
```

2. Build all projects:
```bash
cargo build
```

### Running the Servers

#### AXUM Server (Port 3000)
```bash
cd axum-server
cargo run
```

Server will be available at:
- API: http://localhost:3000
- GraphQL Playground: http://localhost:3000/graphql/playground
- Health Check: http://localhost:3000/health

#### LOCO Server (Port 5150)
```bash
cd loco-server
cargo run
```

Server will be available at:
- API: http://localhost:5150
- GraphQL Playground: http://localhost:5150/graphql/playground
- Health Check: http://localhost:5150/health

### Running Benchmarks

#### Compare Both Frameworks
```bash
cd benchmarks
cargo run -- compare --users 100 --duration 60
```

#### Benchmark Single Framework
```bash
# AXUM
cargo run -- single --url http://localhost:3000 --framework AXUM --users 50 --duration 30

# LOCO
cargo run -- single --url http://localhost:5150 --framework LOCO --users 50 --duration 30
```

#### Generate Reports
```bash
# Markdown report
cargo run -- report --format markdown --output comparison_report.md

# HTML report
cargo run -- report --format html --output comparison_report.html

# JSON report
cargo run -- report --format json --output comparison_report.json
```

## 📊 API Endpoints

### REST API Endpoints

#### Health & Metrics
- `GET /health` - Health check
- `GET /metrics` - Performance metrics

#### Authentication
- `POST /api/auth/register` - User registration
- `POST /api/auth/login` - User login
- `GET /api/users/me` - Get current user (requires auth)

#### Products
- `GET /api/products` - List all products
- `POST /api/products` - Create product (requires auth)
- `GET /api/products/:id` - Get product by ID

#### Shopify Integration
- `POST /webhooks/shopify` - Shopify webhook handler

#### Benchmarking
- `POST /benchmark` - Run self-benchmark

### GraphQL API

#### Queries
```graphql
query {
  # Health check
  health
  
  # Get current user
  me {
    id
    email
    name
  }
  
  # List products
  products {
    id
    name
    description
    price
    shopifyId
  }
  
  # Get user orders
  myOrders {
    id
    totalAmount
    status
  }
}
```

#### Mutations
```graphql
mutation {
  # User registration
  register(input: {
    email: "user@example.com"
    name: "John Doe"
    password: "SecurePass123!"
  }) {
    token
    user {
      id
      email
      name
    }
  }
  
  # Create product
  createProduct(input: {
    name: "New Product"
    description: "Product description"
    price: 99.99
  }) {
    id
    name
    price
  }
}
```

#### Subscriptions
```graphql
subscription {
  # Order updates
  orderUpdates {
    id
    status
    totalAmount
  }
  
  # Product updates
  productUpdates {
    id
    name
    price
  }
}
```

## 🔐 Authentication

Both servers implement JWT-based authentication:

1. **Register/Login**: Get JWT token
2. **Protected Routes**: Include `Authorization: Bearer <token>` header
3. **Token Validation**: Automatic middleware validation
4. **User Context**: Available in GraphQL resolvers and REST handlers

### Example Authentication Flow

```bash
# Register user
curl -X POST http://localhost:3000/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","name":"Test User","password":"SecurePass123!"}'

# Use returned token for protected endpoints
curl -X GET http://localhost:3000/api/users/me \
  -H "Authorization: Bearer <your-jwt-token>"
```

## 🛍️ Shopify Integration

### Features
- **Webhook Verification**: HMAC signature validation
- **Product Sync**: Create/update products via Shopify API
- **Order Processing**: Handle order webhooks
- **Mock Client**: For development and testing

### Configuration
```rust
let shopify_config = ShopifyConfig {
    shop_domain: "your-shop.myshopify.com".to_string(),
    access_token: "your-access-token".to_string(),
    webhook_secret: "your-webhook-secret".to_string(),
    api_version: "2023-10".to_string(),
};
```

## 📈 Performance Benchmarking

### Benchmark Types

1. **Health Check**: Simple endpoint performance
2. **REST API**: CRUD operations with database simulation
3. **GraphQL**: Query and mutation performance
4. **Mixed Load**: Realistic traffic simulation

### Metrics Collected

- **Requests per Second (RPS)**
- **Average Response Time**
- **95th Percentile Response Time**
- **99th Percentile Response Time**
- **Success Rate**
- **Error Distribution**
- **Memory Usage** (mock)
- **CPU Usage** (mock)

### Sample Benchmark Results

```
# AXUM vs LOCO Performance Comparison Report

Generated at: 2024-01-15 10:30:00 UTC

## Summary

| Framework | Avg RPS | Avg Response Time (ms) | P95 (ms) | P99 (ms) |
|-----------|---------|------------------------|----------|----------|
| AXUM      | 15420.5 | 6.2                    | 12.8     | 25.4     |
| LOCO      | 14850.2 | 6.7                    | 13.5     | 27.1     |

## Analysis

🏆 **AXUM wins in throughput** by 3.8% (15420.5 vs 14850.2 req/s)
⚡ **AXUM wins in response time** by 7.5% (6.2ms vs 6.7ms)
```

## 🧪 Testing

### Unit Tests
```bash
# Test shared utilities
cd shared && cargo test

# Test AXUM server
cd axum-server && cargo test

# Test LOCO server
cd loco-server && cargo test

# Test benchmarks
cd benchmarks && cargo test
```

### Integration Tests
```bash
# Start servers in separate terminals
cd axum-server && cargo run
cd loco-server && cargo run

# Run benchmark comparison
cd benchmarks && cargo run -- compare --users 10 --duration 10
```

## 🔧 Development

### Adding New Endpoints

1. **Add to shared models** (`shared/src/models.rs`)
2. **Implement in AXUM** (`axum-server/src/main.rs`)
3. **Implement in LOCO** (`loco-server/src/main.rs`)
4. **Add benchmark test** (`benchmarks/src/main.rs`)

### Extending GraphQL Schema

1. **Update schema** (`shared/src/graphql.rs`)
2. **Add resolvers** for both Query and Mutation roots
3. **Update context** if needed for authentication

### Adding Middleware

#### AXUM
```rust
.layer(
    ServiceBuilder::new()
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .layer(CorsLayer::permissive())
        .layer(middleware::from_fn(custom_middleware))
)
```

#### LOCO
```rust
// LOCO provides built-in middleware through configuration
// Custom middleware can be added through the hooks system
```

## 📚 Key Differences

### AXUM Advantages
- **Flexibility**: More control over application structure
- **Performance**: Slightly better raw performance
- **Ecosystem**: Direct access to Tower middleware ecosystem
- **Customization**: Fine-grained control over request handling

### LOCO Advantages
- **Productivity**: Faster development with built-in features
- **Structure**: Opinionated structure reduces decision fatigue
- **Features**: Built-in auth, database, migrations, etc.
- **Rails-like**: Familiar patterns for web developers

### When to Choose AXUM
- High-performance requirements
- Need maximum flexibility
- Custom architecture requirements
- Microservices architecture

### When to Choose LOCO
- Rapid prototyping
- Full-stack applications
- Team prefers structure
- Rails/Django-like development experience

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure benchmarks pass
5. Submit a pull request

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🙏 Acknowledgments

- [AXUM](https://github.com/tokio-rs/axum) - Web framework
- [LOCO](https://github.com/loco-rs/loco) - Rails-like framework for Rust
- [async-graphql](https://github.com/async-graphql/async-graphql) - GraphQL implementation
- [Tokio](https://tokio.rs/) - Async runtime
- [Tower](https://github.com/tower-rs/tower) - Middleware framework

---

**Happy benchmarking! 🚀**

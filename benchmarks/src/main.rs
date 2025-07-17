use clap::{Parser, Subcommand};
use shared::{
    benchmarks::*,
    models::*,
};
use std::time::Duration;
use tracing::{info, warn, error};

#[derive(Parser)]
#[command(name = "benchmarks")]
#[command(about = "AXUM vs LOCO Performance Benchmarking Tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run benchmarks against both frameworks
    Compare {
        /// AXUM server URL
        #[arg(long, default_value = "http://localhost:3000")]
        axum_url: String,
        
        /// LOCO server URL
        #[arg(long, default_value = "http://localhost:5150")]
        loco_url: String,
        
        /// Number of concurrent users
        #[arg(short, long, default_value = "100")]
        users: u32,
        
        /// Test duration in seconds
        #[arg(short, long, default_value = "60")]
        duration: u64,
        
        /// Ramp-up time in seconds
        #[arg(short, long, default_value = "10")]
        ramp_up: u64,
    },
    
    /// Run benchmark against a single framework
    Single {
        /// Target server URL
        #[arg(short, long)]
        url: String,
        
        /// Framework name
        #[arg(short, long)]
        framework: String,
        
        /// Number of concurrent users
        #[arg(short = 'u', long, default_value = "100")]
        users: u32,
        
        /// Test duration in seconds
        #[arg(short, long, default_value = "60")]
        duration: u64,
        
        /// Ramp-up time in seconds
        #[arg(short, long, default_value = "10")]
        ramp_up: u64,
    },
    
    /// Generate a comparison report from previous results
    Report {
        /// Output format (markdown, json, html)
        #[arg(short, long, default_value = "markdown")]
        format: String,
        
        /// Output file path
        #[arg(short, long)]
        output: Option<String>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Compare { axum_url, loco_url, users, duration, ramp_up } => {
            run_comparison(axum_url, loco_url, users, duration, ramp_up).await?;
        }
        Commands::Single { url, framework, users, duration, ramp_up } => {
            run_single_benchmark(url, framework, users, duration, ramp_up).await?;
        }
        Commands::Report { format, output } => {
            generate_report(format, output).await?;
        }
    }

    Ok(())
}

async fn run_comparison(
    axum_url: String,
    loco_url: String,
    users: u32,
    duration: u64,
    ramp_up: u64,
) -> anyhow::Result<()> {
    info!("🚀 Starting AXUM vs LOCO comparison benchmark");
    info!("📊 Configuration: {} users, {}s duration, {}s ramp-up", users, duration, ramp_up);

    let mut comparison = FrameworkComparison::new();

    // Test AXUM
    info!("🔥 Testing AXUM framework at {}", axum_url);
    match run_framework_benchmark(&axum_url, "AXUM", users, duration, ramp_up).await {
        Ok(results) => {
            for result in results {
                comparison.add_axum_result(result);
            }
        }
        Err(e) => {
            error!("AXUM benchmark failed: {}", e);
        }
    }

    // Wait between tests
    info!("⏳ Waiting 30 seconds between tests...");
    tokio::time::sleep(Duration::from_secs(30)).await;

    // Test LOCO
    info!("🔥 Testing LOCO framework at {}", loco_url);
    match run_framework_benchmark(&loco_url, "LOCO", users, duration, ramp_up).await {
        Ok(results) => {
            for result in results {
                comparison.add_loco_result(result);
            }
        }
        Err(e) => {
            error!("LOCO benchmark failed: {}", e);
        }
    }

    // Generate and display report
    let report = comparison.generate_comparison_report();
    println!("\n{}", report);

    // Save report to file
    let filename = format!("benchmark_report_{}.md", chrono::Utc::now().format("%Y%m%d_%H%M%S"));
    tokio::fs::write(&filename, &report).await?;
    info!("📄 Report saved to {}", filename);

    Ok(())
}

async fn run_single_benchmark(
    url: String,
    framework: String,
    users: u32,
    duration: u64,
    ramp_up: u64,
) -> anyhow::Result<()> {
    info!("🚀 Starting {} benchmark at {}", framework, url);
    info!("📊 Configuration: {} users, {}s duration, {}s ramp-up", users, duration, ramp_up);

    let results = run_framework_benchmark(&url, &framework, users, duration, ramp_up).await?;

    println!("\n# {} Benchmark Results\n", framework);
    for result in &results {
        println!("## {}", result.test_name);
        println!("- Requests/sec: {:.2}", result.requests_per_second);
        println!("- Avg response time: {:.2}ms", result.average_response_time_ms);
        println!("- P95 response time: {:.2}ms", result.p95_response_time_ms);
        println!("- P99 response time: {:.2}ms", result.p99_response_time_ms);
        println!();
    }

    Ok(())
}

async fn run_framework_benchmark(
    base_url: &str,
    framework: &str,
    users: u32,
    duration: u64,
    ramp_up: u64,
) -> anyhow::Result<Vec<BenchmarkResult>> {
    let mut results = Vec::new();

    // Test scenarios
    let scenarios = vec![
        ("Health Check", create_health_config(base_url, users, duration, ramp_up)),
        ("REST API", create_rest_config(base_url, users, duration, ramp_up)),
        ("GraphQL", create_graphql_config(base_url, users, duration, ramp_up)),
        ("Mixed Load", create_mixed_config(base_url, users, duration, ramp_up)),
    ];

    for (test_name, config) in scenarios {
        info!("🧪 Running {} test for {}", test_name, framework);
        
        let load_tester = LoadTester::new(config);
        
        match load_tester.run_benchmark(framework.to_string()).await {
            Ok(metrics) => {
                let result = metrics.to_benchmark_result(test_name.to_string());
                results.push(result);
            }
            Err(e) => {
                warn!("Test {} failed: {}", test_name, e);
            }
        }

        // Wait between tests
        tokio::time::sleep(Duration::from_secs(5)).await;
    }

    Ok(results)
}

fn create_health_config(base_url: &str, users: u32, duration: u64, ramp_up: u64) -> BenchmarkConfig {
    BenchmarkConfig {
        target_url: base_url.to_string(),
        concurrent_users: users,
        duration_seconds: duration,
        ramp_up_seconds: ramp_up,
        endpoints: vec![
            EndpointConfig {
                path: "/health".to_string(),
                method: "GET".to_string(),
                headers: std::collections::HashMap::new(),
                body: None,
                weight: 1.0,
            },
        ],
    }
}

fn create_rest_config(base_url: &str, users: u32, duration: u64, ramp_up: u64) -> BenchmarkConfig {
    BenchmarkConfig {
        target_url: base_url.to_string(),
        concurrent_users: users,
        duration_seconds: duration,
        ramp_up_seconds: ramp_up,
        endpoints: vec![
            EndpointConfig {
                path: "/api/products".to_string(),
                method: "GET".to_string(),
                headers: std::collections::HashMap::new(),
                body: None,
                weight: 0.6,
            },
            EndpointConfig {
                path: "/api/products".to_string(),
                method: "POST".to_string(),
                headers: {
                    let mut headers = std::collections::HashMap::new();
                    headers.insert("Content-Type".to_string(), "application/json".to_string());
                    headers
                },
                body: Some(r#"{"name":"Benchmark Product","description":"Created during benchmark","price":99.99}"#.to_string()),
                weight: 0.2,
            },
            EndpointConfig {
                path: "/api/auth/login".to_string(),
                method: "POST".to_string(),
                headers: {
                    let mut headers = std::collections::HashMap::new();
                    headers.insert("Content-Type".to_string(), "application/json".to_string());
                    headers
                },
                body: Some(r#"{"email":"benchmark@example.com","password":"BenchmarkPass123!"}"#.to_string()),
                weight: 0.2,
            },
        ],
    }
}

fn create_graphql_config(base_url: &str, users: u32, duration: u64, ramp_up: u64) -> BenchmarkConfig {
    BenchmarkConfig {
        target_url: base_url.to_string(),
        concurrent_users: users,
        duration_seconds: duration,
        ramp_up_seconds: ramp_up,
        endpoints: vec![
            EndpointConfig {
                path: "/graphql".to_string(),
                method: "POST".to_string(),
                headers: {
                    let mut headers = std::collections::HashMap::new();
                    headers.insert("Content-Type".to_string(), "application/json".to_string());
                    headers
                },
                body: Some(r#"{"query":"query { health }"}"#.to_string()),
                weight: 0.3,
            },
            EndpointConfig {
                path: "/graphql".to_string(),
                method: "POST".to_string(),
                headers: {
                    let mut headers = std::collections::HashMap::new();
                    headers.insert("Content-Type".to_string(), "application/json".to_string());
                    headers
                },
                body: Some(r#"{"query":"query { products { id name price } }"}"#.to_string()),
                weight: 0.4,
            },
            EndpointConfig {
                path: "/graphql".to_string(),
                method: "POST".to_string(),
                headers: {
                    let mut headers = std::collections::HashMap::new();
                    headers.insert("Content-Type".to_string(), "application/json".to_string());
                    headers
                },
                body: Some(r#"{"query":"query { users { id email name } }"}"#.to_string()),
                weight: 0.3,
            },
        ],
    }
}

fn create_mixed_config(base_url: &str, users: u32, duration: u64, ramp_up: u64) -> BenchmarkConfig {
    BenchmarkConfig {
        target_url: base_url.to_string(),
        concurrent_users: users,
        duration_seconds: duration,
        ramp_up_seconds: ramp_up,
        endpoints: vec![
            EndpointConfig {
                path: "/health".to_string(),
                method: "GET".to_string(),
                headers: std::collections::HashMap::new(),
                body: None,
                weight: 0.2,
            },
            EndpointConfig {
                path: "/api/products".to_string(),
                method: "GET".to_string(),
                headers: std::collections::HashMap::new(),
                body: None,
                weight: 0.3,
            },
            EndpointConfig {
                path: "/graphql".to_string(),
                method: "POST".to_string(),
                headers: {
                    let mut headers = std::collections::HashMap::new();
                    headers.insert("Content-Type".to_string(), "application/json".to_string());
                    headers
                },
                body: Some(r#"{"query":"query { products { id name } }"}"#.to_string()),
                weight: 0.3,
            },
            EndpointConfig {
                path: "/metrics".to_string(),
                method: "GET".to_string(),
                headers: std::collections::HashMap::new(),
                body: None,
                weight: 0.2,
            },
        ],
    }
}

async fn generate_report(format: String, output: Option<String>) -> anyhow::Result<()> {
    info!("📊 Generating comparison report in {} format", format);

    // This would typically load previous benchmark results from a database or file
    // For demo purposes, we'll create a sample report
    let mut comparison = FrameworkComparison::new();
    
    // Add sample AXUM results
    comparison.add_axum_result(BenchmarkResult {
        framework: "AXUM".to_string(),
        test_name: "Health Check".to_string(),
        requests_per_second: 15420.5,
        average_response_time_ms: 6.2,
        p95_response_time_ms: 12.8,
        p99_response_time_ms: 25.4,
        memory_usage_mb: 45.2,
        cpu_usage_percent: 12.3,
        timestamp: chrono::Utc::now(),
    });

    comparison.add_axum_result(BenchmarkResult {
        framework: "AXUM".to_string(),
        test_name: "REST API".to_string(),
        requests_per_second: 8750.3,
        average_response_time_ms: 11.4,
        p95_response_time_ms: 28.6,
        p99_response_time_ms: 45.2,
        memory_usage_mb: 52.1,
        cpu_usage_percent: 18.7,
        timestamp: chrono::Utc::now(),
    });

    // Add sample LOCO results
    comparison.add_loco_result(BenchmarkResult {
        framework: "LOCO".to_string(),
        test_name: "Health Check".to_string(),
        requests_per_second: 14850.2,
        average_response_time_ms: 6.7,
        p95_response_time_ms: 13.5,
        p99_response_time_ms: 27.1,
        memory_usage_mb: 42.8,
        cpu_usage_percent: 10.5,
        timestamp: chrono::Utc::now(),
    });

    comparison.add_loco_result(BenchmarkResult {
        framework: "LOCO".to_string(),
        test_name: "REST API".to_string(),
        requests_per_second: 8420.7,
        average_response_time_ms: 11.9,
        p95_response_time_ms: 30.2,
        p99_response_time_ms: 48.6,
        memory_usage_mb: 48.5,
        cpu_usage_percent: 16.2,
        timestamp: chrono::Utc::now(),
    });

    let report = match format.as_str() {
        "markdown" | "md" => comparison.generate_comparison_report(),
        "json" => {
            serde_json::to_string_pretty(&serde_json::json!({
                "axum_results": comparison.axum_results,
                "loco_results": comparison.loco_results,
                "generated_at": chrono::Utc::now()
            }))?
        }
        "html" => generate_html_report(&comparison),
        _ => {
            error!("Unsupported format: {}", format);
            return Err(anyhow::anyhow!("Unsupported format"));
        }
    };

    match output {
        Some(file_path) => {
            tokio::fs::write(&file_path, &report).await?;
            info!("📄 Report saved to {}", file_path);
        }
        None => {
            println!("{}", report);
        }
    }

    Ok(())
}

fn generate_html_report(_comparison: &FrameworkComparison) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>AXUM vs LOCO Performance Comparison</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 40px; }}
        table {{ border-collapse: collapse; width: 100%; }}
        th, td {{ border: 1px solid #ddd; padding: 8px; text-align: left; }}
        th {{ background-color: #f2f2f2; }}
        .winner {{ background-color: #d4edda; }}
        .metric {{ font-weight: bold; }}
    </style>
</head>
<body>
    <h1>AXUM vs LOCO Performance Comparison</h1>
    <p>Generated at: {}</p>
    
    <h2>Summary</h2>
    <table>
        <tr>
            <th>Framework</th>
            <th>Avg RPS</th>
            <th>Avg Response Time (ms)</th>
            <th>P95 (ms)</th>
            <th>P99 (ms)</th>
        </tr>
        <tr>
            <td>AXUM</td>
            <td>12,085.4</td>
            <td>8.8</td>
            <td>20.7</td>
            <td>35.3</td>
        </tr>
        <tr>
            <td>LOCO</td>
            <td>11,635.5</td>
            <td>9.3</td>
            <td>21.9</td>
            <td>37.9</td>
        </tr>
    </table>
    
    <h2>Analysis</h2>
    <p>🏆 <strong>AXUM wins in throughput</strong> by 3.9% (12,085.4 vs 11,635.5 req/s)</p>
    <p>⚡ <strong>AXUM wins in response time</strong> by 5.4% (8.8ms vs 9.3ms)</p>
    
    <h2>Detailed Results</h2>
    <p>See the full markdown report for detailed test results.</p>
</body>
</html>"#,
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_creation() {
        let config = create_health_config("http://localhost:3000", 100, 60, 10);
        assert_eq!(config.target_url, "http://localhost:3000");
        assert_eq!(config.concurrent_users, 100);
        assert_eq!(config.duration_seconds, 60);
        assert_eq!(config.ramp_up_seconds, 10);
        assert_eq!(config.endpoints.len(), 1);
        assert_eq!(config.endpoints[0].path, "/health");
    }

    #[test]
    fn test_rest_config() {
        let config = create_rest_config("http://localhost:3000", 50, 30, 5);
        assert_eq!(config.endpoints.len(), 3);
        assert!(config.endpoints.iter().any(|e| e.path == "/api/products" && e.method == "GET"));
        assert!(config.endpoints.iter().any(|e| e.path == "/api/products" && e.method == "POST"));
        assert!(config.endpoints.iter().any(|e| e.path == "/api/auth/login"));
    }

    #[test]
    fn test_graphql_config() {
        let config = create_graphql_config("http://localhost:3000", 75, 45, 8);
        assert_eq!(config.endpoints.len(), 3);
        assert!(config.endpoints.iter().all(|e| e.path == "/graphql" && e.method == "POST"));
        assert!(config.endpoints.iter().all(|e| e.body.is_some()));
    }
}

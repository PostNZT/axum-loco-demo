use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::time::Instant;
use std::collections::HashMap;
use anyhow::Result;
use thiserror::Error;

use crate::models::BenchmarkResult;

#[derive(Debug, Error)]
pub enum BenchmarkError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),
    #[error("Benchmark timeout")]
    Timeout,
    #[error("Invalid benchmark configuration")]
    InvalidConfig,
    #[error("Benchmark execution failed: {0}")]
    ExecutionFailed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkConfig {
    pub target_url: String,
    pub concurrent_users: u32,
    pub duration_seconds: u64,
    pub ramp_up_seconds: u64,
    pub endpoints: Vec<EndpointConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointConfig {
    pub path: String,
    pub method: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
    pub weight: f32, // Probability weight for this endpoint
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            target_url: "http://localhost:3000".to_string(),
            concurrent_users: 100,
            duration_seconds: 60,
            ramp_up_seconds: 10,
            endpoints: vec![
                EndpointConfig {
                    path: "/health".to_string(),
                    method: "GET".to_string(),
                    headers: HashMap::new(),
                    body: None,
                    weight: 0.3,
                },
                EndpointConfig {
                    path: "/api/products".to_string(),
                    method: "GET".to_string(),
                    headers: HashMap::new(),
                    body: None,
                    weight: 0.4,
                },
                EndpointConfig {
                    path: "/api/users/me".to_string(),
                    method: "GET".to_string(),
                    headers: {
                        let mut headers = HashMap::new();
                        headers.insert("Authorization".to_string(), "Bearer demo-token".to_string());
                        headers
                    },
                    body: None,
                    weight: 0.2,
                },
                EndpointConfig {
                    path: "/graphql".to_string(),
                    method: "POST".to_string(),
                    headers: {
                        let mut headers = HashMap::new();
                        headers.insert("Content-Type".to_string(), "application/json".to_string());
                        headers
                    },
                    body: Some(r#"{"query":"query { health }"}"#.to_string()),
                    weight: 0.1,
                },
            ],
        }
    }
}

#[derive(Debug, Clone)]
pub struct RequestMetrics {
    pub start_time: Instant,
    pub end_time: Instant,
    pub status_code: u16,
    pub response_size: usize,
    pub endpoint: String,
    pub success: bool,
}

impl RequestMetrics {
    pub fn duration_ms(&self) -> f64 {
        self.end_time.duration_since(self.start_time).as_millis() as f64
    }
}

#[derive(Debug, Clone)]
pub struct BenchmarkMetrics {
    pub framework: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub total_bytes_received: u64,
    pub request_metrics: Vec<RequestMetrics>,
    pub error_counts: HashMap<String, u32>,
}

impl BenchmarkMetrics {
    pub fn new(framework: String) -> Self {
        Self {
            framework,
            start_time: Utc::now(),
            end_time: Utc::now(),
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            total_bytes_received: 0,
            request_metrics: Vec::new(),
            error_counts: HashMap::new(),
        }
    }

    pub fn add_request(&mut self, metrics: RequestMetrics) {
        self.total_requests += 1;
        self.total_bytes_received += metrics.response_size as u64;
        
        if metrics.success {
            self.successful_requests += 1;
        } else {
            self.failed_requests += 1;
            let error_key = format!("HTTP_{}", metrics.status_code);
            *self.error_counts.entry(error_key).or_insert(0) += 1;
        }
        
        self.request_metrics.push(metrics);
    }

    pub fn finalize(&mut self) {
        self.end_time = Utc::now();
    }

    pub fn duration_seconds(&self) -> f64 {
        (self.end_time - self.start_time).num_milliseconds() as f64 / 1000.0
    }

    pub fn requests_per_second(&self) -> f64 {
        self.total_requests as f64 / self.duration_seconds()
    }

    pub fn average_response_time_ms(&self) -> f64 {
        if self.request_metrics.is_empty() {
            return 0.0;
        }
        
        let total_time: f64 = self.request_metrics
            .iter()
            .map(|m| m.duration_ms())
            .sum();
        
        total_time / self.request_metrics.len() as f64
    }

    pub fn percentile_response_time_ms(&self, percentile: f64) -> f64 {
        if self.request_metrics.is_empty() {
            return 0.0;
        }

        let mut durations: Vec<f64> = self.request_metrics
            .iter()
            .map(|m| m.duration_ms())
            .collect();
        
        durations.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let index = ((percentile / 100.0) * durations.len() as f64) as usize;
        let clamped_index = index.min(durations.len() - 1);
        
        durations[clamped_index]
    }

    pub fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            return 0.0;
        }
        (self.successful_requests as f64 / self.total_requests as f64) * 100.0
    }

    pub fn throughput_mb_per_second(&self) -> f64 {
        let mb = self.total_bytes_received as f64 / (1024.0 * 1024.0);
        mb / self.duration_seconds()
    }

    pub fn to_benchmark_result(&self, test_name: String) -> BenchmarkResult {
        BenchmarkResult {
            framework: self.framework.clone(),
            test_name,
            requests_per_second: self.requests_per_second(),
            average_response_time_ms: self.average_response_time_ms(),
            p95_response_time_ms: self.percentile_response_time_ms(95.0),
            p99_response_time_ms: self.percentile_response_time_ms(99.0),
            memory_usage_mb: 0.0, // Would need system monitoring
            cpu_usage_percent: 0.0, // Would need system monitoring
            timestamp: Utc::now(),
        }
    }
}

pub struct LoadTester {
    client: reqwest::Client,
    config: BenchmarkConfig,
}

impl LoadTester {
    pub fn new(config: BenchmarkConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self { client, config }
    }

    pub async fn run_benchmark(&self, framework_name: String) -> Result<BenchmarkMetrics, BenchmarkError> {
        let mut metrics = BenchmarkMetrics::new(framework_name);
        
        println!("🚀 Starting benchmark for {} framework", metrics.framework);
        println!("📊 Config: {} users, {}s duration, {}s ramp-up", 
                 self.config.concurrent_users, 
                 self.config.duration_seconds, 
                 self.config.ramp_up_seconds);

        let _start_time = Instant::now();
        let benchmark_duration = std::time::Duration::from_secs(self.config.duration_seconds);
        
        // Create tasks for concurrent users
        let mut tasks = Vec::new();
        
        for user_id in 0..self.config.concurrent_users {
            let client = self.client.clone();
            let config = self.config.clone();
            let user_start_delay = (self.config.ramp_up_seconds * 1000 / self.config.concurrent_users as u64) * user_id as u64;
            
            let task = tokio::spawn(async move {
                // Ramp-up delay
                if user_start_delay > 0 {
                    tokio::time::sleep(std::time::Duration::from_millis(user_start_delay)).await;
                }
                
                let mut user_metrics = Vec::new();
                let user_start = Instant::now();
                
                while user_start.elapsed() < benchmark_duration {
                    // Select random endpoint based on weights
                    let endpoint = Self::select_weighted_endpoint(&config.endpoints);
                    
                    let request_start = Instant::now();
                    let mut request_builder = match endpoint.method.as_str() {
                        "GET" => client.get(&format!("{}{}", config.target_url, endpoint.path)),
                        "POST" => client.post(&format!("{}{}", config.target_url, endpoint.path)),
                        "PUT" => client.put(&format!("{}{}", config.target_url, endpoint.path)),
                        "DELETE" => client.delete(&format!("{}{}", config.target_url, endpoint.path)),
                        _ => client.get(&format!("{}{}", config.target_url, endpoint.path)),
                    };

                    // Add headers
                    for (key, value) in &endpoint.headers {
                        request_builder = request_builder.header(key, value);
                    }

                    // Add body if present
                    if let Some(body) = &endpoint.body {
                        request_builder = request_builder.body(body.clone());
                    }

                    // Execute request
                    match request_builder.send().await {
                        Ok(response) => {
                            let status_code = response.status().as_u16();
                            let response_size = response.content_length().unwrap_or(0) as usize;
                            let success = response.status().is_success();
                            
                            user_metrics.push(RequestMetrics {
                                start_time: request_start,
                                end_time: Instant::now(),
                                status_code,
                                response_size,
                                endpoint: endpoint.path.clone(),
                                success,
                            });
                        }
                        Err(_) => {
                            user_metrics.push(RequestMetrics {
                                start_time: request_start,
                                end_time: Instant::now(),
                                status_code: 0,
                                response_size: 0,
                                endpoint: endpoint.path.clone(),
                                success: false,
                            });
                        }
                    }

                    // Small delay between requests
                    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                }
                
                user_metrics
            });
            
            tasks.push(task);
        }

        // Wait for all tasks to complete
        for task in tasks {
            match task.await {
                Ok(user_metrics) => {
                    for request_metric in user_metrics {
                        metrics.add_request(request_metric);
                    }
                }
                Err(e) => {
                    eprintln!("Task failed: {}", e);
                }
            }
        }

        metrics.finalize();
        
        println!("✅ Benchmark completed for {} framework", metrics.framework);
        println!("📈 Results: {:.2} req/s, {:.2}ms avg response time, {:.1}% success rate",
                 metrics.requests_per_second(),
                 metrics.average_response_time_ms(),
                 metrics.success_rate());

        Ok(metrics)
    }

    fn select_weighted_endpoint(endpoints: &[EndpointConfig]) -> &EndpointConfig {
        use rand::Rng;
        
        let total_weight: f32 = endpoints.iter().map(|e| e.weight).sum();
        let mut rng = rand::thread_rng();
        let mut random_value: f32 = rng.gen_range(0.0..total_weight);
        
        for endpoint in endpoints {
            random_value -= endpoint.weight;
            if random_value <= 0.0 {
                return endpoint;
            }
        }
        
        // Fallback to first endpoint
        &endpoints[0]
    }
}

// Comparison utilities
pub struct FrameworkComparison {
    pub axum_results: Vec<BenchmarkResult>,
    pub loco_results: Vec<BenchmarkResult>,
}

impl FrameworkComparison {
    pub fn new() -> Self {
        Self {
            axum_results: Vec::new(),
            loco_results: Vec::new(),
        }
    }

    pub fn add_axum_result(&mut self, result: BenchmarkResult) {
        self.axum_results.push(result);
    }

    pub fn add_loco_result(&mut self, result: BenchmarkResult) {
        self.loco_results.push(result);
    }

    pub fn generate_comparison_report(&self) -> String {
        let mut report = String::new();
        
        report.push_str("# AXUM vs LOCO Performance Comparison Report\n\n");
        report.push_str(&format!("Generated at: {}\n\n", Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));

        // Summary table
        report.push_str("## Summary\n\n");
        report.push_str("| Framework | Avg RPS | Avg Response Time (ms) | P95 (ms) | P99 (ms) |\n");
        report.push_str("|-----------|---------|------------------------|----------|----------|\n");

        if let Some(axum_avg) = self.calculate_average_metrics(&self.axum_results) {
            report.push_str(&format!("| AXUM      | {:.2}    | {:.2}                   | {:.2}     | {:.2}     |\n",
                axum_avg.requests_per_second,
                axum_avg.average_response_time_ms,
                axum_avg.p95_response_time_ms,
                axum_avg.p99_response_time_ms));
        }

        if let Some(loco_avg) = self.calculate_average_metrics(&self.loco_results) {
            report.push_str(&format!("| LOCO      | {:.2}    | {:.2}                   | {:.2}     | {:.2}     |\n",
                loco_avg.requests_per_second,
                loco_avg.average_response_time_ms,
                loco_avg.p95_response_time_ms,
                loco_avg.p99_response_time_ms));
        }

        report.push_str("\n## Detailed Results\n\n");

        // AXUM results
        if !self.axum_results.is_empty() {
            report.push_str("### AXUM Framework Results\n\n");
            for result in &self.axum_results {
                report.push_str(&format!("**{}**\n", result.test_name));
                report.push_str(&format!("- Requests/sec: {:.2}\n", result.requests_per_second));
                report.push_str(&format!("- Avg response time: {:.2}ms\n", result.average_response_time_ms));
                report.push_str(&format!("- P95 response time: {:.2}ms\n", result.p95_response_time_ms));
                report.push_str(&format!("- P99 response time: {:.2}ms\n", result.p99_response_time_ms));
                report.push_str("\n");
            }
        }

        // LOCO results
        if !self.loco_results.is_empty() {
            report.push_str("### LOCO Framework Results\n\n");
            for result in &self.loco_results {
                report.push_str(&format!("**{}**\n", result.test_name));
                report.push_str(&format!("- Requests/sec: {:.2}\n", result.requests_per_second));
                report.push_str(&format!("- Avg response time: {:.2}ms\n", result.average_response_time_ms));
                report.push_str(&format!("- P95 response time: {:.2}ms\n", result.p95_response_time_ms));
                report.push_str(&format!("- P99 response time: {:.2}ms\n", result.p99_response_time_ms));
                report.push_str("\n");
            }
        }

        // Winner analysis
        report.push_str("## Analysis\n\n");
        if let (Some(axum_avg), Some(loco_avg)) = (
            self.calculate_average_metrics(&self.axum_results),
            self.calculate_average_metrics(&self.loco_results)
        ) {
            if axum_avg.requests_per_second > loco_avg.requests_per_second {
                let diff = ((axum_avg.requests_per_second - loco_avg.requests_per_second) / loco_avg.requests_per_second) * 100.0;
                report.push_str(&format!("🏆 **AXUM wins in throughput** by {:.1}% ({:.2} vs {:.2} req/s)\n\n",
                    diff, axum_avg.requests_per_second, loco_avg.requests_per_second));
            } else {
                let diff = ((loco_avg.requests_per_second - axum_avg.requests_per_second) / axum_avg.requests_per_second) * 100.0;
                report.push_str(&format!("🏆 **LOCO wins in throughput** by {:.1}% ({:.2} vs {:.2} req/s)\n\n",
                    diff, loco_avg.requests_per_second, axum_avg.requests_per_second));
            }

            if axum_avg.average_response_time_ms < loco_avg.average_response_time_ms {
                let diff = ((loco_avg.average_response_time_ms - axum_avg.average_response_time_ms) / loco_avg.average_response_time_ms) * 100.0;
                report.push_str(&format!("⚡ **AXUM wins in response time** by {:.1}% ({:.2}ms vs {:.2}ms)\n\n",
                    diff, axum_avg.average_response_time_ms, loco_avg.average_response_time_ms));
            } else {
                let diff = ((axum_avg.average_response_time_ms - loco_avg.average_response_time_ms) / axum_avg.average_response_time_ms) * 100.0;
                report.push_str(&format!("⚡ **LOCO wins in response time** by {:.1}% ({:.2}ms vs {:.2}ms)\n\n",
                    diff, loco_avg.average_response_time_ms, axum_avg.average_response_time_ms));
            }
        }

        report
    }

    fn calculate_average_metrics(&self, results: &[BenchmarkResult]) -> Option<BenchmarkResult> {
        if results.is_empty() {
            return None;
        }

        let count = results.len() as f64;
        Some(BenchmarkResult {
            framework: results[0].framework.clone(),
            test_name: "Average".to_string(),
            requests_per_second: results.iter().map(|r| r.requests_per_second).sum::<f64>() / count,
            average_response_time_ms: results.iter().map(|r| r.average_response_time_ms).sum::<f64>() / count,
            p95_response_time_ms: results.iter().map(|r| r.p95_response_time_ms).sum::<f64>() / count,
            p99_response_time_ms: results.iter().map(|r| r.p99_response_time_ms).sum::<f64>() / count,
            memory_usage_mb: results.iter().map(|r| r.memory_usage_mb).sum::<f64>() / count,
            cpu_usage_percent: results.iter().map(|r| r.cpu_usage_percent).sum::<f64>() / count,
            timestamp: Utc::now(),
        })
    }
}

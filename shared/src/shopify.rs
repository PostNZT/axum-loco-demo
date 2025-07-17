use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use anyhow::Result;
use thiserror::Error;
use reqwest::Client;

#[derive(Debug, Error)]
pub enum ShopifyError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),
    #[error("Authentication failed")]
    AuthenticationFailed,
    #[error("Product not found")]
    ProductNotFound,
    #[error("Order not found")]
    OrderNotFound,
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    #[error("Invalid webhook signature")]
    InvalidWebhookSignature,
    #[error("Shopify API error: {0}")]
    ApiError(String),
}

#[derive(Debug, Clone)]
pub struct ShopifyConfig {
    pub shop_domain: String,
    pub access_token: String,
    pub webhook_secret: String,
    pub api_version: String,
}

impl Default for ShopifyConfig {
    fn default() -> Self {
        Self {
            shop_domain: "your-shop.myshopify.com".to_string(),
            access_token: "your-access-token".to_string(),
            webhook_secret: "your-webhook-secret".to_string(),
            api_version: "2023-10".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopifyProduct {
    pub id: Option<i64>,
    pub title: String,
    pub body_html: Option<String>,
    pub vendor: String,
    pub product_type: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub published_at: Option<DateTime<Utc>>,
    pub template_suffix: Option<String>,
    pub status: String,
    pub published_scope: String,
    pub tags: String,
    pub admin_graphql_api_id: Option<String>,
    pub variants: Vec<ShopifyVariant>,
    pub options: Vec<ShopifyOption>,
    pub images: Vec<ShopifyImage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopifyVariant {
    pub id: Option<i64>,
    pub product_id: Option<i64>,
    pub title: String,
    pub price: String,
    pub sku: Option<String>,
    pub position: i32,
    pub inventory_policy: String,
    pub compare_at_price: Option<String>,
    pub fulfillment_service: String,
    pub inventory_management: Option<String>,
    pub option1: Option<String>,
    pub option2: Option<String>,
    pub option3: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub taxable: bool,
    pub barcode: Option<String>,
    pub grams: i32,
    pub image_id: Option<i64>,
    pub weight: f64,
    pub weight_unit: String,
    pub inventory_item_id: Option<i64>,
    pub inventory_quantity: i32,
    pub old_inventory_quantity: i32,
    pub requires_shipping: bool,
    pub admin_graphql_api_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopifyOption {
    pub id: Option<i64>,
    pub product_id: Option<i64>,
    pub name: String,
    pub position: i32,
    pub values: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopifyImage {
    pub id: Option<i64>,
    pub product_id: Option<i64>,
    pub position: i32,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub alt: Option<String>,
    pub width: i32,
    pub height: i32,
    pub src: String,
    pub variant_ids: Vec<i64>,
    pub admin_graphql_api_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopifyOrder {
    pub id: Option<i64>,
    pub admin_graphql_api_id: Option<String>,
    pub app_id: Option<i64>,
    pub browser_ip: Option<String>,
    pub buyer_accepts_marketing: bool,
    pub cancel_reason: Option<String>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub cart_token: Option<String>,
    pub checkout_id: Option<i64>,
    pub checkout_token: Option<String>,
    pub closed_at: Option<DateTime<Utc>>,
    pub confirmed: bool,
    pub contact_email: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub currency: String,
    pub current_subtotal_price: String,
    pub current_subtotal_price_set: Option<serde_json::Value>,
    pub current_total_discounts: String,
    pub current_total_discounts_set: Option<serde_json::Value>,
    pub current_total_duties_set: Option<serde_json::Value>,
    pub current_total_price: String,
    pub current_total_price_set: Option<serde_json::Value>,
    pub current_total_tax: String,
    pub current_total_tax_set: Option<serde_json::Value>,
    pub customer_locale: Option<String>,
    pub device_id: Option<i64>,
    pub discount_codes: Vec<serde_json::Value>,
    pub email: String,
    pub estimated_taxes: bool,
    pub financial_status: String,
    pub fulfillment_status: Option<String>,
    pub gateway: String,
    pub landing_site: Option<String>,
    pub landing_site_ref: Option<String>,
    pub location_id: Option<i64>,
    pub name: String,
    pub note: Option<String>,
    pub note_attributes: Vec<serde_json::Value>,
    pub number: i32,
    pub order_number: i32,
    pub order_status_url: String,
    pub original_total_duties_set: Option<serde_json::Value>,
    pub payment_gateway_names: Vec<String>,
    pub phone: Option<String>,
    pub presentment_currency: String,
    pub processed_at: Option<DateTime<Utc>>,
    pub processing_method: String,
    pub reference: Option<String>,
    pub referring_site: Option<String>,
    pub source_identifier: Option<String>,
    pub source_name: String,
    pub source_url: Option<String>,
    pub subtotal_price: String,
    pub subtotal_price_set: Option<serde_json::Value>,
    pub tags: String,
    pub tax_lines: Vec<serde_json::Value>,
    pub taxes_included: bool,
    pub test: bool,
    pub token: String,
    pub total_discounts: String,
    pub total_discounts_set: Option<serde_json::Value>,
    pub total_line_items_price: String,
    pub total_line_items_price_set: Option<serde_json::Value>,
    pub total_outstanding: String,
    pub total_price: String,
    pub total_price_set: Option<serde_json::Value>,
    pub total_price_usd: String,
    pub total_shipping_price_set: Option<serde_json::Value>,
    pub total_tax: String,
    pub total_tax_set: Option<serde_json::Value>,
    pub total_tip_received: String,
    pub total_weight: i32,
    pub updated_at: Option<DateTime<Utc>>,
    pub user_id: Option<i64>,
    pub billing_address: Option<serde_json::Value>,
    pub customer: Option<serde_json::Value>,
    pub discount_applications: Vec<serde_json::Value>,
    pub fulfillments: Vec<serde_json::Value>,
    pub line_items: Vec<serde_json::Value>,
    pub payment_terms: Option<serde_json::Value>,
    pub refunds: Vec<serde_json::Value>,
    pub shipping_address: Option<serde_json::Value>,
    pub shipping_lines: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopifyWebhook {
    pub topic: String,
    pub shop_domain: String,
    pub payload: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

pub struct ShopifyClient {
    client: Client,
    config: ShopifyConfig,
}

impl ShopifyClient {
    pub fn new(config: ShopifyConfig) -> Self {
        let client = Client::new();
        Self { client, config }
    }

    fn base_url(&self) -> String {
        format!("https://{}/admin/api/{}", self.config.shop_domain, self.config.api_version)
    }

    pub async fn get_products(&self) -> Result<Vec<ShopifyProduct>, ShopifyError> {
        let url = format!("{}/products.json", self.base_url());
        
        let response = self.client
            .get(&url)
            .header("X-Shopify-Access-Token", &self.config.access_token)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(ShopifyError::ApiError(format!("HTTP {}", response.status())));
        }

        let json: serde_json::Value = response.json().await?;
        let products = json["products"].as_array()
            .ok_or_else(|| ShopifyError::ApiError("Invalid response format".to_string()))?;

        let mut result = Vec::new();
        for product_json in products {
            if let Ok(product) = serde_json::from_value::<ShopifyProduct>(product_json.clone()) {
                result.push(product);
            }
        }

        Ok(result)
    }

    pub async fn get_product(&self, product_id: i64) -> Result<ShopifyProduct, ShopifyError> {
        let url = format!("{}/products/{}.json", self.base_url(), product_id);
        
        let response = self.client
            .get(&url)
            .header("X-Shopify-Access-Token", &self.config.access_token)
            .send()
            .await?;

        if response.status() == 404 {
            return Err(ShopifyError::ProductNotFound);
        }

        if !response.status().is_success() {
            return Err(ShopifyError::ApiError(format!("HTTP {}", response.status())));
        }

        let json: serde_json::Value = response.json().await?;
        let product = serde_json::from_value(json["product"].clone())
            .map_err(|e| ShopifyError::ApiError(e.to_string()))?;

        Ok(product)
    }

    pub async fn create_product(&self, product: &ShopifyProduct) -> Result<ShopifyProduct, ShopifyError> {
        let url = format!("{}/products.json", self.base_url());
        
        let payload = serde_json::json!({
            "product": product
        });

        let response = self.client
            .post(&url)
            .header("X-Shopify-Access-Token", &self.config.access_token)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(ShopifyError::ApiError(format!("HTTP {}", response.status())));
        }

        let json: serde_json::Value = response.json().await?;
        let product = serde_json::from_value(json["product"].clone())
            .map_err(|e| ShopifyError::ApiError(e.to_string()))?;

        Ok(product)
    }

    pub async fn get_orders(&self) -> Result<Vec<ShopifyOrder>, ShopifyError> {
        let url = format!("{}/orders.json", self.base_url());
        
        let response = self.client
            .get(&url)
            .header("X-Shopify-Access-Token", &self.config.access_token)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(ShopifyError::ApiError(format!("HTTP {}", response.status())));
        }

        let json: serde_json::Value = response.json().await?;
        let orders = json["orders"].as_array()
            .ok_or_else(|| ShopifyError::ApiError("Invalid response format".to_string()))?;

        let mut result = Vec::new();
        for order_json in orders {
            if let Ok(order) = serde_json::from_value::<ShopifyOrder>(order_json.clone()) {
                result.push(order);
            }
        }

        Ok(result)
    }

    pub async fn get_order(&self, order_id: i64) -> Result<ShopifyOrder, ShopifyError> {
        let url = format!("{}/orders/{}.json", self.base_url(), order_id);
        
        let response = self.client
            .get(&url)
            .header("X-Shopify-Access-Token", &self.config.access_token)
            .send()
            .await?;

        if response.status() == 404 {
            return Err(ShopifyError::OrderNotFound);
        }

        if !response.status().is_success() {
            return Err(ShopifyError::ApiError(format!("HTTP {}", response.status())));
        }

        let json: serde_json::Value = response.json().await?;
        let order = serde_json::from_value(json["order"].clone())
            .map_err(|e| ShopifyError::ApiError(e.to_string()))?;

        Ok(order)
    }

    pub fn verify_webhook(&self, payload: &str, signature: &str) -> Result<bool, ShopifyError> {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;
        use base64::Engine;

        type HmacSha256 = Hmac<Sha256>;

        let mut mac = HmacSha256::new_from_slice(self.config.webhook_secret.as_bytes())
            .map_err(|_| ShopifyError::InvalidWebhookSignature)?;
        
        mac.update(payload.as_bytes());
        
        let expected = mac.finalize().into_bytes();
        let expected_b64 = base64::engine::general_purpose::STANDARD.encode(expected);
        
        Ok(signature == expected_b64)
    }
}

// Utility functions for Shopify integration
pub fn extract_shopify_id_from_gid(gid: &str) -> Option<i64> {
    gid.split('/').last()?.parse().ok()
}

pub fn create_shopify_gid(resource_type: &str, id: i64) -> String {
    format!("gid://shopify/{}/{}", resource_type, id)
}

// Mock Shopify client for testing and demo purposes
pub struct MockShopifyClient {
    products: Vec<ShopifyProduct>,
    orders: Vec<ShopifyOrder>,
}

impl MockShopifyClient {
    pub fn new() -> Self {
        Self {
            products: Self::create_mock_products(),
            orders: Self::create_mock_orders(),
        }
    }

    fn create_mock_products() -> Vec<ShopifyProduct> {
        vec![
            ShopifyProduct {
                id: Some(1),
                title: "Demo Product 1".to_string(),
                body_html: Some("<p>This is a demo product</p>".to_string()),
                vendor: "Demo Vendor".to_string(),
                product_type: "Demo Type".to_string(),
                created_at: Some(Utc::now()),
                updated_at: Some(Utc::now()),
                published_at: Some(Utc::now()),
                template_suffix: None,
                status: "active".to_string(),
                published_scope: "web".to_string(),
                tags: "demo,test".to_string(),
                admin_graphql_api_id: Some("gid://shopify/Product/1".to_string()),
                variants: vec![],
                options: vec![],
                images: vec![],
            },
            ShopifyProduct {
                id: Some(2),
                title: "Demo Product 2".to_string(),
                body_html: Some("<p>This is another demo product</p>".to_string()),
                vendor: "Demo Vendor".to_string(),
                product_type: "Demo Type".to_string(),
                created_at: Some(Utc::now()),
                updated_at: Some(Utc::now()),
                published_at: Some(Utc::now()),
                template_suffix: None,
                status: "active".to_string(),
                published_scope: "web".to_string(),
                tags: "demo,test,featured".to_string(),
                admin_graphql_api_id: Some("gid://shopify/Product/2".to_string()),
                variants: vec![],
                options: vec![],
                images: vec![],
            },
        ]
    }

    fn create_mock_orders() -> Vec<ShopifyOrder> {
        vec![]
    }

    pub async fn get_products(&self) -> Result<Vec<ShopifyProduct>, ShopifyError> {
        Ok(self.products.clone())
    }

    pub async fn get_product(&self, product_id: i64) -> Result<ShopifyProduct, ShopifyError> {
        self.products
            .iter()
            .find(|p| p.id == Some(product_id))
            .cloned()
            .ok_or(ShopifyError::ProductNotFound)
    }

    pub async fn create_product(&self, product: &ShopifyProduct) -> Result<ShopifyProduct, ShopifyError> {
        let mut new_product = product.clone();
        new_product.id = Some(999);
        new_product.created_at = Some(Utc::now());
        new_product.updated_at = Some(Utc::now());
        Ok(new_product)
    }

    pub async fn get_orders(&self) -> Result<Vec<ShopifyOrder>, ShopifyError> {
        Ok(self.orders.clone())
    }
}

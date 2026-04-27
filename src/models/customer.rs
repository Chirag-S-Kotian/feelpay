use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgRow;
use sqlx::Row;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Customer {
    pub id: Uuid,
    pub merchant_id: Uuid,
    pub tenant_id: Option<Uuid>,
    pub customer_reference: String,
    pub customer_reference_type: CustomerReferenceType,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub name: Option<String>,
    pub consent_given: bool,
    pub consent_timestamp: Option<DateTime<Utc>>,
    pub consent_purpose: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CustomerReferenceType {
    Email,
    Phone,
    ExternalId,
}

impl From<&str> for CustomerReferenceType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "email" => CustomerReferenceType::Email,
            "phone" => CustomerReferenceType::Phone,
            _ => CustomerReferenceType::ExternalId,
        }
    }
}

impl ToString for CustomerReferenceType {
    fn to_string(&self) -> String {
        match self {
            CustomerReferenceType::Email => "email".to_string(),
            CustomerReferenceType::Phone => "phone".to_string(),
            CustomerReferenceType::ExternalId => "external_id".to_string(),
        }
    }
}

impl From<PgRow> for Customer {
    fn from(row: PgRow) -> Self {
        Customer {
            id: row.get("id"),
            merchant_id: row.get("merchant_id"),
            tenant_id: row.get("tenant_id"),
            customer_reference: row.get("customer_reference"),
            customer_reference_type: CustomerReferenceType::from(
                row.get::<String, _>("customer_reference_type").as_str(),
            ),
            email: row.get("email"),
            phone: row.get("phone"),
            name: row.get("name"),
            consent_given: row.get("consent_given"),
            consent_timestamp: row.get("consent_timestamp"),
            consent_purpose: row.get("consent_purpose"),
            metadata: row.get("metadata"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            deleted_at: row.get("deleted_at"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCustomerRequest {
    pub customer_reference: String,
    pub customer_reference_type: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub name: Option<String>,
    pub consent_given: Option<bool>,
    pub consent_purpose: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerResponse {
    pub id: Uuid,
    pub merchant_id: Uuid,
    pub customer_reference: String,
    pub customer_reference_type: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub name: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl From<Customer> for CustomerResponse {
    fn from(c: Customer) -> Self {
        CustomerResponse {
            id: c.id,
            merchant_id: c.merchant_id,
            customer_reference: c.customer_reference,
            customer_reference_type: c.customer_reference_type.to_string(),
            email: c.email,
            phone: c.phone,
            name: c.name,
            created_at: c.created_at,
        }
    }
}

// Saved Payment Methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedPaymentMethod {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub merchant_id: Uuid,
    pub tenant_id: Option<Uuid>,
    pub method_type: PaymentMethodType,
    pub vault_token: String,
    pub card_brand: Option<String>,
    pub card_last4: Option<String>,
    pub card_expiry_month: Option<i32>,
    pub card_expiry_year: Option<i32>,
    pub card_type: Option<String>,
    pub card_network: Option<String>,
    pub vpa: Option<String>,
    pub bank_name: Option<String>,
    pub bank_account_last4: Option<String>,
    pub ifsc_code: Option<String>,
    pub is_default: bool,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PaymentMethodType {
    Card,
    Upi,
    BankAccount,
    Wallet,
}

impl From<&str> for PaymentMethodType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "card" => PaymentMethodType::Card,
            "upi" => PaymentMethodType::Upi,
            "bank_account" => PaymentMethodType::BankAccount,
            "wallet" => PaymentMethodType::Wallet,
            _ => PaymentMethodType::Card,
        }
    }
}

impl ToString for PaymentMethodType {
    fn to_string(&self) -> String {
        match self {
            PaymentMethodType::Card => "card".to_string(),
            PaymentMethodType::Upi => "upi".to_string(),
            PaymentMethodType::BankAccount => "bank_account".to_string(),
            PaymentMethodType::Wallet => "wallet".to_string(),
        }
    }
}

impl From<PgRow> for SavedPaymentMethod {
    fn from(row: PgRow) -> Self {
        SavedPaymentMethod {
            id: row.get("id"),
            customer_id: row.get("customer_id"),
            merchant_id: row.get("merchant_id"),
            tenant_id: row.get("tenant_id"),
            method_type: PaymentMethodType::from(row.get::<String, _>("method_type").as_str()),
            vault_token: row.get("vault_token"),
            card_brand: row.get("card_brand"),
            card_last4: row.get("card_last4"),
            card_expiry_month: row.get("card_expiry_month"),
            card_expiry_year: row.get("card_expiry_year"),
            card_type: row.get("card_type"),
            card_network: row.get("card_network"),
            vpa: row.get("vpa"),
            bank_name: row.get("bank_name"),
            bank_account_last4: row.get("bank_account_last4"),
            ifsc_code: row.get("ifsc_code"),
            is_default: row.get("is_default"),
            is_active: row.get("is_active"),
            metadata: row.get("metadata"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            deleted_at: row.get("deleted_at"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSavedPaymentMethodRequest {
    pub customer_id: Uuid,
    pub method_type: String,
    pub vault_token: String,
    // Card fields
    pub card_brand: Option<String>,
    pub card_last4: Option<String>,
    pub card_expiry_month: Option<i32>,
    pub card_expiry_year: Option<i32>,
    pub card_type: Option<String>,
    pub card_network: Option<String>,
    // UPI fields
    pub vpa: Option<String>,
    // Bank fields
    pub bank_name: Option<String>,
    pub bank_account_last4: Option<String>,
    pub ifsc_code: Option<String>,
    pub is_default: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedPaymentMethodResponse {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub method_type: String,
    pub is_default: bool,
    pub card_brand: Option<String>,
    pub card_last4: Option<String>,
    pub card_expiry_month: Option<i32>,
    pub card_expiry_year: Option<i32>,
    pub vpa: Option<String>,
    pub bank_name: Option<String>,
    pub bank_account_last4: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl From<SavedPaymentMethod> for SavedPaymentMethodResponse {
    fn from(pm: SavedPaymentMethod) -> Self {
        SavedPaymentMethodResponse {
            id: pm.id,
            customer_id: pm.customer_id,
            method_type: pm.method_type.to_string(),
            is_default: pm.is_default,
            card_brand: pm.card_brand,
            card_last4: pm.card_last4,
            card_expiry_month: pm.card_expiry_month,
            card_expiry_year: pm.card_expiry_year,
            vpa: pm.vpa,
            bank_name: pm.bank_name,
            bank_account_last4: pm.bank_account_last4,
            created_at: pm.created_at,
        }
    }
}
use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use utoipa::{ToSchema, IntoParams};
use crate::error::FeelPayError;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/emi-bnpl")
            .route("/emi-plans", web::get().to(get_emi_plans))
            .route("/bnpl-providers", web::get().to(get_bnpl_providers)),
    );
}

#[derive(Debug, Serialize, ToSchema)]
pub struct EmiPlan {
    pub bank_name: String,
    pub bank_code: String,
    pub duration_months: u8,
    pub interest_rate: f64,
    pub monthly_installment: i64,
    pub total_amount: i64,
}

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct EmiPlansQuery {
    pub amount: i64,
}

#[utoipa::path(
    get,
    path = "/v1/emi-bnpl/emi-plans",
    params(
        EmiPlansQuery
    ),
    responses(
        (status = 200, description = "Returns available EMI plans", body = [EmiPlan])
    )
)]
pub async fn get_emi_plans(
    _req: HttpRequest,
    query: web::Query<EmiPlansQuery>,
) -> Result<HttpResponse, FeelPayError> {
    let amount = query.amount;

    // Simulated EMI plans from major Indian banks
    let plans = vec![
        EmiPlan {
            bank_name: "HDFC Bank".into(),
            bank_code: "HDFC".into(),
            duration_months: 3,
            interest_rate: 12.0,
            monthly_installment: (amount as f64 * 1.03 / 3.0) as i64,
            total_amount: (amount as f64 * 1.03) as i64,
        },
        EmiPlan {
            bank_name: "ICICI Bank".into(),
            bank_code: "ICICI".into(),
            duration_months: 6,
            interest_rate: 14.0,
            monthly_installment: (amount as f64 * 1.07 / 6.0) as i64,
            total_amount: (amount as f64 * 1.07) as i64,
        },
        EmiPlan {
            bank_name: "SBI".into(),
            bank_code: "SBI".into(),
            duration_months: 9,
            interest_rate: 13.5,
            monthly_installment: (amount as f64 * 1.10 / 9.0) as i64,
            total_amount: (amount as f64 * 1.10) as i64,
        },
    ];

    Ok(HttpResponse::Ok().json(plans))
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BnplProvider {
    pub name: String,
    pub code: String,
    pub description: String,
}

#[utoipa::path(
    get,
    path = "/v1/emi-bnpl/bnpl-providers",
    responses(
        (status = 200, description = "Returns available BNPL providers", body = [BnplProvider])
    )
)]
pub async fn get_bnpl_providers(
    _req: HttpRequest,
) -> Result<HttpResponse, FeelPayError> {
    let providers = vec![
        BnplProvider {
            name: "Simpl".into(),
            code: "simpl".into(),
            description: "Buy now, pay later in 3 interest-free installments".into(),
        },
        BnplProvider {
            name: "LazyPay".into(),
            code: "lazypay".into(),
            description: "Pay later across thousands of apps".into(),
        },
        BnplProvider {
            name: "ZestMoney".into(),
            code: "zestmoney".into(),
            description: "No cost EMI without a credit card".into(),
        },
    ];

    Ok(HttpResponse::Ok().json(providers))
}

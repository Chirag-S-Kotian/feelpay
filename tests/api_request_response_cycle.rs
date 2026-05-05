use actix_web::{http::StatusCode, test, web, App};
use base64::Engine;

use feelpay::api;

#[actix_web::test]
async fn webhook_verify_cycle_valid_signature() {
    let app = test::init_service(App::new().route(
        "/v1/webhooks/verify",
        web::post().to(api::webhooks::verify_webhook),
    ))
    .await;

    let payload = br#"{"event":"payment.success","amount_paise":1000}"#;
    let payload_b64 = base64::engine::general_purpose::STANDARD.encode(payload);
    let secret = "test-secret";

    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    let ts = chrono::Utc::now().timestamp();
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(payload);
    let sig = hex::encode(mac.finalize().into_bytes());
    let signature = format!("t={},v1={}", ts, sig);

    let req = test::TestRequest::post()
        .uri("/v1/webhooks/verify")
        .set_json(serde_json::json!({
            "payload": payload_b64,
            "signature": signature,
            "secret": secret
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["valid"], true);
}

#[actix_web::test]
async fn webhook_verify_cycle_bad_signature() {
    let app = test::init_service(App::new().route(
        "/v1/webhooks/verify",
        web::post().to(api::webhooks::verify_webhook),
    ))
    .await;

    let payload = br#"{"event":"payment.failed","amount_paise":1000}"#;
    let payload_b64 = base64::engine::general_purpose::STANDARD.encode(payload);
    let req = test::TestRequest::post()
        .uri("/v1/webhooks/verify")
        .set_json(serde_json::json!({
            "payload": payload_b64,
            "signature": "t=1,v1=deadbeef",
            "secret": "bad-secret"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["valid"], false);
}

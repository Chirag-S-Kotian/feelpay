use feelpay::engine::services::webhook_test::WebhookTestService;

#[tokio::test]
async fn webhook_signature_roundtrip_should_verify() {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect_lazy("postgres://localhost/test")
        .expect("lazy pool");
    let service = WebhookTestService::new(pool);
    let payload = r#"{"event_type":"payment.success","amount_paise":1000}"#;
    let secret = "test-secret";

    let sig = service.sign_webhook(payload, secret).expect("sign");
    let ok = service
        .verify_webhook_signature(payload, &sig, secret)
        .expect("verify");
    assert!(ok);
}

#[tokio::test]
async fn webhook_signature_invalid_should_fail() {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect_lazy("postgres://localhost/test")
        .expect("lazy pool");
    let service = WebhookTestService::new(pool);
    let payload = r#"{"event_type":"payment.failed","amount_paise":1000}"#;
    let secret = "test-secret";

    let ok = service
        .verify_webhook_signature(payload, "bad-signature", secret)
        .expect("verify");
    assert!(!ok);
}

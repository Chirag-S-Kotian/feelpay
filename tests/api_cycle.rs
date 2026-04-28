use actix_web::{http::StatusCode, test, web, App};

use feelpay::api;

#[actix_web::test]
async fn health_cycle_should_return_ok() {
    let app =
        test::init_service(App::new().route("/v1/health", web::get().to(api::health::health_check)))
            .await;

    let req = test::TestRequest::get().uri("/v1/health").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_web::test]
async fn health_cycle_json_shape() {
    let app =
        test::init_service(App::new().route("/v1/health", web::get().to(api::health::health_check)))
            .await;

    let req = test::TestRequest::get().uri("/v1/health").to_request();
    let body: serde_json::Value = test::call_and_read_body_json(&app, req).await;
    assert!(body.get("status").is_some());
}


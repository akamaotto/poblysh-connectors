#!/usr/bin/env rust-script

//! Quick verification that provider errors follow the spec exactly:
//! "Provider upstream HTTP errors → 502 PROVIDER_ERROR with provider/status metadata in details"

use axum::http::StatusCode;
use connectors::error::provider_error;
use serde_json::json;

fn main() {
    println!("=== Testing Provider Error Spec Compliance ===");

    // Test all possible upstream status codes to verify they ALL return 502
    let test_cases = vec![
        (200, "Success but invalid format"),
        (400, "Bad request from provider"),
        (401, "Unauthorized from provider"),
        (403, "Forbidden from provider"),
        (404, "Not found from provider"),
        (429, "Rate limited from provider"),
        (500, "Internal server error from provider"),
        (502, "Bad gateway from provider"),
        (503, "Service unavailable from provider"),
        (504, "Gateway timeout from provider"),
    ];

    for (upstream_status, message) in test_cases {
        println!("\nTesting upstream {} error:", upstream_status);

        let error = provider_error(
            "test-provider".to_string(),
            upstream_status,
            Some(message.to_string()),
        );

        // SPEC REQUIREMENT: ALL provider upstream HTTP errors → 502 PROVIDER_ERROR
        assert_eq!(
            error.status,
            StatusCode::BAD_GATEWAY,
            "Should return HTTP 502 for upstream {}",
            upstream_status
        );
        assert_eq!(
            error.code, "PROVIDER_ERROR",
            "Should return PROVIDER_ERROR code for upstream {}",
            upstream_status
        );

        // SPEC REQUIREMENT: with provider/status metadata in details
        let details = error.details.as_ref().expect("Should have details");
        let details_obj = details.as_object().expect("Details should be object");

        assert_eq!(
            details_obj.get("provider").unwrap(),
            "test-provider",
            "Should include provider name"
        );
        assert_eq!(
            details_obj.get("status").unwrap(),
            upstream_status,
            "Should include upstream status"
        );
        assert!(
            details_obj.get("body_snippet").is_some(),
            "Should include body snippet"
        );

        println!(
            "  ✅ HTTP Status: {} (502 BAD_GATEWAY)",
            error.status.as_u16()
        );
        println!("  ✅ Error Code: {}", error.code);
        println!(
            "  ✅ Details: {}",
            serde_json::to_string_pretty(&details).unwrap()
        );
    }

    println!("\n=== All Provider Errors Follow Spec ===");
    println!("✅ ALL upstream HTTP errors return 502 PROVIDER_ERROR");
    println!("✅ ALL include provider/status metadata in details");
    println!("✅ Spec compliance verified!");
}

use std::time::Duration;
use chronos::app::middleware::security::{
    SecurityState, check_registration_rate_limit, check_refresh_rate_limit,
    check_password_reset_rate_limit, log_security_event
};

#[tokio::test]
async fn test_registration_rate_limiting() {
    let security_state = SecurityState::new();
    let test_ip = "192.168.1.1";

    // First 5 attempts should succeed
    for i in 0..5 {
        let result = check_registration_rate_limit(&security_state, test_ip);
        assert!(result.is_ok(), "Attempt {} should succeed", i + 1);
    }

    // 6th attempt should fail due to rate limit
    let result = check_registration_rate_limit(&security_state, test_ip);
    assert!(result.is_err(), "6th attempt should be rate limited");
}

#[tokio::test]
async fn test_refresh_rate_limiting() {
    let security_state = SecurityState::new();
    let test_user_id = "user-123";

    // First 10 attempts should succeed
    for i in 0..10 {
        let result = check_refresh_rate_limit(&security_state, test_user_id);
        assert!(result.is_ok(), "Refresh attempt {} should succeed", i + 1);
    }

    // 11th attempt should fail due to rate limit
    let result = check_refresh_rate_limit(&security_state, test_user_id);
    assert!(result.is_err(), "11th refresh attempt should be rate limited");
}

#[tokio::test]
async fn test_password_reset_rate_limiting() {
    let security_state = SecurityState::new();
    let test_email = "test@example.com";

    // First 3 attempts should succeed
    for i in 0..3 {
        let result = check_password_reset_rate_limit(&security_state, test_email);
        assert!(result.is_ok(), "Password reset attempt {} should succeed", i + 1);
    }

    // 4th attempt should fail due to rate limit
    let result = check_password_reset_rate_limit(&security_state, test_email);
    assert!(result.is_err(), "4th password reset attempt should be rate limited");
}

#[tokio::test]
async fn test_rate_limit_isolation() {
    let security_state = SecurityState::new();

    // Different IPs should have separate rate limits
    let ip1 = "192.168.1.1";
    let ip2 = "192.168.1.2";

    // Exhaust rate limit for ip1
    for _ in 0..5 {
        check_registration_rate_limit(&security_state, ip1).unwrap();
    }

    // ip1 should be rate limited
    assert!(check_registration_rate_limit(&security_state, ip1).is_err());

    // ip2 should still work
    assert!(check_registration_rate_limit(&security_state, ip2).is_ok());
}

#[tokio::test]
async fn test_rate_limit_different_users() {
    let security_state = SecurityState::new();

    let user1 = "user-123";
    let user2 = "user-456";

    // Exhaust rate limit for user1
    for _ in 0..10 {
        check_refresh_rate_limit(&security_state, user1).unwrap();
    }

    // user1 should be rate limited
    assert!(check_refresh_rate_limit(&security_state, user1).is_err());

    // user2 should still work
    assert!(check_refresh_rate_limit(&security_state, user2).is_ok());
}

#[tokio::test]
async fn test_rate_limit_different_emails() {
    let security_state = SecurityState::new();

    let email1 = "test1@example.com";
    let email2 = "test2@example.com";

    // Exhaust rate limit for email1
    for _ in 0..3 {
        check_password_reset_rate_limit(&security_state, email1).unwrap();
    }

    // email1 should be rate limited
    assert!(check_password_reset_rate_limit(&security_state, email1).is_err());

    // email2 should still work
    assert!(check_password_reset_rate_limit(&security_state, email2).is_ok());
}

#[test]
fn test_security_logging() {
    // This test ensures that security logging doesn't panic and accepts various input types
    log_security_event(
        "test_event",
        "192.168.1.1",
        Some("test-user-agent"),
        Some("user-123"),
        Some("test@example.com"),
        true,
        Some("Test details"),
    );

    log_security_event(
        "test_event_failure",
        "192.168.1.1",
        None,
        None,
        None,
        false,
        None,
    );

    // If we reach here without panicking, the logging works correctly
    assert!(true);
}

#[tokio::test]
async fn test_security_state_creation() {
    let security_state = SecurityState::new();

    // Verify state is properly initialized
    assert_eq!(security_state.ip_rate_limiter.len(), 0);
    assert_eq!(security_state.user_rate_limiter.len(), 0);
    assert_eq!(security_state.email_rate_limiter.len(), 0);
}

#[tokio::test]
async fn test_security_state_default() {
    let security_state = SecurityState::default();

    // Verify default implementation works the same as new()
    assert_eq!(security_state.ip_rate_limiter.len(), 0);
    assert_eq!(security_state.user_rate_limiter.len(), 0);
    assert_eq!(security_state.email_rate_limiter.len(), 0);
}

#[tokio::test]
async fn test_concurrent_rate_limiting() {
    use tokio::task::JoinSet;

    let security_state = SecurityState::new();
    let test_ip = "192.168.1.100";

    let mut join_set = JoinSet::new();

    // Spawn 10 concurrent registration attempts
    for i in 0..10 {
        let security_state_clone = security_state.clone();
        let ip_clone = test_ip.to_string();

        join_set.spawn(async move {
            tokio::time::sleep(Duration::from_millis(i * 10)).await;
            check_registration_rate_limit(&security_state_clone, &ip_clone)
        });
    }

    let mut success_count = 0;
    let mut failure_count = 0;

    while let Some(result) = join_set.join_next().await {
        match result.unwrap() {
            Ok(_) => success_count += 1,
            Err(_) => failure_count += 1,
        }
    }

    // Should have exactly 5 successes due to rate limiting
    assert_eq!(success_count, 5);
    assert_eq!(failure_count, 5);
}

// Integration test that simulates realistic rate limiting scenarios
#[tokio::test]
async fn test_realistic_rate_limiting_scenario() {
    let security_state = SecurityState::new();

    // Simulate multiple users from different IPs performing various operations
    let scenarios = vec![
        ("192.168.1.10", "user1@test.com", "user-001"),
        ("192.168.1.20", "user2@test.com", "user-002"),
        ("192.168.1.30", "user3@test.com", "user-003"),
    ];

    for (ip, email, user_id) in scenarios {
        // Each IP should be able to do 5 registrations
        for i in 0..5 {
            let result = check_registration_rate_limit(&security_state, ip);
            assert!(result.is_ok(), "Registration {} for {} should succeed", i + 1, ip);
        }

        // 6th registration should fail
        assert!(check_registration_rate_limit(&security_state, ip).is_err(),
                "6th registration for {} should fail", ip);

        // Each email should be able to do 3 password resets
        for i in 0..3 {
            let result = check_password_reset_rate_limit(&security_state, email);
            assert!(result.is_ok(), "Password reset {} for {} should succeed", i + 1, email);
        }

        // 4th password reset should fail
        assert!(check_password_reset_rate_limit(&security_state, email).is_err(),
                "4th password reset for {} should fail", email);

        // Each user should be able to do 10 refresh attempts
        for i in 0..10 {
            let result = check_refresh_rate_limit(&security_state, user_id);
            assert!(result.is_ok(), "Refresh {} for {} should succeed", i + 1, user_id);
        }

        // 11th refresh should fail
        assert!(check_refresh_rate_limit(&security_state, user_id).is_err(),
                "11th refresh for {} should fail", user_id);
    }
}
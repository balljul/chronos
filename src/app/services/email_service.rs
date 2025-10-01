use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct EmailMessage {
    pub to: String,
    pub subject: String,
    pub body: String,
    pub sent_at: time::OffsetDateTime,
}

// Mock email service for testing - stores emails in memory
pub struct MockEmailService {
    sent_emails: Arc<Mutex<Vec<EmailMessage>>>,
}

impl MockEmailService {
    pub fn new() -> Self {
        Self {
            sent_emails: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn send_password_reset_email(&self, email: &str, token: &str) -> Result<(), EmailError> {
        let subject = "Password Reset Request - Chronos".to_string();
        let body = format!(
            r#"
Hello,

You have requested a password reset for your Chronos account.

To reset your password, please use the following reset token:

{}

This token will expire in 1 hour. If you did not request this password reset, please ignore this email.

For security reasons, please do not share this token with anyone.

Best regards,
The Chronos Team
            "#,
            token
        );

        let message = EmailMessage {
            to: email.to_string(),
            subject,
            body,
            sent_at: time::OffsetDateTime::now_utc(),
        };

        // Store the email in our mock service
        if let Ok(mut emails) = self.sent_emails.lock() {
            emails.push(message);
        }

        // Simulate potential email sending delay
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        println!("📧 Mock Email Sent to: {}", email);
        println!("🔐 Reset Token: {}", token);

        Ok(())
    }

    // Helper method for testing - get all sent emails
    pub fn get_sent_emails(&self) -> Vec<EmailMessage> {
        if let Ok(emails) = self.sent_emails.lock() {
            emails.clone()
        } else {
            Vec::new()
        }
    }

    // Helper method for testing - clear sent emails
    pub fn clear_sent_emails(&self) {
        if let Ok(mut emails) = self.sent_emails.lock() {
            emails.clear();
        }
    }

    // Helper method for testing - count sent emails
    pub fn count_sent_emails(&self) -> usize {
        if let Ok(emails) = self.sent_emails.lock() {
            emails.len()
        } else {
            0
        }
    }

    // Helper method for testing - get last sent email
    pub fn get_last_sent_email(&self) -> Option<EmailMessage> {
        if let Ok(emails) = self.sent_emails.lock() {
            emails.last().cloned()
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub enum EmailError {
    SendingFailed(String),
    InvalidRecipient(String),
    TemplateError(String),
}

impl std::fmt::Display for EmailError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            EmailError::SendingFailed(e) => write!(f, "Email sending failed: {}", e),
            EmailError::InvalidRecipient(e) => write!(f, "Invalid recipient: {}", e),
            EmailError::TemplateError(e) => write!(f, "Template error: {}", e),
        }
    }
}

impl std::error::Error for EmailError {}

// Trait for email service to allow easy swapping between mock and real implementations
pub trait EmailServiceTrait {
    async fn send_password_reset_email(&self, email: &str, token: &str) -> Result<(), EmailError>;
}

impl EmailServiceTrait for MockEmailService {
    async fn send_password_reset_email(&self, email: &str, token: &str) -> Result<(), EmailError> {
        self.send_password_reset_email(email, token).await
    }
}
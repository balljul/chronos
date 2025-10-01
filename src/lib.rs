use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Server modules (only compiled when server feature is enabled)
#[cfg(feature = "server")]
pub mod app;
#[cfg(feature = "server")]
pub mod build;
#[cfg(feature = "server")]
pub mod routes;

// WebAssembly modules (only compiled for WASM target)
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;
#[cfg(target_arch = "wasm32")]
use web_sys::console;
#[cfg(target_arch = "wasm32")]
use gloo_net::http::Request;
#[cfg(target_arch = "wasm32")]
use gloo_storage::{LocalStorage, Storage};

#[cfg(target_arch = "wasm32")]
pub mod wasm_frontend;

// Re-export the main function for the CLI version (only for server builds)
#[cfg(feature = "server")]
pub use crate::routes::create_router;

// WebAssembly entry point
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    console::log_1(&"Chronos WebAssembly frontend initialized".into());

    spawn_local(async {
        if let Err(e) = wasm_frontend::init().await {
            console::error_1(&format!("Failed to initialize frontend: {:?}", e).into());
        }
    });
}

// Export WebAssembly bindings
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct ChronosAuth {
    base_url: String,
    token: Option<String>,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl ChronosAuth {
    #[wasm_bindgen(constructor)]
    pub fn new(base_url: Option<String>) -> ChronosAuth {
        let base_url = base_url.unwrap_or_else(|| "http://localhost:3000".to_string());
        let token = LocalStorage::get("auth_token").ok();

        ChronosAuth {
            base_url,
            token,
        }
    }

    #[wasm_bindgen]
    pub async fn login(&mut self, email: &str, password: &str) -> Result<JsValue, JsValue> {
        let login_request = LoginRequest {
            email: email.to_string(),
            password: password.to_string(),
        };

        let response = Request::post(&format!("{}/api/auth/login", self.base_url))
            .header("Content-Type", "application/json")
            .json(&login_request)
            .map_err(|e| JsValue::from_str(&e.to_string()))?
            .send()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        if response.ok() {
            let login_response: LoginResponse = response
                .json()
                .await
                .map_err(|e| JsValue::from_str(&e.to_string()))?;

            self.token = Some(login_response.access_token.clone());

            // Store tokens in localStorage
            LocalStorage::set("auth_token", &login_response.access_token)
                .map_err(|e| JsValue::from_str(&e.to_string()))?;

            if let Some(refresh_token) = &login_response.refresh_token {
                LocalStorage::set("refresh_token", refresh_token)
                    .map_err(|e| JsValue::from_str(&e.to_string()))?;
            }

            Ok(serde_wasm_bindgen::to_value(&login_response)?)
        } else {
            let error_text = response.text().await.unwrap_or_else(|_| "Login failed".to_string());
            Err(JsValue::from_str(&error_text))
        }
    }

    #[wasm_bindgen]
    pub async fn register(&self, name: &str, email: &str, password: &str) -> Result<JsValue, JsValue> {
        let register_request = RegisterRequest {
            name: name.to_string(),
            email: email.to_string(),
            password: password.to_string(),
        };

        let response = Request::post(&format!("{}/api/auth/register", self.base_url))
            .header("Content-Type", "application/json")
            .json(&register_request)
            .map_err(|e| JsValue::from_str(&e.to_string()))?
            .send()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        if response.ok() {
            let register_response: RegisterResponse = response
                .json()
                .await
                .map_err(|e| JsValue::from_str(&e.to_string()))?;

            Ok(serde_wasm_bindgen::to_value(&register_response)?)
        } else {
            let error_text = response.text().await.unwrap_or_else(|_| "Registration failed".to_string());
            Err(JsValue::from_str(&error_text))
        }
    }

    #[wasm_bindgen]
    pub async fn get_profile(&self) -> Result<JsValue, JsValue> {
        let token = self.token.as_ref().ok_or_else(|| JsValue::from_str("Not authenticated"))?;

        let response = Request::get(&format!("{}/api/auth/profile", self.base_url))
            .header("Authorization", &format!("Bearer {}", token))
            .send()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        if response.ok() {
            let profile: ProfileResponse = response
                .json()
                .await
                .map_err(|e| JsValue::from_str(&e.to_string()))?;

            Ok(serde_wasm_bindgen::to_value(&profile)?)
        } else {
            let error_text = response.text().await.unwrap_or_else(|_| "Failed to get profile".to_string());
            Err(JsValue::from_str(&error_text))
        }
    }

    #[wasm_bindgen]
    pub async fn update_profile(&self, name: Option<String>, email: Option<String>, current_password: Option<String>) -> Result<JsValue, JsValue> {
        let token = self.token.as_ref().ok_or_else(|| JsValue::from_str("Not authenticated"))?;

        let update_request = ProfileUpdateRequest {
            name,
            email,
            current_password,
        };

        let response = Request::put(&format!("{}/api/auth/profile", self.base_url))
            .header("Authorization", &format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(&update_request)
            .map_err(|e| JsValue::from_str(&e.to_string()))?
            .send()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        if response.ok() {
            let profile: ProfileResponse = response
                .json()
                .await
                .map_err(|e| JsValue::from_str(&e.to_string()))?;

            Ok(serde_wasm_bindgen::to_value(&profile)?)
        } else {
            let error_text = response.text().await.unwrap_or_else(|_| "Failed to update profile".to_string());
            Err(JsValue::from_str(&error_text))
        }
    }

    #[wasm_bindgen]
    pub async fn change_password(&self, current_password: &str, new_password: &str) -> Result<JsValue, JsValue> {
        let token = self.token.as_ref().ok_or_else(|| JsValue::from_str("Not authenticated"))?;

        let change_request = ChangePasswordRequest {
            current_password: current_password.to_string(),
            new_password: new_password.to_string(),
        };

        let response = Request::post(&format!("{}/api/auth/change-password", self.base_url))
            .header("Authorization", &format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(&change_request)
            .map_err(|e| JsValue::from_str(&e.to_string()))?
            .send()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        if response.ok() {
            let change_response: ChangePasswordResponse = response
                .json()
                .await
                .map_err(|e| JsValue::from_str(&e.to_string()))?;

            Ok(serde_wasm_bindgen::to_value(&change_response)?)
        } else {
            let error_text = response.text().await.unwrap_or_else(|_| "Failed to change password".to_string());
            Err(JsValue::from_str(&error_text))
        }
    }

    #[wasm_bindgen]
    pub async fn forgot_password(&self, email: &str) -> Result<JsValue, JsValue> {
        let forgot_request = ForgotPasswordRequest {
            email: email.to_string(),
        };

        let response = Request::post(&format!("{}/api/auth/forgot-password", self.base_url))
            .header("Content-Type", "application/json")
            .json(&forgot_request)
            .map_err(|e| JsValue::from_str(&e.to_string()))?
            .send()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        if response.ok() {
            let forgot_response: ForgotPasswordResponse = response
                .json()
                .await
                .map_err(|e| JsValue::from_str(&e.to_string()))?;

            Ok(serde_wasm_bindgen::to_value(&forgot_response)?)
        } else {
            let error_text = response.text().await.unwrap_or_else(|_| "Failed to send password reset email".to_string());
            Err(JsValue::from_str(&error_text))
        }
    }

    #[wasm_bindgen]
    pub async fn reset_password(&self, token: &str, new_password: &str) -> Result<JsValue, JsValue> {
        let reset_request = ResetPasswordRequest {
            token: token.to_string(),
            new_password: new_password.to_string(),
        };

        let response = Request::post(&format!("{}/api/auth/reset-password", self.base_url))
            .header("Content-Type", "application/json")
            .json(&reset_request)
            .map_err(|e| JsValue::from_str(&e.to_string()))?
            .send()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        if response.ok() {
            let reset_response: ResetPasswordResponse = response
                .json()
                .await
                .map_err(|e| JsValue::from_str(&e.to_string()))?;

            Ok(serde_wasm_bindgen::to_value(&reset_response)?)
        } else {
            let error_text = response.text().await.unwrap_or_else(|_| "Failed to reset password".to_string());
            Err(JsValue::from_str(&error_text))
        }
    }

    #[wasm_bindgen]
    pub async fn logout(&mut self, logout_all_devices: Option<bool>) -> Result<JsValue, JsValue> {
        let token = self.token.as_ref().ok_or_else(|| JsValue::from_str("Not authenticated"))?;
        let refresh_token: Option<String> = LocalStorage::get("refresh_token").ok();

        let logout_request = LogoutRequest {
            refresh_token,
            logout_all_devices,
        };

        let response = Request::post(&format!("{}/api/auth/logout", self.base_url))
            .header("Authorization", &format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(&logout_request)
            .map_err(|e| JsValue::from_str(&e.to_string()))?
            .send()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        // Clear local storage regardless of response
        LocalStorage::delete("auth_token");
        LocalStorage::delete("refresh_token");
        self.token = None;

        if response.ok() {
            let logout_response: LogoutResponse = response
                .json()
                .await
                .map_err(|e| JsValue::from_str(&e.to_string()))?;

            Ok(serde_wasm_bindgen::to_value(&logout_response)?)
        } else {
            // Still return success for logout even if server request failed
            let logout_response = LogoutResponse {
                message: "Logged out locally".to_string(),
                logged_out_devices: None,
            };
            Ok(serde_wasm_bindgen::to_value(&logout_response)?)
        }
    }

    #[wasm_bindgen]
    pub async fn refresh_token(&mut self) -> Result<JsValue, JsValue> {
        let refresh_token: String = LocalStorage::get("refresh_token")
            .map_err(|_| JsValue::from_str("No refresh token found"))?;

        let refresh_request = RefreshTokenRequest {
            refresh_token: refresh_token.clone(),
        };

        let response = Request::post(&format!("{}/api/auth/refresh", self.base_url))
            .header("Content-Type", "application/json")
            .json(&refresh_request)
            .map_err(|e| JsValue::from_str(&e.to_string()))?
            .send()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        if response.ok() {
            let refresh_response: RefreshTokenResponse = response
                .json()
                .await
                .map_err(|e| JsValue::from_str(&e.to_string()))?;

            self.token = Some(refresh_response.access_token.clone());

            // Update tokens in localStorage
            LocalStorage::set("auth_token", &refresh_response.access_token)
                .map_err(|e| JsValue::from_str(&e.to_string()))?;

            if let Some(new_refresh_token) = &refresh_response.refresh_token {
                LocalStorage::set("refresh_token", new_refresh_token)
                    .map_err(|e| JsValue::from_str(&e.to_string()))?;
            }

            Ok(serde_wasm_bindgen::to_value(&refresh_response)?)
        } else {
            let error_text = response.text().await.unwrap_or_else(|_| "Token refresh failed".to_string());
            Err(JsValue::from_str(&error_text))
        }
    }

    #[wasm_bindgen]
    pub fn is_authenticated(&self) -> bool {
        self.token.is_some()
    }

    #[wasm_bindgen]
    pub fn get_token(&self) -> Option<String> {
        self.token.clone()
    }
}

// Request/Response types for WebAssembly
#[derive(Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub token_type: String,
    pub expires_in: u32,
    pub user: UserInfo,
}

#[derive(Serialize, Deserialize)]
pub struct UserInfo {
    pub id: Uuid,
    pub name: String,
    pub email: String,
}

#[derive(Serialize, Deserialize)]
pub struct RegisterRequest {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct RegisterResponse {
    pub message: String,
    pub user: UserInfo,
}

#[derive(Serialize, Deserialize)]
pub struct ProfileResponse {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize, Deserialize)]
pub struct ProfileUpdateRequest {
    pub name: Option<String>,
    pub email: Option<String>,
    pub current_password: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Serialize, Deserialize)]
pub struct ChangePasswordResponse {
    pub message: String,
}

#[derive(Serialize, Deserialize)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

#[derive(Serialize, Deserialize)]
pub struct ForgotPasswordResponse {
    pub message: String,
}

#[derive(Serialize, Deserialize)]
pub struct ResetPasswordRequest {
    pub token: String,
    pub new_password: String,
}

#[derive(Serialize, Deserialize)]
pub struct ResetPasswordResponse {
    pub message: String,
}

#[derive(Serialize, Deserialize)]
pub struct LogoutRequest {
    pub refresh_token: Option<String>,
    pub logout_all_devices: Option<bool>,
}

#[derive(Serialize, Deserialize)]
pub struct LogoutResponse {
    pub message: String,
    pub logged_out_devices: Option<u32>,
}

#[derive(Serialize, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[derive(Serialize, Deserialize)]
pub struct RefreshTokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub token_type: String,
    pub expires_in: u32,
    pub refresh_expires_in: Option<u32>,
}

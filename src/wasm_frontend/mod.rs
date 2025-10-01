use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::{console, window, Document, Element, HtmlElement, HtmlInputElement, HtmlButtonElement};
use gloo_storage::{LocalStorage, Storage};
use wasm_bindgen::JsCast;

pub mod ui;

pub async fn init() -> Result<(), JsValue> {
    console::log_1(&"Initializing Chronos WebAssembly frontend...".into());

    // Get the document
    let document = window()
        .ok_or("No window found")?
        .document()
        .ok_or("No document found")?;

    // Create the main application container
    create_app_structure(&document)?;

    // Initialize event handlers
    setup_event_handlers(&document)?;

    // Check if user is already authenticated
    check_authentication_status(&document);

    console::log_1(&"Frontend initialized successfully".into());
    Ok(())
}

fn create_app_structure(document: &Document) -> Result<(), JsValue> {
    let body = document.body().ok_or("No body found")?;

    // Clear existing content
    body.set_inner_html("");

    // Create main app container
    let app_container = document.create_element("div")?;
    app_container.set_id("chronos-app");
    app_container.set_class_name("app-container");

    // Create header
    let header = create_header(document)?;
    app_container.append_child(&header)?;

    // Create main content area
    let main_content = document.create_element("main")?;
    main_content.set_id("main-content");
    main_content.set_class_name("main-content");

    // Initially show login form
    show_login_view(document, &main_content)?;

    app_container.append_child(&main_content)?;
    body.append_child(&app_container)?;

    Ok(())
}

fn create_header(document: &Document) -> Result<Element, JsValue> {
    let header = document.create_element("header")?;
    header.set_class_name("app-header");

    let title = document.create_element("h1")?;
    title.set_text_content(Some("Chronos Authentication System"));
    header.append_child(&title)?;

    let nav = document.create_element("nav")?;
    nav.set_id("main-nav");
    nav.set_class_name("main-nav hidden");

    // Navigation buttons (hidden initially, shown when authenticated)
    let nav_buttons = [
        ("profile-btn", "Profile"),
        ("change-password-btn", "Change Password"),
        ("logout-btn", "Logout"),
    ];

    for (id, text) in nav_buttons.iter() {
        let button = document.create_element("button")?;
        button.set_id(id);
        button.set_class_name("nav-button");
        button.set_text_content(Some(text));
        nav.append_child(&button)?;
    }

    header.append_child(&nav)?;

    Ok(header)
}

fn show_login_view(document: &Document, container: &Element) -> Result<(), JsValue> {
    container.set_inner_html("");

    let login_container = document.create_element("div")?;
    login_container.set_class_name("auth-container");

    // Tab navigation
    let tab_nav = document.create_element("div")?;
    tab_nav.set_class_name("tab-nav");

    let login_tab = document.create_element("button")?;
    login_tab.set_id("login-tab");
    login_tab.set_class_name("tab-button active");
    login_tab.set_text_content(Some("Login"));

    let register_tab = document.create_element("button")?;
    register_tab.set_id("register-tab");
    register_tab.set_class_name("tab-button");
    register_tab.set_text_content(Some("Register"));

    let forgot_tab = document.create_element("button")?;
    forgot_tab.set_id("forgot-tab");
    forgot_tab.set_class_name("tab-button");
    forgot_tab.set_text_content(Some("Forgot Password"));

    tab_nav.append_child(&login_tab)?;
    tab_nav.append_child(&register_tab)?;
    tab_nav.append_child(&forgot_tab)?;

    // Forms container
    let forms_container = document.create_element("div")?;
    forms_container.set_id("forms-container");
    forms_container.set_class_name("forms-container");

    // Login form
    let login_form = create_login_form(document)?;
    forms_container.append_child(&login_form)?;

    // Register form (hidden initially)
    let register_form = create_register_form(document)?;
    forms_container.append_child(&register_form)?;

    // Forgot password form (hidden initially)
    let forgot_form = create_forgot_password_form(document)?;
    forms_container.append_child(&forgot_form)?;

    login_container.append_child(&tab_nav)?;
    login_container.append_child(&forms_container)?;

    container.append_child(&login_container)?;

    Ok(())
}

fn create_login_form(document: &Document) -> Result<Element, JsValue> {
    let form = document.create_element("div")?;
    form.set_id("login-form");
    form.set_class_name("auth-form active");

    let title = document.create_element("h2")?;
    title.set_text_content(Some("Login"));
    form.append_child(&title)?;

    // Email input
    let email_group = create_input_group(document, "login-email", "email", "Email", true)?;
    form.append_child(&email_group)?;

    // Password input
    let password_group = create_input_group(document, "login-password", "password", "Password", true)?;
    form.append_child(&password_group)?;

    // Login button
    let button = document.create_element("button")?;
    button.set_id("login-submit");
    button.set_class_name("submit-button");
    button.set_text_content(Some("Login"));
    form.append_child(&button)?;

    // Message area
    let message = document.create_element("div")?;
    message.set_id("login-message");
    message.set_class_name("message");
    form.append_child(&message)?;

    Ok(form)
}

fn create_register_form(document: &Document) -> Result<Element, JsValue> {
    let form = document.create_element("div")?;
    form.set_id("register-form");
    form.set_class_name("auth-form");

    let title = document.create_element("h2")?;
    title.set_text_content(Some("Register"));
    form.append_child(&title)?;

    // Name input
    let name_group = create_input_group(document, "register-name", "text", "Full Name", true)?;
    form.append_child(&name_group)?;

    // Email input
    let email_group = create_input_group(document, "register-email", "email", "Email", true)?;
    form.append_child(&email_group)?;

    // Password input
    let password_group = create_input_group(document, "register-password", "password", "Password", true)?;
    form.append_child(&password_group)?;

    // Register button
    let button = document.create_element("button")?;
    button.set_id("register-submit");
    button.set_class_name("submit-button");
    button.set_text_content(Some("Register"));
    form.append_child(&button)?;

    // Message area
    let message = document.create_element("div")?;
    message.set_id("register-message");
    message.set_class_name("message");
    form.append_child(&message)?;

    Ok(form)
}

fn create_forgot_password_form(document: &Document) -> Result<Element, JsValue> {
    let form = document.create_element("div")?;
    form.set_id("forgot-form");
    form.set_class_name("auth-form");

    let title = document.create_element("h2")?;
    title.set_text_content(Some("Reset Password"));
    form.append_child(&title)?;

    // Email input
    let email_group = create_input_group(document, "forgot-email", "email", "Email", true)?;
    form.append_child(&email_group)?;

    // Submit button
    let button = document.create_element("button")?;
    button.set_id("forgot-submit");
    button.set_class_name("submit-button");
    button.set_text_content(Some("Send Reset Email"));
    form.append_child(&button)?;

    // Message area
    let message = document.create_element("div")?;
    message.set_id("forgot-message");
    message.set_class_name("message");
    form.append_child(&message)?;

    Ok(form)
}

fn create_input_group(document: &Document, id: &str, input_type: &str, label: &str, required: bool) -> Result<Element, JsValue> {
    let group = document.create_element("div")?;
    group.set_class_name("input-group");

    let label_elem = document.create_element("label")?;
    label_elem.set_attribute("for", id)?;
    label_elem.set_text_content(Some(label));

    let input = document.create_element("input")?;
    input.set_id(id);
    input.set_attribute("type", input_type)?;
    input.set_attribute("name", id)?;
    if required {
        input.set_attribute("required", "")?;
    }

    group.append_child(&label_elem)?;
    group.append_child(&input)?;

    Ok(group)
}

fn setup_event_handlers(document: &Document) -> Result<(), JsValue> {
    // Tab switching
    setup_tab_handlers(document)?;

    // Form submissions
    setup_form_handlers(document)?;

    // Navigation handlers
    setup_nav_handlers(document)?;

    Ok(())
}

fn setup_tab_handlers(document: &Document) -> Result<(), JsValue> {
    let tabs = ["login-tab", "register-tab", "forgot-tab"];
    let forms = ["login-form", "register-form", "forgot-form"];

    for (i, tab_id) in tabs.iter().enumerate() {
        let tab = document.get_element_by_id(tab_id).ok_or("Tab not found")?;
        let tab = tab.dyn_into::<HtmlElement>()?;

        let forms_clone = forms.clone();
        let tab_id_clone = tab_id.to_string();

        let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
            let window = web_sys::window().unwrap();
            let document = window.document().unwrap();

            // Remove active class from all tabs
            for t in &tabs {
                if let Some(elem) = document.get_element_by_id(t) {
                    elem.set_class_name("tab-button");
                }
            }

            // Add active class to clicked tab
            if let Some(elem) = document.get_element_by_id(&tab_id_clone) {
                elem.set_class_name("tab-button active");
            }

            // Hide all forms
            for f in &forms_clone {
                if let Some(elem) = document.get_element_by_id(f) {
                    elem.set_class_name("auth-form");
                }
            }

            // Show corresponding form
            if let Some(elem) = document.get_element_by_id(forms_clone[i]) {
                elem.set_class_name("auth-form active");
            }
        }) as Box<dyn FnMut(_)>);

        tab.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    Ok(())
}

fn setup_form_handlers(document: &Document) -> Result<(), JsValue> {
    // Login form handler
    if let Some(login_btn) = document.get_element_by_id("login-submit") {
        let login_btn = login_btn.dyn_into::<HtmlElement>()?;
        let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
            spawn_local(async {
                handle_login().await;
            });
        }) as Box<dyn FnMut(_)>);

        login_btn.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    // Register form handler
    if let Some(register_btn) = document.get_element_by_id("register-submit") {
        let register_btn = register_btn.dyn_into::<HtmlElement>()?;
        let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
            spawn_local(async {
                handle_register().await;
            });
        }) as Box<dyn FnMut(_)>);

        register_btn.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    // Forgot password handler
    if let Some(forgot_btn) = document.get_element_by_id("forgot-submit") {
        let forgot_btn = forgot_btn.dyn_into::<HtmlElement>()?;
        let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
            spawn_local(async {
                handle_forgot_password().await;
            });
        }) as Box<dyn FnMut(_)>);

        forgot_btn.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    Ok(())
}

fn setup_nav_handlers(document: &Document) -> Result<(), JsValue> {
    // Profile button
    if let Some(profile_btn) = document.get_element_by_id("profile-btn") {
        let profile_btn = profile_btn.dyn_into::<HtmlElement>()?;
        let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
            spawn_local(async {
                show_profile_view().await;
            });
        }) as Box<dyn FnMut(_)>);

        profile_btn.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    // Change password button
    if let Some(change_pwd_btn) = document.get_element_by_id("change-password-btn") {
        let change_pwd_btn = change_pwd_btn.dyn_into::<HtmlElement>()?;
        let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
            spawn_local(async {
                show_change_password_view().await;
            });
        }) as Box<dyn FnMut(_)>);

        change_pwd_btn.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    // Logout button
    if let Some(logout_btn) = document.get_element_by_id("logout-btn") {
        let logout_btn = logout_btn.dyn_into::<HtmlElement>()?;
        let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
            spawn_local(async {
                handle_logout().await;
            });
        }) as Box<dyn FnMut(_)>);

        logout_btn.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    Ok(())
}

async fn handle_login() {
    use crate::ChronosAuth;

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();

    let email_input = document.get_element_by_id("login-email")
        .unwrap().dyn_into::<HtmlInputElement>().unwrap();
    let password_input = document.get_element_by_id("login-password")
        .unwrap().dyn_into::<HtmlInputElement>().unwrap();
    let message_div = document.get_element_by_id("login-message").unwrap();

    let email = email_input.value();
    let password = password_input.value();

    if email.is_empty() || password.is_empty() {
        message_div.set_text_content(Some("Please fill in all fields"));
        message_div.set_class_name("message error");
        return;
    }

    let mut auth = ChronosAuth::new(None);

    match auth.login(&email, &password).await {
        Ok(_) => {
            message_div.set_text_content(Some("Login successful!"));
            message_div.set_class_name("message success");

            // Show authenticated view
            show_authenticated_view(&document).await;
        },
        Err(e) => {
            let error_msg = format!("Login failed: {}", e.as_string().unwrap_or_else(|| "Unknown error".to_string()));
            message_div.set_text_content(Some(&error_msg));
            message_div.set_class_name("message error");
        }
    }
}

async fn handle_register() {
    use crate::ChronosAuth;

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();

    let name_input = document.get_element_by_id("register-name")
        .unwrap().dyn_into::<HtmlInputElement>().unwrap();
    let email_input = document.get_element_by_id("register-email")
        .unwrap().dyn_into::<HtmlInputElement>().unwrap();
    let password_input = document.get_element_by_id("register-password")
        .unwrap().dyn_into::<HtmlInputElement>().unwrap();
    let message_div = document.get_element_by_id("register-message").unwrap();

    let name = name_input.value();
    let email = email_input.value();
    let password = password_input.value();

    if name.is_empty() || email.is_empty() || password.is_empty() {
        message_div.set_text_content(Some("Please fill in all fields"));
        message_div.set_class_name("message error");
        return;
    }

    let auth = ChronosAuth::new(None);

    match auth.register(&name, &email, &password).await {
        Ok(_) => {
            message_div.set_text_content(Some("Registration successful! You can now login."));
            message_div.set_class_name("message success");

            // Switch to login tab
            document.get_element_by_id("login-tab").unwrap().dyn_into::<HtmlElement>().unwrap().click();
        },
        Err(e) => {
            let error_msg = format!("Registration failed: {}", e.as_string().unwrap_or_else(|| "Unknown error".to_string()));
            message_div.set_text_content(Some(&error_msg));
            message_div.set_class_name("message error");
        }
    }
}

async fn handle_forgot_password() {
    use crate::ChronosAuth;

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();

    let email_input = document.get_element_by_id("forgot-email")
        .unwrap().dyn_into::<HtmlInputElement>().unwrap();
    let message_div = document.get_element_by_id("forgot-message").unwrap();

    let email = email_input.value();

    if email.is_empty() {
        message_div.set_text_content(Some("Please enter your email address"));
        message_div.set_class_name("message error");
        return;
    }

    let auth = ChronosAuth::new(None);

    match auth.forgot_password(&email).await {
        Ok(_) => {
            message_div.set_text_content(Some("Password reset email sent! Check your inbox."));
            message_div.set_class_name("message success");
        },
        Err(e) => {
            let error_msg = format!("Failed to send reset email: {}", e.as_string().unwrap_or_else(|| "Unknown error".to_string()));
            message_div.set_text_content(Some(&error_msg));
            message_div.set_class_name("message error");
        }
    }
}

async fn show_authenticated_view(document: &Document) {
    let _main_content = document.get_element_by_id("main-content").unwrap();
    let nav = document.get_element_by_id("main-nav").unwrap();

    // Show navigation
    nav.set_class_name("main-nav");

    // Show profile by default
    show_profile_view().await;
}

async fn show_profile_view() {
    use crate::ChronosAuth;

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let main_content = document.get_element_by_id("main-content").unwrap();

    main_content.set_inner_html("");

    let profile_container = document.create_element("div").unwrap();
    profile_container.set_class_name("profile-container");

    let title = document.create_element("h2").unwrap();
    title.set_text_content(Some("User Profile"));
    profile_container.append_child(&title).unwrap();

    let auth = ChronosAuth::new(None);

    match auth.get_profile().await {
        Ok(profile_data) => {
            // Create profile display (simplified - in reality you'd parse the JSON)
            let profile_info = document.create_element("div").unwrap();
            profile_info.set_class_name("profile-info");
            profile_info.set_inner_html(&format!("<p>Profile loaded successfully</p><pre>{:?}</pre>", profile_data));
            profile_container.append_child(&profile_info).unwrap();
        },
        Err(e) => {
            let error_div = document.create_element("div").unwrap();
            error_div.set_class_name("message error");
            error_div.set_text_content(Some(&format!("Failed to load profile: {}",
                e.as_string().unwrap_or_else(|| "Unknown error".to_string()))));
            profile_container.append_child(&error_div).unwrap();
        }
    }

    main_content.append_child(&profile_container).unwrap();
}

async fn show_change_password_view() {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let main_content = document.get_element_by_id("main-content").unwrap();

    main_content.set_inner_html("");

    let change_pwd_container = document.create_element("div").unwrap();
    change_pwd_container.set_class_name("change-password-container");

    let title = document.create_element("h2").unwrap();
    title.set_text_content(Some("Change Password"));
    change_pwd_container.append_child(&title).unwrap();

    // Current password input
    let current_pwd_group = create_input_group(&document, "current-password", "password", "Current Password", true).unwrap();
    change_pwd_container.append_child(&current_pwd_group).unwrap();

    // New password input
    let new_pwd_group = create_input_group(&document, "new-password", "password", "New Password", true).unwrap();
    change_pwd_container.append_child(&new_pwd_group).unwrap();

    // Change password button
    let change_btn = document.create_element("button").unwrap();
    change_btn.set_id("change-password-submit");
    change_btn.set_class_name("submit-button");
    change_btn.set_text_content(Some("Change Password"));
    change_pwd_container.append_child(&change_btn).unwrap();

    // Message area
    let message = document.create_element("div").unwrap();
    message.set_id("change-password-message");
    message.set_class_name("message");
    change_pwd_container.append_child(&message).unwrap();

    // Add event handler
    let change_btn = change_btn.dyn_into::<HtmlElement>().unwrap();
    let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
        spawn_local(async {
            handle_change_password().await;
        });
    }) as Box<dyn FnMut(_)>);

    change_btn.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref()).unwrap();
    closure.forget();

    main_content.append_child(&change_pwd_container).unwrap();
}

async fn handle_change_password() {
    use crate::ChronosAuth;

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();

    let current_pwd_input = document.get_element_by_id("current-password")
        .unwrap().dyn_into::<HtmlInputElement>().unwrap();
    let new_pwd_input = document.get_element_by_id("new-password")
        .unwrap().dyn_into::<HtmlInputElement>().unwrap();
    let message_div = document.get_element_by_id("change-password-message").unwrap();

    let current_password = current_pwd_input.value();
    let new_password = new_pwd_input.value();

    if current_password.is_empty() || new_password.is_empty() {
        message_div.set_text_content(Some("Please fill in all fields"));
        message_div.set_class_name("message error");
        return;
    }

    let auth = ChronosAuth::new(None);

    match auth.change_password(&current_password, &new_password).await {
        Ok(_) => {
            message_div.set_text_content(Some("Password changed successfully!"));
            message_div.set_class_name("message success");

            // Clear the form
            current_pwd_input.set_value("");
            new_pwd_input.set_value("");
        },
        Err(e) => {
            let error_msg = format!("Failed to change password: {}", e.as_string().unwrap_or_else(|| "Unknown error".to_string()));
            message_div.set_text_content(Some(&error_msg));
            message_div.set_class_name("message error");
        }
    }
}

async fn handle_logout() {
    use crate::ChronosAuth;

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();

    let mut auth = ChronosAuth::new(None);

    match auth.logout(Some(false)).await {
        Ok(_) => {
            // Hide navigation
            let nav = document.get_element_by_id("main-nav").unwrap();
            nav.set_class_name("main-nav hidden");

            // Show login view again
            let main_content = document.get_element_by_id("main-content").unwrap();
            show_login_view(&document, &main_content).unwrap();

            // Re-setup form handlers for the new forms
            setup_form_handlers(&document).unwrap();
        },
        Err(e) => {
            console::error_1(&format!("Logout failed: {}", e.as_string().unwrap_or_else(|| "Unknown error".to_string())).into());
        }
    }
}

fn check_authentication_status(_document: &Document) {
    // Check if user has a valid token in localStorage
    if let Ok(token) = LocalStorage::get::<String>("auth_token") {
        if !token.is_empty() {
            // User appears to be authenticated, show authenticated view
            spawn_local(async move {
                let document = web_sys::window().unwrap().document().unwrap();
                show_authenticated_view(&document).await;
            });
        }
    }
}
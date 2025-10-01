// UI utilities and components for the WebAssembly frontend

use web_sys::{Document, Element, HtmlElement, NodeList};
use wasm_bindgen::{JsValue, JsCast};

pub fn create_loading_spinner(document: &Document) -> Result<Element, JsValue> {
    let spinner = document.create_element("div")?;
    spinner.set_class_name("loading-spinner");

    let inner = document.create_element("div")?;
    inner.set_class_name("spinner");

    spinner.append_child(&inner)?;
    Ok(spinner)
}

pub fn create_notification(document: &Document, message: &str, notification_type: &str) -> Result<Element, JsValue> {
    let notification = document.create_element("div")?;
    notification.set_class_name(&format!("notification {}", notification_type));
    notification.set_text_content(Some(message));

    // Auto-remove after 5 seconds
    let notification_clone = notification.clone();
    let closure = wasm_bindgen::closure::Closure::once(move || {
        if let Some(parent) = notification_clone.parent_node() {
            let _ = parent.remove_child(&notification_clone);
        }
    });
    let timeout = web_sys::window().unwrap().set_timeout_with_callback_and_timeout_and_arguments_0(
        closure.as_ref().unchecked_ref(),
        5000
    );
    closure.forget();

    Ok(notification)
}

pub fn show_message(element_id: &str, message: &str, message_type: &str) {
    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            if let Some(element) = document.get_element_by_id(element_id) {
                element.set_text_content(Some(message));
                element.set_class_name(&format!("message {}", message_type));
            }
        }
    }
}

pub fn clear_form(form_id: &str) {
    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            if let Some(form) = document.get_element_by_id(form_id) {
                let inputs = form.query_selector_all("input").unwrap();
                for i in 0..inputs.length() {
                    if let Some(input) = inputs.get(i) {
                        if let Ok(input_element) = input.dyn_into::<web_sys::HtmlInputElement>() {
                            input_element.set_value("");
                        }
                    }
                }
            }
        }
    }
}

pub fn toggle_element_visibility(element_id: &str, visible: bool) {
    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            if let Some(element) = document.get_element_by_id(element_id) {
                if visible {
                    element.set_class_name(&element.class_name().replace(" hidden", ""));
                } else {
                    if !element.class_name().contains("hidden") {
                        element.set_class_name(&format!("{} hidden", element.class_name()));
                    }
                }
            }
        }
    }
}

pub fn set_button_loading(button_id: &str, loading: bool) {
    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            if let Some(button) = document.get_element_by_id(button_id) {
                if let Ok(button_element) = button.dyn_into::<web_sys::HtmlButtonElement>() {
                    button_element.set_disabled(loading);
                    if loading {
                        button_element.set_text_content(Some("Loading..."));
                    }
                }
            }
        }
    }
}
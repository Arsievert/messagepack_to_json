use eframe::egui;
use serde_json;
use rmp_serde;
use base64::{engine::general_purpose, Engine};
use std::sync::{Arc, Mutex};
use clipboard::{ClipboardProvider, ClipboardContext};
use hex;

#[derive(Default)]
struct MessagePackJsonConverterApp {
    json_input: String,
    messagepack_output: String,
    messagepack_input: String,
    json_output: String,
    error_message: Arc<Mutex<String>>,
}

impl eframe::App for MessagePackJsonConverterApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {

            ui.vertical_centered(|ui| {
                ui.heading("JSON <-> MessagePack Converter");
            });

            ui.separator();

            // Add the "Clear All" button at the top
            ui.vertical_centered(|ui| {
                if ui.button("Clear All").clicked() {
                    self.json_input.clear();
                    self.messagepack_output.clear();
                    self.messagepack_input.clear();
                    self.json_output.clear();
                    *self.error_message.lock().unwrap() = String::new();
                }
            });

            ui.separator();

            ui.horizontal(|ui| {
                // JSON to MessagePack Conversion Section
                ui.vertical(|ui| {
                    ui.heading("JSON to MessagePack");

                    ui.label("JSON Input:");
                    ui.push_id("json_input", |ui| {
                        egui::ScrollArea::vertical()
                            .min_scrolled_height(300.0)
                            .max_height(300.0)
                            .show(ui, |ui| {
                                ui.add(egui::TextEdit::multiline(&mut self.json_input)
                                    .frame(true)
                                    .desired_width(400.0)
                                    .desired_rows(12)
                                    .min_size(egui::vec2(400.0, 300.0)));
                            });
                    });

                    if ui.button("Convert to MessagePack").clicked() {
                        match json_to_messagepack(&self.json_input) {
                            Ok(mp) => {
                                self.messagepack_output = mp;
                                *self.error_message.lock().unwrap() = String::new();
                            }
                            Err(e) => {
                                *self.error_message.lock().unwrap() = e;
                            }
                        }
                    }

                    ui.label("MessagePack Output (Base64):");
                    ui.push_id("messagepack_output", |ui| {
                        egui::ScrollArea::vertical()
                            .min_scrolled_height(300.0)
                            .max_height(300.0)
                            .show(ui, |ui| {
                                ui.add(egui::TextEdit::multiline(&mut self.messagepack_output)
                                    .frame(true)
                                    .desired_width(400.0)
                                    .desired_rows(12)
                                    .min_size(egui::vec2(400.0, 300.0))
                                    .cursor_at_end(false));
                            });
                    });

                    if ui.button("Copy MessagePack").clicked() {
                        copy_to_clipboard(&self.messagepack_output);
                    }
                });

                // MessagePack to JSON Conversion Section
                ui.vertical(|ui| {
                    ui.heading("MessagePack to JSON");

                    ui.label("MessagePack Input (Base64 or Hex):");
                    ui.push_id("messagepack_input", |ui| {
                        egui::ScrollArea::vertical()
                            .min_scrolled_height(300.0)
                            .max_height(300.0)
                            .show(ui, |ui| {
                                ui.add(egui::TextEdit::multiline(&mut self.messagepack_input)
                                    .frame(true)
                                    .desired_width(400.0)
                                    .desired_rows(12)
                                    .min_size(egui::vec2(400.0, 300.0)));
                            });
                    });

                    if ui.button("Convert to JSON").clicked() {
                        match messagepack_to_json(&self.messagepack_input) {
                            Ok(json) => {
                                self.json_output = json;
                                *self.error_message.lock().unwrap() = String::new();
                            }
                            Err(e) => {
                                *self.error_message.lock().unwrap() = e;
                            }
                        }
                    }

                    ui.label("JSON Output:");
                    ui.push_id("json_output", |ui| {
                        egui::ScrollArea::vertical()
                            .min_scrolled_height(300.0)
                            .max_height(300.0)
                            .show(ui, |ui| {
                                ui.add(egui::TextEdit::multiline(&mut self.json_output)
                                    .frame(true)
                                    .desired_width(400.0)
                                    .desired_rows(12)
                                    .min_size(egui::vec2(400.0, 300.0))
                                    .cursor_at_end(false));
                            });
                    });

                    if ui.button("Copy JSON").clicked() {
                        copy_to_clipboard(&self.json_output);
                    }
                });
            });

            // Error Display Section
            let error_message = self.error_message.lock().unwrap();
            if !error_message.is_empty() {
                ui.label(egui::RichText::new(&*error_message).color(egui::Color32::RED));
            }
        });
    }
}

fn json_to_messagepack(json_str: &str) -> Result<String, String> {
    let json_value: serde_json::Value = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;
    let messagepack = rmp_serde::to_vec(&json_value)
        .map_err(|e| format!("Failed to serialize to MessagePack: {}", e))?;
    Ok(general_purpose::STANDARD.encode(&messagepack))
}

fn messagepack_to_json(encoded_str: &str) -> Result<String, String> {
    let messagepack = if is_hex(encoded_str) {
        hex::decode(encoded_str).map_err(|e| format!("Failed to decode Hex: {}", e))?
    } else {
        general_purpose::STANDARD.decode(encoded_str).map_err(|e| format!("Failed to decode Base64: {}", e))?
    };

    let json_value: serde_json::Value = rmp_serde::from_slice(&messagepack)
        .map_err(|e| format!("Failed to deserialize MessagePack: {}", e))?;
    serde_json::to_string_pretty(&json_value)
        .map_err(|e| format!("Failed to serialize to JSON: {}", e))
}

fn is_hex(s: &str) -> bool {
    s.chars().all(|c| c.is_digit(16))
}

fn copy_to_clipboard(text: &str) {
    let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
    ctx.set_contents(text.to_owned()).unwrap();
}

fn main() {
    let app = MessagePackJsonConverterApp::default();

    let custom_viewport = egui::ViewportBuilder {
        min_inner_size: Some(egui::vec2(850.0, 800.0)),
        ..Default::default()
    };
    let options = eframe::NativeOptions {
        viewport: custom_viewport,
        ..Default::default()
    };

    let _ = eframe::run_native(
        "MessagePack <-> JSON Converter",
        options,
        Box::new(|_cc| Box::new(app)),
    );
}


/* Tests */
#[test]
fn test_json_to_messagepack() {
    let json_data = r#"{"name":"Alice","age":30,"city":"Wonderland"}"#;
    let result = json_to_messagepack(json_data);
    assert!(result.is_ok());

    // Decode the base64 string into raw bytes
    let actual_bytes = general_purpose::STANDARD.decode(result.unwrap()).expect("Failed to decode base64");

    // Convert the bytes into a hex string
    let actual_hex = hex::encode(actual_bytes);

    // Expected hex directly corresponding to MessagePack
    let expected_hex = "83a36167651ea463697479aa576f6e6465726c616e64a46e616d65a5416c696365";

    // Compare the actual hex string with the expected hex string
    assert_eq!(actual_hex, expected_hex);
}

#[test]
fn test_messagepack_to_json() {
    let msgpack_data = "g6NhZ2UepGNpdHmqV29uZGVybGFuZKRuYW1lpUFsaWNl";
    let result = messagepack_to_json(msgpack_data);
    assert!(result.is_ok());

    let expected = r#"{
  "age": 30,
  "city": "Wonderland",
  "name": "Alice"
}"#;
    assert_eq!(result.unwrap(), expected);
}

#[test]
fn test_invalid_json_to_messagepack() {
    let invalid_json = r#"{"name":"Alice","age":30,"city":Wonderland}"#; // Missing quotes around Wonderland
    let result = json_to_messagepack(invalid_json);
    assert!(result.is_err());
}

#[test]
fn test_invalid_messagepack_to_json() {
    let invalid_msgpack = "invalid_base64_string";
    let result = messagepack_to_json(invalid_msgpack);
    assert!(result.is_err());
}

#[test]
fn test_is_hex() {
    assert!(is_hex("a1b2c3"));
    assert!(!is_hex("g1h2i3")); // Invalid hex
}

#[test]
fn test_json_to_messagepack_and_back() {
    let original_json = r#"{"name":"Alice","age":30,"city":"Wonderland"}"#;

    // Convert JSON to MessagePack
    let messagepack = json_to_messagepack(original_json).expect("Failed to convert JSON to MessagePack");

    // Convert MessagePack back to JSON
    let result_json = messagepack_to_json(&messagepack).expect("Failed to convert MessagePack back to JSON");

    // Parse both original and result JSON strings to ensure they are structurally the same
    let original_json_value: serde_json::Value = serde_json::from_str(original_json).expect("Failed to parse original JSON");
    let result_json_value: serde_json::Value = serde_json::from_str(&result_json).expect("Failed to parse result JSON");

    assert_eq!(original_json_value, result_json_value);
}

#[test]
fn test_complex_json_to_messagepack_and_back() {
    let original_json = r#"{
        "person": {
            "name": "Bob",
            "age": 25,
            "address": {
                "street": "123 Elm Street",
                "city": "Somewhere",
                "zip": "12345"
            }
        },
        "hobbies": ["reading", "gaming", "hiking"],
        "is_student": false
    }"#;

    let messagepack = json_to_messagepack(original_json).expect("Failed to convert JSON to MessagePack");
    let result_json = messagepack_to_json(&messagepack).expect("Failed to convert MessagePack back to JSON");

    let original_json_value: serde_json::Value = serde_json::from_str(original_json).expect("Failed to parse original JSON");
    let result_json_value: serde_json::Value = serde_json::from_str(&result_json).expect("Failed to parse result JSON");

    assert_eq!(original_json_value, result_json_value);
}

#[test]
fn test_empty_json() {
    let empty_json = r#"{}"#; // Empty JSON object
    let messagepack = json_to_messagepack(empty_json).expect("Failed to convert empty JSON");
    let result_json = messagepack_to_json(&messagepack).expect("Failed to convert back to JSON");

    assert_eq!(empty_json, result_json);
}

#[test]
fn test_large_json() {
    let large_json = r#"{"data": ["long_string", 1000]}"#.replace("long_string", &"a".repeat(1000)); // Large JSON string
    let messagepack = json_to_messagepack(&large_json).expect("Failed to convert large JSON");
    let result_json = messagepack_to_json(&messagepack).expect("Failed to convert back to JSON");

    let original_json_value: serde_json::Value = serde_json::from_str(&large_json).expect("Failed to parse large JSON");
    let result_json_value: serde_json::Value = serde_json::from_str(&result_json).expect("Failed to parse result JSON");

    assert_eq!(original_json_value, result_json_value);
}

#[test]
fn test_is_hex_with_valid_and_invalid_input() {
    assert!(is_hex("a1b2c3"));
    assert!(is_hex("0f0f0f"));
    assert!(!is_hex("z1g2h3")); // Invalid hex string
}

#[test]
fn test_messagepack_to_json_with_hex_input() {
    let valid_messagepack_hex = "83a36167651ea463697479aa576f6e6465726c616e64a46e616d65a5416c696365";

    let result = messagepack_to_json(valid_messagepack_hex);
    assert!(result.is_ok(), "Valid hex MessagePack should decode to JSON");

    let expected_json = r#"{"name":"Alice","age":30,"city":"Wonderland"}"#;
    let expected_json_value: serde_json::Value = serde_json::from_str(expected_json).expect("Failed to parse expected JSON");
    let result_json_value: serde_json::Value = serde_json::from_str(&result.unwrap()).expect("Failed to parse result JSON");

    assert_eq!(expected_json_value, result_json_value);
}

#[test]
fn test_messagepack_to_json_and_back() {
    let original_messagepack_hex = "83a36167651ea463697479aa576f6e6465726c616e64a46e616d65a5416c696365";

    // Convert MessagePack hex to JSON
    let json_data = messagepack_to_json(original_messagepack_hex).expect("Failed to convert MessagePack to JSON");

    // Convert JSON back to MessagePack (which will be base64 encoded)
    let messagepack_b64 = json_to_messagepack(&json_data).expect("Failed to convert JSON back to MessagePack");

    // Decode the base64 back into the original hex string
    let new_messagepack_bytes = general_purpose::STANDARD.decode(&messagepack_b64).expect("Failed to decode base64 back to bytes");
    let new_messagepack_hex = hex::encode(new_messagepack_bytes);

    // Compare the original and new MessagePack hex values
    assert_eq!(original_messagepack_hex, new_messagepack_hex);
}

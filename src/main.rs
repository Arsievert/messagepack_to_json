use base64::{engine::general_purpose, Engine};
use clipboard::{ClipboardContext, ClipboardProvider};
use eframe::egui;
use hex;
use rmp_serde;
use serde_json;
use std::io::Cursor;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InputFormat {
    Auto,
    Hex,
    Base64,
}

impl Default for InputFormat {
    fn default() -> Self {
        InputFormat::Auto
    }
}

#[derive(Default)]
struct MessagePackJsonConverterApp {
    json_input: String,
    messagepack_output: String,
    messagepack_input: String,
    json_output: String,
    error_message: Arc<Mutex<String>>,
    input_format: InputFormat,
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
                                ui.add(
                                    egui::TextEdit::multiline(&mut self.json_input)
                                        .frame(true)
                                        .desired_width(400.0)
                                        .desired_rows(12)
                                        .min_size(egui::vec2(400.0, 300.0)),
                                );
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
                                ui.add(
                                    egui::TextEdit::multiline(&mut self.messagepack_output)
                                        .frame(true)
                                        .desired_width(400.0)
                                        .desired_rows(12)
                                        .min_size(egui::vec2(400.0, 300.0))
                                        .cursor_at_end(false),
                                );
                            });
                    });

                    if ui.button("Copy MessagePack").clicked() {
                        copy_to_clipboard(&self.messagepack_output);
                    }
                });

                // MessagePack to JSON Conversion Section
                ui.vertical(|ui| {
                    ui.heading("MessagePack to JSON");

                    ui.label("MessagePack Input:");
                    ui.horizontal(|ui| {
                        ui.label("Format:");
                        ui.radio_value(&mut self.input_format, InputFormat::Auto, "Auto");
                        ui.radio_value(&mut self.input_format, InputFormat::Hex, "Hex");
                        ui.radio_value(&mut self.input_format, InputFormat::Base64, "Base64");
                    });
                    ui.push_id("messagepack_input", |ui| {
                        egui::ScrollArea::vertical()
                            .min_scrolled_height(300.0)
                            .max_height(300.0)
                            .show(ui, |ui| {
                                ui.add(
                                    egui::TextEdit::multiline(&mut self.messagepack_input)
                                        .frame(true)
                                        .desired_width(400.0)
                                        .desired_rows(12)
                                        .min_size(egui::vec2(400.0, 300.0)),
                                );
                            });
                    });

                    if ui.button("Convert to JSON").clicked() {
                        match messagepack_to_json(&self.messagepack_input, self.input_format) {
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
                                ui.add(
                                    egui::TextEdit::multiline(&mut self.json_output)
                                        .frame(true)
                                        .desired_width(400.0)
                                        .desired_rows(12)
                                        .min_size(egui::vec2(400.0, 300.0))
                                        .cursor_at_end(false),
                                );
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
    let json_value: serde_json::Value =
        serde_json::from_str(json_str).map_err(|e| format!("Failed to parse JSON: {}", e))?;
    let messagepack = rmp_serde::to_vec(&json_value)
        .map_err(|e| format!("Failed to serialize to MessagePack: {}", e))?;
    Ok(general_purpose::STANDARD.encode(&messagepack))
}

fn messagepack_to_json(encoded_str: &str, format: InputFormat) -> Result<String, String> {
    let messagepack = match format {
        InputFormat::Hex => {
            let normalized = normalize_hex_input(encoded_str);
            if normalized.is_empty() {
                return Err("Input is empty after normalization".to_string());
            }
            hex::decode(&normalized).map_err(|e| format!("Failed to decode Hex: {}", e))?
        }
        InputFormat::Base64 => general_purpose::STANDARD
            .decode(encoded_str.trim())
            .map_err(|e| format!("Failed to decode Base64: {}", e))?,
        InputFormat::Auto => {
            let normalized = normalize_hex_input(encoded_str);
            if is_hex(&normalized) {
                hex::decode(&normalized).map_err(|e| format!("Failed to decode Hex: {}", e))?
            } else {
                general_purpose::STANDARD
                    .decode(encoded_str.trim())
                    .map_err(|e| format!("Failed to decode Base64: {}", e))?
            }
        }
    };

    let json_value = msgpack_bytes_to_json_value(&messagepack)?;
    serde_json::to_string_pretty(&json_value)
        .map_err(|e| format!("Failed to serialize to JSON: {}", e))
}

/// Decode raw messagepack bytes into a serde_json::Value.
///
/// Uses `rmpv` to first parse into a messagepack-native value tree (which
/// supports integer keys, binary data, etc.), then converts that tree into
/// a JSON-compatible value. Integer map keys are converted to their string
/// representation so they can be used as JSON object keys.
fn msgpack_bytes_to_json_value(bytes: &[u8]) -> Result<serde_json::Value, String> {
    let mut cursor = Cursor::new(bytes);
    let msgpack_value = rmpv::decode::read_value(&mut cursor)
        .map_err(|e| format!("Failed to deserialize MessagePack: {}", e))?;
    rmpv_to_json(msgpack_value)
}

/// Recursively convert an `rmpv::Value` into a `serde_json::Value`.
///
/// - Integer and float map keys are stringified so they work as JSON object keys.
/// - Binary data is hex-encoded into a string.
/// - MessagePack extension types are represented as `{"type": N, "data": "<hex>"}`.
fn rmpv_to_json(val: rmpv::Value) -> Result<serde_json::Value, String> {
    match val {
        rmpv::Value::Nil => Ok(serde_json::Value::Null),
        rmpv::Value::Boolean(b) => Ok(serde_json::Value::Bool(b)),
        rmpv::Value::Integer(i) => {
            if let Some(n) = i.as_i64() {
                Ok(serde_json::Value::Number(n.into()))
            } else if let Some(n) = i.as_u64() {
                Ok(serde_json::Value::Number(n.into()))
            } else {
                Err(format!("Integer value out of range: {}", i))
            }
        }
        rmpv::Value::F32(f) => serde_json::Number::from_f64(f as f64)
            .map(serde_json::Value::Number)
            .ok_or_else(|| format!("Float value not representable in JSON: {}", f)),
        rmpv::Value::F64(f) => serde_json::Number::from_f64(f)
            .map(serde_json::Value::Number)
            .ok_or_else(|| format!("Float value not representable in JSON: {}", f)),
        rmpv::Value::String(s) => match s.into_str() {
            Some(s) => Ok(serde_json::Value::String(s.to_owned())),
            None => Err("MessagePack string is not valid UTF-8".to_string()),
        },
        rmpv::Value::Binary(b) => Ok(serde_json::Value::String(hex::encode(b))),
        rmpv::Value::Array(arr) => {
            let items: Result<Vec<_>, _> = arr.into_iter().map(rmpv_to_json).collect();
            Ok(serde_json::Value::Array(items?))
        }
        rmpv::Value::Map(entries) => {
            let mut map = serde_json::Map::new();
            for (k, v) in entries {
                let key = rmpv_key_to_string(k)?;
                let value = rmpv_to_json(v)?;
                map.insert(key, value);
            }
            Ok(serde_json::Value::Object(map))
        }
        rmpv::Value::Ext(type_id, data) => {
            let mut map = serde_json::Map::new();
            map.insert(
                "type".to_string(),
                serde_json::Value::Number(type_id.into()),
            );
            map.insert(
                "data".to_string(),
                serde_json::Value::String(hex::encode(data)),
            );
            Ok(serde_json::Value::Object(map))
        }
    }
}

/// Convert a messagepack map key into a JSON-compatible string.
fn rmpv_key_to_string(key: rmpv::Value) -> Result<String, String> {
    match key {
        rmpv::Value::String(s) => s
            .into_str()
            .map(|s| s.to_owned())
            .ok_or_else(|| "Map key is not valid UTF-8".to_string()),
        rmpv::Value::Integer(i) => Ok(i.to_string()),
        rmpv::Value::Boolean(b) => Ok(b.to_string()),
        rmpv::Value::F32(f) => Ok(f.to_string()),
        rmpv::Value::F64(f) => Ok(f.to_string()),
        rmpv::Value::Binary(b) => Ok(hex::encode(b)),
        rmpv::Value::Nil => Ok("null".to_string()),
        other => Err(format!("Unsupported map key type: {:?}", other)),
    }
}

/// Normalize a hex input string by stripping a leading `0x`/`0X` prefix
/// and removing all whitespace (spaces, tabs, newlines).
fn normalize_hex_input(s: &str) -> String {
    let trimmed = s.trim();
    let without_prefix = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
        .unwrap_or(trimmed);
    without_prefix
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect()
}

/// Check if a string (already normalized) is valid hex:
/// non-empty, even length, and all characters are hex digits.
fn is_hex(s: &str) -> bool {
    !s.is_empty() && s.len() % 2 == 0 && s.chars().all(|c| c.is_ascii_hexdigit())
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
        Box::new(|_cc| Ok(Box::new(app))),
    );
}

/* Tests */
#[test]
fn test_json_to_messagepack() {
    let json_data = r#"{"name":"Alice","age":30,"city":"Wonderland"}"#;
    let result = json_to_messagepack(json_data);
    assert!(result.is_ok());

    // Decode the base64 string into raw bytes
    let actual_bytes = general_purpose::STANDARD
        .decode(result.unwrap())
        .expect("Failed to decode base64");

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
    let result = messagepack_to_json(msgpack_data, InputFormat::Auto);
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
    let result = messagepack_to_json(invalid_msgpack, InputFormat::Auto);
    assert!(result.is_err());
}

#[test]
fn test_is_hex() {
    assert!(is_hex("a1b2c3"));
    assert!(!is_hex("g1h2i3")); // Invalid hex
    assert!(!is_hex("")); // Empty string
    assert!(!is_hex("abc")); // Odd length
}

#[test]
fn test_json_to_messagepack_and_back() {
    let original_json = r#"{"name":"Alice","age":30,"city":"Wonderland"}"#;

    // Convert JSON to MessagePack
    let messagepack =
        json_to_messagepack(original_json).expect("Failed to convert JSON to MessagePack");

    // Convert MessagePack back to JSON
    let result_json = messagepack_to_json(&messagepack, InputFormat::Auto)
        .expect("Failed to convert MessagePack back to JSON");

    // Parse both original and result JSON strings to ensure they are structurally the same
    let original_json_value: serde_json::Value =
        serde_json::from_str(original_json).expect("Failed to parse original JSON");
    let result_json_value: serde_json::Value =
        serde_json::from_str(&result_json).expect("Failed to parse result JSON");

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

    let messagepack =
        json_to_messagepack(original_json).expect("Failed to convert JSON to MessagePack");
    let result_json = messagepack_to_json(&messagepack, InputFormat::Auto)
        .expect("Failed to convert MessagePack back to JSON");

    let original_json_value: serde_json::Value =
        serde_json::from_str(original_json).expect("Failed to parse original JSON");
    let result_json_value: serde_json::Value =
        serde_json::from_str(&result_json).expect("Failed to parse result JSON");

    assert_eq!(original_json_value, result_json_value);
}

#[test]
fn test_empty_json() {
    let empty_json = r#"{}"#; // Empty JSON object
    let messagepack = json_to_messagepack(empty_json).expect("Failed to convert empty JSON");
    let result_json = messagepack_to_json(&messagepack, InputFormat::Auto)
        .expect("Failed to convert back to JSON");

    assert_eq!(empty_json, result_json);
}

#[test]
fn test_large_json() {
    let large_json = r#"{"data": ["long_string", 1000]}"#.replace("long_string", &"a".repeat(1000)); // Large JSON string
    let messagepack = json_to_messagepack(&large_json).expect("Failed to convert large JSON");
    let result_json = messagepack_to_json(&messagepack, InputFormat::Auto)
        .expect("Failed to convert back to JSON");

    let original_json_value: serde_json::Value =
        serde_json::from_str(&large_json).expect("Failed to parse large JSON");
    let result_json_value: serde_json::Value =
        serde_json::from_str(&result_json).expect("Failed to parse result JSON");

    assert_eq!(original_json_value, result_json_value);
}

#[test]
fn test_is_hex_with_valid_and_invalid_input() {
    assert!(is_hex("a1b2c3"));
    assert!(is_hex("0f0f0f"));
    assert!(!is_hex("z1g2h3")); // Invalid hex string
    assert!(!is_hex("")); // Empty string
    assert!(!is_hex("a")); // Odd length, single char
    assert!(!is_hex("abc")); // Odd length
    assert!(is_hex("AABB")); // Uppercase
    assert!(is_hex("aAbB")); // Mixed case
}

#[test]
fn test_messagepack_to_json_with_hex_input() {
    let valid_messagepack_hex =
        "83a36167651ea463697479aa576f6e6465726c616e64a46e616d65a5416c696365";

    let result = messagepack_to_json(valid_messagepack_hex, InputFormat::Auto);
    assert!(
        result.is_ok(),
        "Valid hex MessagePack should decode to JSON"
    );

    let expected_json = r#"{"name":"Alice","age":30,"city":"Wonderland"}"#;
    let expected_json_value: serde_json::Value =
        serde_json::from_str(expected_json).expect("Failed to parse expected JSON");
    let result_json_value: serde_json::Value =
        serde_json::from_str(&result.unwrap()).expect("Failed to parse result JSON");

    assert_eq!(expected_json_value, result_json_value);
}

#[test]
fn test_messagepack_to_json_and_back() {
    let original_messagepack_hex =
        "83a36167651ea463697479aa576f6e6465726c616e64a46e616d65a5416c696365";

    // Convert MessagePack hex to JSON
    let json_data = messagepack_to_json(original_messagepack_hex, InputFormat::Auto)
        .expect("Failed to convert MessagePack to JSON");

    // Convert JSON back to MessagePack (which will be base64 encoded)
    let messagepack_b64 =
        json_to_messagepack(&json_data).expect("Failed to convert JSON back to MessagePack");

    // Decode the base64 back into the original hex string
    let new_messagepack_bytes = general_purpose::STANDARD
        .decode(&messagepack_b64)
        .expect("Failed to decode base64 back to bytes");
    let new_messagepack_hex = hex::encode(new_messagepack_bytes);

    // Compare the original and new MessagePack hex values
    assert_eq!(original_messagepack_hex, new_messagepack_hex);
}

#[test]
fn test_normalize_hex_input_strips_0x_prefix() {
    assert_eq!(normalize_hex_input("0x83a3"), "83a3");
    assert_eq!(normalize_hex_input("0X83a3"), "83a3");
}

#[test]
fn test_normalize_hex_input_strips_whitespace() {
    assert_eq!(normalize_hex_input("83 a3 61"), "83a361");
    assert_eq!(normalize_hex_input("83\ta3\n61"), "83a361");
    assert_eq!(normalize_hex_input("  83a3  "), "83a3");
}

#[test]
fn test_normalize_hex_input_strips_prefix_and_whitespace() {
    assert_eq!(normalize_hex_input("0x83 a3 61 67 65 1e"), "83a36167651e");
    assert_eq!(normalize_hex_input("  0x 83 a3  "), "83a3");
}

#[test]
fn test_hex_with_0x_prefix_decodes() {
    let hex_input = "0x83a36167651ea463697479aa576f6e6465726c616e64a46e616d65a5416c696365";
    let result = messagepack_to_json(hex_input, InputFormat::Auto);
    assert!(
        result.is_ok(),
        "0x-prefixed hex should decode: {:?}",
        result.err()
    );

    let expected_json = r#"{"name":"Alice","age":30,"city":"Wonderland"}"#;
    let expected: serde_json::Value = serde_json::from_str(expected_json).unwrap();
    let actual: serde_json::Value = serde_json::from_str(&result.unwrap()).unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn test_hex_with_spaces_decodes() {
    let hex_input = "83 a3 61 67 65 1e a4 63 69 74 79 aa 57 6f 6e 64 65 72 6c 61 6e 64 a4 6e 61 6d 65 a5 41 6c 69 63 65";
    let result = messagepack_to_json(hex_input, InputFormat::Auto);
    assert!(
        result.is_ok(),
        "Space-separated hex should decode: {:?}",
        result.err()
    );

    let expected_json = r#"{"name":"Alice","age":30,"city":"Wonderland"}"#;
    let expected: serde_json::Value = serde_json::from_str(expected_json).unwrap();
    let actual: serde_json::Value = serde_json::from_str(&result.unwrap()).unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn test_hex_with_newlines_decodes() {
    let hex_input = "83a36167651ea463697479aa576f6e\n6465726c616e64a46e616d65a5416c696365";
    let result = messagepack_to_json(hex_input, InputFormat::Auto);
    assert!(
        result.is_ok(),
        "Newline-wrapped hex should decode: {:?}",
        result.err()
    );

    let expected_json = r#"{"name":"Alice","age":30,"city":"Wonderland"}"#;
    let expected: serde_json::Value = serde_json::from_str(expected_json).unwrap();
    let actual: serde_json::Value = serde_json::from_str(&result.unwrap()).unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn test_hex_mixed_case_decodes() {
    let hex_input = "83A36167651eA463697479Aa576F6E6465726C616E64A46E616D65A5416C696365";
    let result = messagepack_to_json(hex_input, InputFormat::Auto);
    assert!(
        result.is_ok(),
        "Mixed-case hex should decode: {:?}",
        result.err()
    );

    let expected_json = r#"{"name":"Alice","age":30,"city":"Wonderland"}"#;
    let expected: serde_json::Value = serde_json::from_str(expected_json).unwrap();
    let actual: serde_json::Value = serde_json::from_str(&result.unwrap()).unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn test_hex_with_0x_prefix_and_spaces_decodes() {
    let hex_input = "0x83 a3 61 67 65 1e a4 63 69 74 79 aa 57 6f 6e 64 65 72 6c 61 6e 64 a4 6e 61 6d 65 a5 41 6c 69 63 65";
    let result = messagepack_to_json(hex_input, InputFormat::Auto);
    assert!(
        result.is_ok(),
        "0x-prefixed space-separated hex should decode: {:?}",
        result.err()
    );

    let expected_json = r#"{"name":"Alice","age":30,"city":"Wonderland"}"#;
    let expected: serde_json::Value = serde_json::from_str(expected_json).unwrap();
    let actual: serde_json::Value = serde_json::from_str(&result.unwrap()).unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn test_empty_input_returns_error() {
    let result = messagepack_to_json("", InputFormat::Auto);
    assert!(result.is_err(), "Empty input should return an error");
}

#[test]
fn test_explicit_hex_format() {
    let hex_input = "83a36167651ea463697479aa576f6e6465726c616e64a46e616d65a5416c696365";
    let result = messagepack_to_json(hex_input, InputFormat::Hex);
    assert!(
        result.is_ok(),
        "Explicit Hex format should decode: {:?}",
        result.err()
    );

    let expected_json = r#"{"name":"Alice","age":30,"city":"Wonderland"}"#;
    let expected: serde_json::Value = serde_json::from_str(expected_json).unwrap();
    let actual: serde_json::Value = serde_json::from_str(&result.unwrap()).unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn test_explicit_base64_format() {
    let b64_input = "g6NhZ2UepGNpdHmqV29uZGVybGFuZKRuYW1lpUFsaWNl";
    let result = messagepack_to_json(b64_input, InputFormat::Base64);
    assert!(
        result.is_ok(),
        "Explicit Base64 format should decode: {:?}",
        result.err()
    );

    let expected_json = r#"{"name":"Alice","age":30,"city":"Wonderland"}"#;
    let expected: serde_json::Value = serde_json::from_str(expected_json).unwrap();
    let actual: serde_json::Value = serde_json::from_str(&result.unwrap()).unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn test_explicit_hex_format_with_spaces_and_prefix() {
    let hex_input = "0x83 a3 61 67 65 1e";
    let result = messagepack_to_json(hex_input, InputFormat::Hex);
    // This should at least decode the hex successfully (msgpack may or may not be valid,
    // but the hex decoding step should not fail)
    assert!(
        result.is_ok()
            || !result
                .as_ref()
                .unwrap_err()
                .contains("Failed to decode Hex"),
        "Hex decoding with spaces and prefix should succeed even if msgpack is incomplete"
    );
}

#[test]
fn test_messagepack_with_integer_keys() {
    // MessagePack data that uses integer keys in maps (common in compact protocols).
    // This was failing with rmp_serde because serde_json::Value requires string keys.
    let hex_input = "83 00 a8 66 6c 65 65 74  5f 76 31 01 01 02 dc 00 \
        20 84 00 01 01 a3 76 69  64 02 02 03 00 84 00 02 \
        01 a5 76 6e 61 6d 65 02  0a 03 a8 74 72 75 63 6b \
        2d 30 31 84 00 03 01 a5  76 74 79 70 65 02 0b 03 \
        a5 63 61 72 67 6f 84 00  04 01 a5 70 6c 61 74 65 \
        02 0b 03 a6 41 42 43 31  32 33 84 00 05 01 a3 64 \
        72 76 02 0a 03 aa 75 6e  61 73 73 69 67 6e 65 64 \
        84 00 06 01 a5 64 72 76  69 64 02 0a 03 c0 84 00 \
        07 01 a5 66 6c 65 65 74  02 0b 03 a7 64 65 66 61 \
        75 6c 74 84 00 08 01 a5  64 65 70 6f 74 02 0a 03 \
        a9 6d 61 69 6e 2d 79 61  72 64 84 00 09 01 a5 67 \
        70 73 72 74 02 01 03 cd  13 88 84 00 0a 01 a5 67 \
        70 73 65 6e 02 00 03 01  84 00 0b 01 a5 67 70 73 \
        6d 64 02 00 03 00 84 00  0c 01 a4 68 64 6f 70 02 \
        08 03 ca 40 20 00 00 84  00 0d 01 a4 73 72 76 68 \
        02 0a 03 b1 66 6c 65 65  74 2e 65 78 61 6d 70 6c \
        65 2e 63 6f 6d 84 00 0e  01 a4 73 72 76 70 02 01 \
        03 cd 20 fb 84 00 0f 01  a4 61 70 69 6b 02 0a 03 \
        c0 84 00 10 01 a3 74 6c  73 02 00 03 01 84 00 11 \
        01 a4 74 6f 75 74 02 01  03 cd 75 30 84 00 12 01 \
        a5 72 65 74 72 79 02 00  03 03 84 00 13 01 a5 70 \
        72 6f 74 6f 02 0b 03 a4  6d 71 74 74 84 00 14 01 \
        a4 61 73 70 64 02 01 03  78 84 00 15 01 a5 61 69 \
        64 6c 65 02 01 03 cd 01  2c 84 00 16 01 a4 61 74 \
        6d 70 02 04 03 ec 84 00  17 01 a5 61 66 75 65 6c \
        02 00 03 0f 84 00 18 01  a5 64 74 63 65 6e 02 00 \
        03 01 84 00 19 01 a5 6f  62 64 72 74 02 01 03 cd \
        27 10 84 00 1a 01 a5 6c  6f 67 66 6e 02 0a 03 a8 \
        64 69 61 67 2e 6c 6f 67  84 00 1b 01 a5 6c 6f 67 \
        6c 76 02 00 03 02 84 00  1c 01 a5 64 6e 61 6d 65 \
        02 0b 03 a6 67 77 2d 30  30 31 84 00 1d 01 a4 64 \
        73 65 72 02 0a 03 c0 84  00 1e 01 a5 66 77 76 65 \
        72 02 0b 03 a5 31 2e 30  2e 30 84 00 1f 01 a5 68 \
        77 76 65 72 02 0b 03 a5  72 65 76 2d 61 84 00 20 \
        01 a3 6d 66 67 02 0a 03  a8 61 63 6d 65 2d 69 6f \
        74";

    let result = messagepack_to_json(hex_input, InputFormat::Auto);
    assert!(
        result.is_ok(),
        "Integer-key msgpack should decode: {:?}",
        result.err()
    );

    let json: serde_json::Value = serde_json::from_str(&result.unwrap()).unwrap();

    // Top-level map has integer keys 0, 1, 2
    assert_eq!(json["0"], "fleet_v1");
    assert_eq!(json["1"], 1);

    // Key "2" is an array of 32 map entries, each with integer keys
    let arr = json["2"].as_array().expect("key '2' should be an array");
    assert_eq!(arr.len(), 32);

    // Each entry is a map with integer keys; spot-check the first entry
    assert_eq!(arr[0]["0"], 1);
    assert_eq!(arr[0]["1"], "vid");
    assert_eq!(arr[0]["2"], 2);
    assert_eq!(arr[0]["3"], 0);

    // Find the entry with name "plate" and check its value
    let plate_entry = arr
        .iter()
        .find(|e| e["1"] == "plate")
        .expect("should have a 'plate' entry");
    assert_eq!(plate_entry["3"], "ABC123");

    // Find the entry with name "fleet" and check its value
    let fleet_entry = arr
        .iter()
        .find(|e| e["1"] == "fleet")
        .expect("should have a 'fleet' entry");
    assert_eq!(fleet_entry["3"], "default");
}

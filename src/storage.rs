use crate::model::{Settings, Sphere, library_json, parse_library, sphere_json};

#[cfg(target_arch = "wasm32")]
use base64::Engine as _;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

#[cfg(target_arch = "wasm32")]
const LIBRARY_KEY: &str = "mindsphere.library.v1";
#[cfg(target_arch = "wasm32")]
const SETTINGS_KEY: &str = "mindsphere.settings.v1";

pub fn load_library() -> Vec<Sphere> {
    #[cfg(target_arch = "wasm32")]
    {
        web_load_library()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        native_load_library()
    }
}

pub fn save_library(spheres: &[Sphere]) -> Result<(), String> {
    let text = library_json(spheres);
    #[cfg(target_arch = "wasm32")]
    {
        web_set(LIBRARY_KEY, &text).map_err(|_| "Could not write browser storage".to_owned())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let dir = data_dir()?;
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        std::fs::write(dir.join("library.json"), text).map_err(|e| e.to_string())
    }
}

pub fn load_settings() -> Settings {
    #[cfg(target_arch = "wasm32")]
    {
        web_load_settings()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        native_load_settings()
    }
}

pub fn save_settings(settings: &Settings) {
    let Ok(text) = serde_json::to_string_pretty(settings) else {
        return;
    };
    #[cfg(target_arch = "wasm32")]
    {
        let _ = web_set(SETTINGS_KEY, &text);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Ok(dir) = data_dir() {
            let _ = std::fs::create_dir_all(&dir);
            let _ = std::fs::write(dir.join("settings.json"), text);
        }
    }
}

pub fn export_sphere(sphere: &Sphere) -> Result<String, String> {
    let name = safe_filename(&sphere.title);
    let text = sphere_json(sphere);
    #[cfg(target_arch = "wasm32")]
    {
        web_download(&format!("{name}.json"), &text)
            .map(|_| format!("Downloaded {name}.json"))
            .map_err(|_| "Could not start the browser download".to_owned())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let dir = dirs::download_dir()
            .or_else(|| dirs::home_dir().map(|home| home.join("Downloads")))
            .unwrap_or_else(|| std::path::PathBuf::from("."));
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let path = dir.join(format!("{name}.json"));
        std::fs::write(&path, text).map_err(|e| e.to_string())?;
        Ok(format!("Exported to {}", path.display()))
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn read_text_file(path: &std::path::Path) -> Option<String> {
    std::fs::read_to_string(path).ok()
}

#[cfg(target_arch = "wasm32")]
pub fn poll_import() -> Option<(u64, String)> {
    let window = web_sys::window()?;
    let nonce = js_sys::Reflect::get(&window, &"__mindsphereImportNonce".into())
        .ok()?
        .as_f64()? as u64;
    if nonce == 0 {
        return None;
    }
    let text = js_sys::Reflect::get(&window, &"__mindsphereImportText".into())
        .ok()?
        .as_string()?;
    Some((nonce, text))
}

#[cfg(target_arch = "wasm32")]
fn storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok()?
}

#[cfg(target_arch = "wasm32")]
fn web_get(key: &str) -> Option<String> {
    storage()?.get_item(key).ok().flatten()
}

#[cfg(target_arch = "wasm32")]
fn web_set(key: &str, value: &str) -> Result<(), ()> {
    storage().ok_or(())?.set_item(key, value).map_err(|_| ())
}

#[cfg(target_arch = "wasm32")]
fn decode_legacy(raw: &str) -> Option<String> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(raw.trim())
        .ok()?;
    String::from_utf8(bytes).ok()
}

#[cfg(target_arch = "wasm32")]
fn web_load_library() -> Vec<Sphere> {
    let Some(raw) = web_get(LIBRARY_KEY) else {
        return Vec::new();
    };
    let mut text = raw.clone();
    let mut spheres = parse_library(&text);
    if spheres.is_empty() {
        if let Some(decoded) = decode_legacy(&raw) {
            text = decoded;
            spheres = parse_library(&text);
        }
    }
    spheres
}

#[cfg(target_arch = "wasm32")]
fn web_load_settings() -> Settings {
    let Some(raw) = web_get(SETTINGS_KEY) else {
        return Settings::default();
    };
    if let Ok(settings) = serde_json::from_str::<Settings>(&raw) {
        return settings;
    }
    decode_legacy(&raw)
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

#[cfg(target_arch = "wasm32")]
fn web_download(name: &str, text: &str) -> Result<(), ()> {
    let window = web_sys::window().ok_or(())?;
    let function = js_sys::Reflect::get(&window, &"mindsphereDownload".into()).map_err(|_| ())?;
    let function: js_sys::Function = function.dyn_into().map_err(|_| ())?;
    function
        .call2(
            &wasm_bindgen::JsValue::NULL,
            &wasm_bindgen::JsValue::from_str(name),
            &wasm_bindgen::JsValue::from_str(text),
        )
        .map_err(|_| ())?;
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn data_dir() -> Result<std::path::PathBuf, String> {
    let base = dirs::data_dir().ok_or_else(|| "No local data directory".to_owned())?;
    Ok(base.join("mindsphere"))
}

#[cfg(not(target_arch = "wasm32"))]
fn native_load_library() -> Vec<Sphere> {
    let Ok(dir) = data_dir() else {
        return Vec::new();
    };
    if let Ok(text) = std::fs::read_to_string(dir.join("library.json")) {
        let spheres = parse_library(&text);
        if !spheres.is_empty() {
            return spheres;
        }
    }
    migrate_godot_library()
}

#[cfg(not(target_arch = "wasm32"))]
fn migrate_godot_library() -> Vec<Sphere> {
    let Some(base) = dirs::data_dir() else {
        return Vec::new();
    };
    let source = base.join("godot/app_userdata/Mindsphere/spheres");
    let Ok(entries) = std::fs::read_dir(source) else {
        return Vec::new();
    };
    let mut spheres = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        if let Ok(text) = std::fs::read_to_string(path)
            && let Some(sphere) = crate::model::parse_sphere(&text)
        {
            spheres.push(sphere);
        }
    }
    spheres
}

#[cfg(not(target_arch = "wasm32"))]
fn native_load_settings() -> Settings {
    let Ok(dir) = data_dir() else {
        return Settings::default();
    };
    std::fs::read_to_string(dir.join("settings.json"))
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

fn safe_filename(title: &str) -> String {
    let mut name: String = title
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | ' ') {
                c
            } else {
                '-'
            }
        })
        .collect();
    while name.contains("--") {
        name = name.replace("--", "-");
    }
    let name = name.trim_matches('-').to_lowercase();
    if name.is_empty() {
        "mindsphere".to_owned()
    } else {
        name
    }
}

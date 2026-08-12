//! การตั้งค่า Commandblock — อ่านจาก config.json (ในโฟลเดอร์ที่รัน หรือข้างไฟล์ .exe) + ตัวแปร env
//!
//! ลำดับความสำคัญ: env vars > config.json > ค่า default
//! env: BUFF_BACKEND, BUFF_API_KEY, BUFF_BASE_URL, BUFF_MODEL
//!      (รองรับ OPENAI_API_KEY ด้วยถ้าไม่ตั้ง BUFF_API_KEY)

use std::env;
use std::fs;
use std::path::PathBuf;

pub const DEFAULT_OPENAI_URL: &str = "https://api.openai.com/v1";
pub const DEFAULT_OLLAMA_URL: &str = "http://localhost:11434/v1";
pub const DEFAULT_OPENAI_MODEL: &str = "gpt-4o-mini";
pub const DEFAULT_OLLAMA_MODEL: &str = "qwen2.5-coder:7b";

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Backend {
    Auto,
    OpenAI,
    Ollama,
    Offline,
}

impl Backend {
    pub fn label(&self) -> &'static str {
        match self {
            Backend::Auto => "auto",
            Backend::OpenAI => "OpenAI-compatible API",
            Backend::Ollama => "Ollama (ท้องถิ่น)",
            Backend::Offline => "offline (ไม่มี AI)",
        }
    }
}

/// ค่าที่ใช้จริงหลัง resolve แล้ว (backend ถูกตัดสินแล้ว, URL/model เต็ม)
#[derive(Debug, Clone)]
pub struct Effective {
    pub backend: Backend,
    pub base_url: String,
    pub api_key: String,
    pub model: String,
}

/// รายการโมเดลที่เลือกสลับได้ (จาก config.json → ใช้ใน status bar ของ GUI)
/// `api_key` ว่าง = ใช้ key หลักของ config, ไม่ว่าง = ใช้ key ของโมเดลนี้เอง (เช่น Gemini/Groq คนละ key)
#[derive(Debug, Clone)]
pub struct ModelEntry {
    pub name: String,
    pub base_url: String,
    pub api_key: String,
}

pub struct Config {
    pub backend: Backend,
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub models: Vec<ModelEntry>,
}

pub fn fallback_models(active: &Effective, models: &[ModelEntry]) -> Vec<Effective> {
    models
        .iter()
        .filter(|model| !model.name.is_empty() && (model.name != active.model || model.base_url != active.base_url))
        .filter_map(|model| {
            let base_url = if model.base_url.is_empty() { active.base_url.clone() } else { clean_url(model.base_url.clone()) };
            if base_url.is_empty() { return None; }
            Some(Effective {
                backend: Backend::OpenAI,
                base_url,
                api_key: if model.api_key.is_empty() { active.api_key.clone() } else { model.api_key.clone() },
                model: model.name.clone(),
            })
        })
        .collect()
}

#[cfg(test)]
mod fallback_tests {
    use super::*;
    #[test]
    fn fallback_models_exclude_the_active_model() {
        let active = Effective { backend: Backend::OpenAI, base_url: "https://api.deepseek.com".into(), api_key: "key".into(), model: "deepseek-v4-flash".into() };
        let models = vec![
            ModelEntry { name: "deepseek-v4-flash".into(), base_url: active.base_url.clone(), api_key: String::new() },
            ModelEntry { name: "llama-3.3-70b-versatile".into(), base_url: "https://api.groq.com/openai/v1".into(), api_key: "fallback".into() },
        ];
        let fallbacks = fallback_models(&active, &models);
        assert_eq!(fallbacks.len(), 1);
        assert_eq!(fallbacks[0].model, "llama-3.3-70b-versatile");
    }
}

fn get_str(v: &serde_json::Value, key: &str) -> String {
    v.get(key)
        .and_then(|x| x.as_str())
        .map(|s| s.trim().to_string())
        .unwrap_or_default()
}

fn parse_backend(s: &str) -> Backend {
    match s.trim().to_lowercase().as_str() {
        "openai" => Backend::OpenAI,
        "ollama" => Backend::Ollama,
        "offline" => Backend::Offline,
        _ => Backend::Auto,
    }
}

pub fn load() -> Config {
    let config_path = config_path();

    let mut file_backend = String::new();
    let mut file_url = String::new();
    let mut file_key = String::new();
    let mut file_model = String::new();
    let mut file_models: Vec<ModelEntry> = Vec::new();

    if config_path.exists() {
        if let Ok(text) = fs::read_to_string(&config_path) {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
                file_backend = get_str(&v, "backend");
                file_url = get_str(&v, "base_url");
                file_key = get_str(&v, "api_key");
                file_model = get_str(&v, "model");
                // models: รายการสลับโมเดล — รับทั้ง string (ใช้ base_url เดิม) และ {model, base_url, api_key?}
                if let Some(arr) = v.get("models").and_then(|m| m.as_array()) {
                    for it in arr {
                        if let Some(s) = it.as_str() {
                            file_models.push(ModelEntry {
                                name: s.to_string(),
                                base_url: String::new(),
                                api_key: String::new(),
                            });
                        } else if let Some(o) = it.as_object() {
                            let name = o
                                .get("model")
                                .and_then(|x| x.as_str())
                                .unwrap_or("")
                                .trim()
                                .to_string();
                            if !name.is_empty() {
                                file_models.push(ModelEntry {
                                    name,
                                    base_url: o
                                        .get("base_url")
                                        .and_then(|x| x.as_str())
                                        .unwrap_or("")
                                        .trim()
                                        .to_string(),
                                    api_key: o
                                        .get("api_key")
                                        .and_then(|x| x.as_str())
                                        .unwrap_or("")
                                        .trim()
                                        .to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }
    } else {
        // สร้างไฟล์ default ให้อัตโนมัติ
        let default = serde_json::json!({
            "backend": "auto",
            "base_url": "",
            "api_key": "",
            "model": "",
            "models": [],
            "_comment": "ตั้งค่า AI ของ Commandblock ได้ที่นี่ หรือใช้ตัวแปร env (BUFF_BACKEND, BUFF_API_KEY, BUFF_BASE_URL, BUFF_MODEL). backend: auto | openai | ollama | offline | models: รายการสลับโมเดลในแอป [{model,base_url} หรือ string]"
        });
        if let Ok(text) = serde_json::to_string_pretty(&default) {
            if fs::write(&config_path, text).is_ok() {
                println!("[ตั้งค่า] สร้างไฟล์ config.json แล้ว: {}", config_path.display());
            }
        }
    }

    load_from_values(file_backend, file_url, file_key, file_model, file_models)
}

pub fn config_path() -> PathBuf {
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let exe_dir = env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()));
    let mut config_path = cwd.join("config.json");
    if !config_path.exists() {
        if let Some(d) = &exe_dir {
            let alt = d.join("config.json");
            if alt.exists() {
                config_path = alt;
            }
        }
    }
    config_path
}

fn load_from_values(
    file_backend: String,
    file_url: String,
    file_key: String,
    file_model: String,
    file_models: Vec<ModelEntry>,
) -> Config {

    // env vars (ชนะ config.json)
    let env_backend = env::var("BUFF_BACKEND").unwrap_or_default();
    let env_url = env::var("BUFF_BASE_URL").unwrap_or_default();
    let env_key =
        env::var("BUFF_API_KEY").unwrap_or_else(|_| env::var("OPENAI_API_KEY").unwrap_or_default());
    let env_model = env::var("BUFF_MODEL").unwrap_or_default();

    let backend = parse_backend(if !env_backend.is_empty() {
        &env_backend
    } else {
        &file_backend
    });
    let base_url = if !env_url.is_empty() {
        env_url
    } else {
        file_url
    };
    let api_key = if !env_key.is_empty() {
        env_key
    } else {
        file_key
    };
    let model = if !env_model.is_empty() {
        env_model
    } else {
        file_model
    };

    Config {
        backend,
        base_url,
        api_key,
        model,
        models: file_models,
    }
}

/// ตัดสินใจ backend จริง + ค่า default ที่เหลือ
/// `ollama_model` = โมเดลที่ดีที่สุดที่ตรวจพบในเครื่อง (ถ้า Ollama เปิดอยู่)
pub fn effective(cfg: &Config, ollama_reachable: bool, ollama_model: Option<String>) -> Effective {
    let has_key = !cfg.api_key.trim().is_empty();

    let ollama_ready = ollama_reachable && ollama_model.is_some();

    let backend = match cfg.backend {
        Backend::Auto => {
            if has_key {
                Backend::OpenAI
            } else if ollama_ready {
                Backend::Ollama
            } else {
                Backend::Offline
            }
        }
        Backend::Ollama => {
            if ollama_ready {
                Backend::Ollama
            } else {
                Backend::Offline
            }
        }
        b => b,
    };

    match backend {
        Backend::OpenAI => Effective {
            backend,
            base_url: clean_url(if cfg.base_url.is_empty() {
                DEFAULT_OPENAI_URL.into()
            } else {
                cfg.base_url.clone()
            }),
            api_key: cfg.api_key.clone(),
            model: if cfg.model.is_empty() {
                DEFAULT_OPENAI_MODEL.into()
            } else {
                cfg.model.clone()
            },
        },
        Backend::Ollama => {
            // ใช้ localhost เป็นค่าเริ่มต้น ยกเว้นผู้ใช้ตั้ง backend=ollama + base_url เองชัดเจน
            let base = if cfg.backend == Backend::Ollama && !cfg.base_url.is_empty() {
                cfg.base_url.clone()
            } else {
                DEFAULT_OLLAMA_URL.to_string()
            };
            Effective {
                backend,
                base_url: clean_url(base),
                api_key: String::new(),
                // auto-fallback ไป Ollama: ใช้โมเดลท้องถิ่นที่ตรวจพบเสมอ
                // (ยกเว้นผู้ใช้ตั้ง backend=ollama + model เองชัดเจน)
                model: if cfg.backend == Backend::Ollama && !cfg.model.is_empty() {
                    cfg.model.clone()
                } else {
                    ollama_model.unwrap_or_else(|| DEFAULT_OLLAMA_MODEL.into())
                },
            }
        }
        _ => Effective {
            backend: Backend::Offline,
            base_url: String::new(),
            api_key: String::new(),
            model: String::new(),
        },
    }
}

fn clean_url(u: String) -> String {
    u.trim_end_matches('/').to_string()
}

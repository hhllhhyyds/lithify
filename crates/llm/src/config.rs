use lithify_core::LLMError;
use std::env;
use std::time::Duration;

const DEFAULT_MODEL: &str = "claude-sonnet-4-6";
const DEFAULT_MAX_TOKENS: u32 = 4096;
const DEFAULT_TIMEOUT_SECS: u64 = 60;
const DEFAULT_BASE_URL: &str = "https://api.anthropic.com";

/// Configuration for the Anthropic LLM client, read from environment variables.
///
/// | Env var | Required | Default |
/// |---|---|---|
/// | `ANTHROPIC_API_KEY` | Yes | — |
/// | `ANTHROPIC_MODEL` | No | `claude-sonnet-4-20250514` |
/// | `ANTHROPIC_MAX_TOKENS` | No | `4096` |
/// | `ANTHROPIC_TIMEOUT_SECS` | No | `60` |
/// | `ANTHROPIC_BASE_URL` | No | `https://api.anthropic.com` |
#[derive(Debug, Clone)]
pub struct Config {
    pub api_key: String,
    pub model: String,
    pub max_tokens: u32,
    pub timeout: Duration,
    pub(crate) base_url: String,
}

impl Config {
    /// Read configuration from environment variables.
    ///
    /// Returns `LLMError::Api` if `ANTHROPIC_API_KEY` is not set.
    pub fn from_env() -> Result<Self, LLMError> {
        let api_key = env::var("ANTHROPIC_API_KEY")
            .map_err(|_| LLMError::Api("ANTHROPIC_API_KEY not set".into()))?;

        let model = env::var("ANTHROPIC_MODEL").unwrap_or_else(|_| DEFAULT_MODEL.to_string());

        let max_tokens = env::var("ANTHROPIC_MAX_TOKENS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(DEFAULT_MAX_TOKENS);

        let timeout_secs = env::var("ANTHROPIC_TIMEOUT_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(DEFAULT_TIMEOUT_SECS);

        let base_url =
            env::var("ANTHROPIC_BASE_URL").unwrap_or_else(|_| DEFAULT_BASE_URL.to_string());

        Ok(Config {
            api_key,
            model,
            max_tokens,
            timeout: Duration::from_secs(timeout_secs),
            base_url,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Snapshot ANTHROPIC_* env vars and restore them on drop.
    ///
    /// This ensures test isolation without per-test cleanup boilerplate,
    /// and keeps branch coverage clean since the match in `drop` handles
    /// both `Some` (set) and `None` (remove) arms in a single invocation
    /// when different keys are in different states.
    struct EnvGuard {
        api_key: Option<String>,
        model: Option<String>,
        max_tokens: Option<String>,
        timeout: Option<String>,
    }

    impl EnvGuard {
        fn new() -> Self {
            Self {
                api_key: env::var("ANTHROPIC_API_KEY").ok(),
                model: env::var("ANTHROPIC_MODEL").ok(),
                max_tokens: env::var("ANTHROPIC_MAX_TOKENS").ok(),
                timeout: env::var("ANTHROPIC_TIMEOUT_SECS").ok(),
            }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            // SAFETY: writing to env vars is not thread-safe, EnvGuard is
            // used inside tests that are serialized by ENV_LOCK below.
            //
            // Use a single match over all keys so different key states
            // exercise both match arms in one call, keeping coverage clean.
            unsafe {
                for (key, val) in [
                    ("ANTHROPIC_API_KEY", &self.api_key),
                    ("ANTHROPIC_MODEL", &self.model),
                    ("ANTHROPIC_MAX_TOKENS", &self.max_tokens),
                    ("ANTHROPIC_TIMEOUT_SECS", &self.timeout),
                ] {
                    match val {
                        Some(v) => env::set_var(key, v),
                        None => env::remove_var(key),
                    }
                }
            }
        }
    }

    use std::sync::Mutex;
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn set_env(key: &str, value: &str) {
        unsafe { env::set_var(key, value) };
    }

    fn remove_env(key: &str) {
        unsafe { env::remove_var(key) };
    }

    #[test]
    fn from_env_requires_api_key() {
        let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let _guard = EnvGuard::new();
        remove_env("ANTHROPIC_API_KEY");
        // Also remove model/timeout/etc so from_env can't accidentally pass
        remove_env("ANTHROPIC_MODEL");
        remove_env("ANTHROPIC_MAX_TOKENS");
        remove_env("ANTHROPIC_TIMEOUT_SECS");

        let result = Config::from_env();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("ANTHROPIC_API_KEY")
        );
    }

    #[test]
    fn from_env_reads_api_key() {
        let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let _guard = EnvGuard::new();
        set_env("ANTHROPIC_API_KEY", "sk-test-123");

        let config = Config::from_env().unwrap();
        assert_eq!(config.api_key, "sk-test-123");
    }

    #[test]
    fn from_env_default_model() {
        let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let _guard = EnvGuard::new();
        set_env("ANTHROPIC_API_KEY", "sk-test");
        remove_env("ANTHROPIC_MODEL");

        let config = Config::from_env().unwrap();
        assert_eq!(config.model, DEFAULT_MODEL);
    }

    #[test]
    fn from_env_default_timeout() {
        let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let _guard = EnvGuard::new();
        set_env("ANTHROPIC_API_KEY", "sk-test");

        let config = Config::from_env().unwrap();
        assert_eq!(config.timeout, Duration::from_secs(DEFAULT_TIMEOUT_SECS));
    }

    #[test]
    fn from_env_reads_custom_timeout() {
        let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let _guard = EnvGuard::new();
        set_env("ANTHROPIC_API_KEY", "sk-test");
        set_env("ANTHROPIC_TIMEOUT_SECS", "30");

        let config = Config::from_env().unwrap();
        assert_eq!(config.timeout, Duration::from_secs(30));
    }

    #[test]
    fn from_env_custom_max_tokens() {
        let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let _guard = EnvGuard::new();
        set_env("ANTHROPIC_API_KEY", "sk-test");
        set_env("ANTHROPIC_MAX_TOKENS", "8192");

        let config = Config::from_env().unwrap();
        assert_eq!(config.max_tokens, 8192);
    }
}

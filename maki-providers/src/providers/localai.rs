//! LocalAI provider: OpenAI-compatible local inference with LocalAI's
//! `/v1/models/capabilities` discovery surface (and Ollama-compat `/api`
//! endpoints as a fallback).

use std::borrow::Cow;
use std::sync::{Arc, Mutex};

use maki_config::providers::Protocol;

use crate::model::ModelFamily;
use crate::provider::Provider;
use crate::spec::{
    ApertureRoute, AuthDoc, Build, CatalogDoc, GeneratedDocs, LoginConfig, NO_CURATED_MODELS,
    Native, ProviderSpec,
};

use super::{ResolvedAuth, Timeouts};
use crate::AgentError;
use crate::types::ThinkingFallback;

use super::aperture::DEFAULT_PATH_PREFIX;
use super::local::{DiscoveryMode, LocalEndpoint, LocalEndpointConfig};
use super::openai_compat::{DEFAULT_MAX_TOKENS_FIELD, OpenAiCompatConfig};

const LOCALAI_SLUG: &str = "localai";
const LOCALAI_BASE_URL: &str = "http://localhost:8080/v1";
const LOCALAI_KEY_ENV: &str = "LOCALAI_API_KEY";

/// LocalAI is configured via `LOCALAI_HOST`. The key stays optional: most
/// instances need none, but a proxy in front of one may demand one, and
/// `KeyPool::resolve` failing is how that stays optional.
const AUTH_DOC: &str = "`LOCALAI_HOST` for local/remote (e.g. `http://localhost:8080`); \
     most instances need no key, but set `LOCALAI_API_KEY` for one behind a proxy";

const FEATURES: &str =
    "Local or remote inference via LOCALAI_HOST, set optional key via LOCALAI_API_KEY";

/// Discovery note: LocalAI's capability endpoint is the authoritative source
/// for context length, and the Ollama-compat `/api` endpoints are a fallback.
const DISCOVERY_NOTE: &str = "LocalAI exposes its full model table over the OpenAI-compatible `/v1` API; its \
    `/v1/models/capabilities` endpoint additionally publishes `context_size` plus \
    capabilities/modalities. Older instances fall back to the Ollama-compat \
    `/api/tags` + `/api/show`. This provider also works with llama.cpp's server, \
    which speaks the same protocol.";

pub(crate) const LOCALAI: LocalEndpointConfig = LocalEndpointConfig {
    slug: LOCALAI_SLUG,
    display_name: "LocalAI",
    host_env: "LOCALAI_HOST",
    api_key_env: LOCALAI_KEY_ENV,
    default_host: "http://localhost:8080",
    default_model: "localai/default",
    cloud_fallback_url: None,
    discovery_mode: DiscoveryMode::LocalAI,
    compat: OpenAiCompatConfig {
        slug: Cow::Borrowed(LOCALAI_SLUG),
        api_key_env: Cow::Borrowed(""),
        base_url: Cow::Borrowed(LOCALAI_BASE_URL),
        max_tokens_field: Cow::Borrowed(DEFAULT_MAX_TOKENS_FIELD),
        include_stream_usage: true,
        provider_name: Cow::Borrowed("LocalAI"),
    },
    thinking_fallback: ThinkingFallback::BudgetField,
};

pub(crate) const SPEC: ProviderSpec = ProviderSpec {
    slug: LOCALAI_SLUG,
    display_name: "LocalAI",
    api_key_env: LOCALAI_KEY_ENV,
    family: ModelFamily::Generic,
    supports_thinking: true,
    supports_deferred_tools: false,
    accepts_arbitrary_models: true,
    fallback_max_output: None,
    fallback_context_window: 128_000,
    models_toml: NO_CURATED_MODELS,
    pricing_schedule: None,
    build: Build::Native(Native {
        new: create,
        with_auth: create_with_auth,
    }),
    aperture: Some(ApertureRoute {
        path_prefix: DEFAULT_PATH_PREFIX,
    }),
    login: Some(LoginConfig {
        protocol: Protocol::Openai,
        default_base_url: "http://localhost:8080",
        default_model: "localai/default",
        plans: None,
        login_url: None,
        needs_url: true,
    }),
    docs: GeneratedDocs {
        api_urls: &[LOCALAI_BASE_URL],
        auth: AuthDoc::Custom(AUTH_DOC),
        features: Some(FEATURES),
        catalog: CatalogDoc::Discovered(DISCOVERY_NOTE),
        trailing_notes: &[],
    },
};

inventory::submit!(SPEC.config_row());

fn create(timeouts: Timeouts) -> Result<Box<dyn Provider>, AgentError> {
    Ok(Box::new(LocalEndpoint::new(&LOCALAI, timeouts)?))
}

fn create_with_auth(
    auth: Arc<Mutex<ResolvedAuth>>,
    timeouts: super::Timeouts,
    system_prefix: Option<String>,
) -> Box<dyn Provider> {
    Box::new(LocalEndpoint::with_auth(&LOCALAI, auth, timeouts).with_system_prefix(system_prefix))
}

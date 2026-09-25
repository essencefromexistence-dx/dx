use anyhow::{Context as _, Result, anyhow};
use futures::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, FutureExt, StreamExt, future::BoxFuture, io::BufReader, stream::BoxStream};
use gpui::{AnyView, App, AsyncApp, Context, Entity, Subscription, Task, Window};
use http_client::{AsyncBody, HttpClient, Method, Request as HttpRequest, StatusCode};
use language_model::{AuthenticateError, IconOrSvg, LanguageModel, LanguageModelCompletionError, LanguageModelCompletionEvent, LanguageModelId, LanguageModelName, LanguageModelProvider, LanguageModelProviderId, LanguageModelProviderName, LanguageModelProviderState, LanguageModelRequest, LanguageModelToolChoice, LanguageModelToolSchemaFormat, MessageContent, RateLimiter, Role, StopReason, TokenUsage};
use serde::{Deserialize, Serialize};
use settings::{Settings, SettingsStore};
use std::{collections::BTreeMap, env, fs::File, io::{Cursor, Read}, path::{Path, PathBuf}, process::{Child, Command, Stdio}, sync::{Arc, Mutex}, time::Duration};
use ui::{ButtonLike, ConfiguredApiCard, List, ListBulletItem, prelude::*};
use util::ResultExt;
use crate::AllLanguageModelSettings;
use collections::{HashMap, HashSet};
use credentials_provider::CredentialsProvider;
use fs::Fs;
use gpui::{TaskExt};
use http_client::{CustomHeaders};
use language_model::{ApiKeyState, EnvVar, InlineDescription, LanguageModelToolResultContent, ProviderSettingsView, SubPageProviderSettings, env_var};
use llama_cpp::{LLAMA_CPP_API_URL, ModelEntry, Props, get_models, get_props, stream_chat_completion, stream_model_events};
use settings::{update_settings_file};
use std::sync::LazyLock;
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use ui::{ButtonLink, Divider, Tooltip};
use ui_input::InputField;
use language_model::chat_completion::ChatCompletionEventMapper;
pub use settings::LlamaCppAvailableModel as AvailableModel;

const LOCAL_LLAMA_PROVIDER_ID: LanguageModelProviderId = LanguageModelProviderId::new("llama_cpp");
const LOCAL_LLAMA_PROVIDER_NAME: LanguageModelProviderName = LanguageModelProviderName::new("Llama.cpp");

const LOW_SPEC_MEMORY_THRESHOLD_BYTES: u64 = 16 * 1024 * 1024 * 1024;
const LOCAL_LLAMA_SERVER_PARALLEL_SLOTS: u64 = 1;
const LOCAL_LLAMA_PROMPT_TOKEN_MARGIN: u64 = 256;

const LLAMA_CPP_DOWNLOAD_URL: &str = "https://llama.app";
const LLAMA_CPP_MODELS_URL: &str = "https://huggingface.co/models?library=gguf&sort=trending";

const PROVIDER_ID: LanguageModelProviderId = LanguageModelProviderId::new("llama.cpp");
const PROVIDER_NAME: LanguageModelProviderName = LanguageModelProviderName::new("llama.cpp");

const API_KEY_ENV_VAR_NAME: &str = "LLAMACPP_API_KEY";
static API_KEY_ENV_VAR: LazyLock<EnvVar> = env_var!(API_KEY_ENV_VAR_NAME);

/// How long to wait before reconnecting to `/models/sse` after the stream ends.
const MODEL_EVENT_RECONNECT_INTERVAL: Duration = Duration::from_secs(5);

/// Context length assumed for an unloaded router model (it can't be probed
/// without loading it). Generous so early messages work; re-discovery refines
/// it once the model loads.
const ASSUMED_UNLOADED_CONTEXT: u64 = 131_072;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LlamaCppSettings {
    pub api_url: String,
    pub host: String,
    pub port: u16,
    pub server_path: Option<String>,
    pub models_dir: Option<String>,
    pub cache_dir: Option<String>,
    pub tools_dir: Option<String>,
    pub auto_start: bool,
    pub auto_download_models: bool,
    pub auto_install_server: bool,
    pub auto_discover_local_models: bool,
    /// Ours' concrete context length; see `remote_context_window` for upstream's tri-state.
    pub context_window: u64,
    /// Upstream's tri-state override: `None` means "unset - keep the model's own `max_tokens`".
    pub remote_context_window: Option<u64>,
    pub threads: Option<u64>,
    pub gpu_layers: Option<i64>,
    pub normal_model: AvailableModel,
    pub low_spec_model: AvailableModel,
    pub available_models: Vec<AvailableModel>,
    pub auto_discover: bool,
    pub custom_headers: CustomHeaders,
}

pub struct LocalLlamaCppLanguageModelProvider {
    http_client: Arc<dyn HttpClient>,
    state: Entity<LocalLlamaCppState>,
}

pub struct LocalLlamaCppState {
    settings: LlamaCppSettings,
    use_low_spec_model: bool,
    discovered_models: Vec<AvailableModel>,
    runtime: Arc<LlamaCppRuntime>,
    _subscription: Subscription,
}

impl LocalLlamaCppState {
    fn selected_model(&self) -> AvailableModel {
        if self.use_low_spec_model {
            self.settings.low_spec_model.clone()
        } else {
            self.settings.normal_model.clone()
        }
    }

    fn all_models(&self) -> Vec<AvailableModel> {
        let mut models = BTreeMap::default();
        models.insert(self.selected_model().name, self.selected_model());
        models.insert(
            self.settings.normal_model.name.clone(),
            self.settings.normal_model.clone(),
        );
        models.insert(
            self.settings.low_spec_model.name.clone(),
            self.settings.low_spec_model.clone(),
        );
        for model in &self.settings.available_models {
            models.insert(model.name.clone(), model.clone());
        }
        for model in &self.discovered_models {
            models.entry(model.name.clone()).or_insert(model.clone());
        }

        models.into_values().collect()
    }

    fn discover_models_from_settings(settings: &LlamaCppSettings) -> Vec<AvailableModel> {
        if !settings.auto_discover_local_models {
            return Vec::new();
        }

        let models_dir = settings
            .models_dir
            .as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(default_models_dir);
        let known_local_paths = std::iter::once(&settings.normal_model)
            .chain(std::iter::once(&settings.low_spec_model))
            .chain(settings.available_models.iter())
            .filter_map(|model| model.local_path.as_ref().map(PathBuf::from))
            .collect::<Vec<_>>();

        discover_local_gguf_models(&models_dir, &known_local_paths, settings.context_window)
    }

    fn launch_config(&self, model: &AvailableModel) -> LlamaCppLaunchConfig {
        LlamaCppLaunchConfig {
            api_url: self.settings.api_url.clone(),
            host: self.settings.host.clone(),
            port: self.settings.port,
            server_path: self.settings.server_path.as_ref().map(PathBuf::from),
            models_dir: self
                .settings
                .models_dir
                .as_ref()
                .map(PathBuf::from)
                .unwrap_or_else(default_models_dir),
            cache_dir: self
                .settings
                .cache_dir
                .as_ref()
                .map(PathBuf::from)
                .unwrap_or_else(default_cache_dir),
            tools_dir: self
                .settings
                .tools_dir
                .as_ref()
                .map(PathBuf::from)
                .unwrap_or_else(default_tools_dir),
            auto_start: self.settings.auto_start,
            auto_download_models: self.settings.auto_download_models,
            auto_install_server: self.settings.auto_install_server,
            context_window: model.max_tokens,
            threads: self.settings.threads,
            gpu_layers: self.settings.gpu_layers,
            model: model.clone(),
        }
    }
}

impl LocalLlamaCppLanguageModelProvider {
    pub fn new(http_client: Arc<dyn HttpClient>, cx: &mut App) -> Self {
        let runtime = Arc::new(LlamaCppRuntime::default());
        let state = cx.new(|cx| {
            let subscription = cx.observe_global::<SettingsStore>({
                let mut settings = AllLanguageModelSettings::get_global(cx).llama_cpp.clone();
                move |this: &mut LocalLlamaCppState, cx| {
                    let new_settings = AllLanguageModelSettings::get_global(cx).llama_cpp.clone();
                    if settings != new_settings {
                        settings = new_settings.clone();
                        this.discovered_models =
                            LocalLlamaCppState::discover_models_from_settings(&new_settings);
                        this.settings = new_settings;
                        cx.notify();
                    }
                }
            });
            let settings = AllLanguageModelSettings::get_global(cx).llama_cpp.clone();
            let discovered_models = LocalLlamaCppState::discover_models_from_settings(&settings);

            LocalLlamaCppState {
                settings,
                use_low_spec_model: is_low_spec_system(),
                discovered_models,
                runtime,
                _subscription: subscription,
            }
        });

        Self { http_client, state }
    }

    fn create_language_model(&self, model: AvailableModel) -> Arc<dyn LanguageModel> {
        Arc::new(LocalLlamaCppLanguageModel {
            id: LanguageModelId::from(model.name.clone()),
            model,
            state: self.state.clone(),
            http_client: self.http_client.clone(),
            request_limiter: RateLimiter::new(2),
        })
    }
}

impl LanguageModelProviderState for LocalLlamaCppLanguageModelProvider {
    type ObservableEntity = LocalLlamaCppState;

    fn observable_entity(&self) -> Option<Entity<Self::ObservableEntity>> {
        Some(self.state.clone())
    }
}

impl LanguageModelProvider for LocalLlamaCppLanguageModelProvider {
    fn id(&self) -> LanguageModelProviderId {
        LOCAL_LLAMA_PROVIDER_ID
    }

    fn name(&self) -> LanguageModelProviderName {
        LOCAL_LLAMA_PROVIDER_NAME
    }

    fn icon(&self) -> IconOrSvg {
        IconOrSvg::Icon(IconName::AiOllama)
    }

    fn default_model(&self, cx: &App) -> Option<Arc<dyn LanguageModel>> {
        Some(self.create_language_model(self.state.read(cx).selected_model()))
    }

    fn default_fast_model(&self, cx: &App) -> Option<Arc<dyn LanguageModel>> {
        Some(self.create_language_model(self.state.read(cx).settings.low_spec_model.clone()))
    }

    fn provided_models(&self, cx: &App) -> Vec<Arc<dyn LanguageModel>> {
        self.state
            .read(cx)
            .all_models()
            .into_iter()
            .map(|model| self.create_language_model(model))
            .collect()
    }

    fn recommended_models(&self, cx: &App) -> Vec<Arc<dyn LanguageModel>> {
        vec![self.create_language_model(self.state.read(cx).selected_model())]
    }

    fn is_authenticated(&self, _cx: &App) -> bool {
        true
    }

    fn authenticate(&self, _cx: &mut App) -> Task<Result<(), AuthenticateError>> {
        // Local models do not have credentials. Avoid loading a model during
        // background provider authentication because the selected agent model
        // may be different from the provider's fallback/default model.
        Task::ready(Ok(()))
    }

    fn configuration_view(
        &self,
        _target_agent: language_model::ConfigurationViewTargetAgent,
        _window: &mut Window,
        cx: &mut App,
    ) -> AnyView {
        cx.new(|cx| LocalLlamaCppConfigurationView::new(self.state.clone(), self.http_client.clone(), cx))
            .into()
    }

    fn reset_credentials(&self, _cx: &mut App) -> Task<Result<()>> {
        Task::ready(Ok(()))
    }
}

pub struct LocalLlamaCppLanguageModel {
    id: LanguageModelId,
    model: AvailableModel,
    state: Entity<LocalLlamaCppState>,
    http_client: Arc<dyn HttpClient>,
    request_limiter: RateLimiter,
}

impl LocalLlamaCppLanguageModel {
    fn stream_completion(
        &self,
        request: LanguageModelRequest,
        cx: &AsyncApp,
    ) -> BoxFuture<
        'static,
        Result<
            BoxStream<'static, Result<LanguageModelCompletionEvent, LanguageModelCompletionError>>,
            LanguageModelCompletionError,
        >,
    > {
        let (runtime, config) = self.state.read_with(cx, |state, _| {
            (state.runtime.clone(), state.launch_config(&self.model))
        });
        let http_client = self.http_client.clone();
        let future = self.request_limiter.stream(async move {
            log::info!(
                "starting llama.cpp completion with model {}",
                config.model.name
            );
            runtime
                .ensure_server(config.clone(), http_client.clone())
                .await
                .map_err(LanguageModelCompletionError::Other)?;
            let config = effective_launch_config(http_client.as_ref(), config).await;
            stream_llama_completion(http_client.as_ref(), &config, request).await
        });

        async move { Ok(future.await?.boxed()) }.boxed()
    }
}

impl LanguageModel for LocalLlamaCppLanguageModel {
    fn id(&self) -> LanguageModelId {
        self.id.clone()
    }

    fn name(&self) -> LanguageModelName {
        LanguageModelName::from(
            self.model
                .display_name
                .clone()
                .unwrap_or_else(|| self.model.name.clone()),
        )
    }

    fn provider_id(&self) -> LanguageModelProviderId {
        LOCAL_LLAMA_PROVIDER_ID
    }

    fn provider_name(&self) -> LanguageModelProviderName {
        LOCAL_LLAMA_PROVIDER_NAME
    }

    fn supports_tools(&self) -> bool {
        false
    }

    fn tool_input_format(&self) -> LanguageModelToolSchemaFormat {
        LanguageModelToolSchemaFormat::JsonSchemaSubset
    }

    fn supports_tool_choice(&self, choice: LanguageModelToolChoice) -> bool {
        match choice {
            LanguageModelToolChoice::Auto | LanguageModelToolChoice::Any => false,
            LanguageModelToolChoice::None => true,
        }
    }

    fn supports_streaming_tools(&self) -> bool {
        false
    }

    fn supports_images(&self) -> bool {
        self.model.supports_images
    }

    fn supports_split_token_display(&self) -> bool {
        true
    }

    fn telemetry_id(&self) -> String {
        format!("llama_cpp/{}", self.model.name)
    }

    fn max_token_count(&self) -> u64 {
        self.model.max_tokens
    }

    fn max_output_tokens(&self) -> Option<u64> {
        self.model.max_output_tokens
    }

    fn stream_completion(
        &self,
        request: LanguageModelRequest,
        cx: &AsyncApp,
    ) -> BoxFuture<
        'static,
        Result<
            futures::stream::BoxStream<
                'static,
                Result<LanguageModelCompletionEvent, LanguageModelCompletionError>,
            >,
            LanguageModelCompletionError,
        >,
    > {
        self.stream_completion(request, cx)
    }
}

#[derive(Clone)]
struct LlamaCppLaunchConfig {
    api_url: String,
    host: String,
    port: u16,
    server_path: Option<PathBuf>,
    models_dir: PathBuf,
    cache_dir: PathBuf,
    tools_dir: PathBuf,
    auto_start: bool,
    auto_download_models: bool,
    auto_install_server: bool,
    context_window: u64,
    threads: Option<u64>,
    gpu_layers: Option<i64>,
    model: AvailableModel,
}

impl LlamaCppLaunchConfig {
    fn signature(&self, server_path: &Path) -> String {
        format!(
            "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
            server_path.display(),
            self.api_url,
            self.model.name,
            self.model.local_path.as_deref().unwrap_or_default(),
            self.model.hf_repo.as_deref().unwrap_or_default(),
            self.model.hf_quant.as_deref().unwrap_or_default(),
            self.context_window,
            self.threads
                .map(|threads| threads.to_string())
                .unwrap_or_default(),
            self.gpu_layers
                .map(|gpu_layers| gpu_layers.to_string())
                .unwrap_or_default(),
            LOCAL_LLAMA_SERVER_PARALLEL_SLOTS,
        )
    }
}

#[derive(Default)]
struct LlamaCppRuntime {
    startup_lock: async_lock::Mutex<()>,
    running: Mutex<Option<RunningServer>>,
}

struct RunningServer {
    signature: String,
    child: Child,
}

impl Drop for LlamaCppRuntime {
    fn drop(&mut self) {
        if let Some(mut running) = self.running.lock().ok().and_then(|mut child| child.take()) {
            _ = running.child.kill();
        }
    }
}

impl LlamaCppRuntime {
    #[allow(clippy::disallowed_methods)]
    async fn ensure_server(
        &self,
        config: LlamaCppLaunchConfig,
        http_client: Arc<dyn HttpClient>,
    ) -> Result<()> {
        if server_is_ready_for_model(http_client.as_ref(), &config).await {
            return Ok(());
        }

        let _startup_guard = self.startup_lock.lock().await;
        if server_is_ready_for_model(http_client.as_ref(), &config).await {
            return Ok(());
        }

        if !config.auto_start {
            return Ok(());
        }

        let server_path = self
            .resolve_server_path(&config, http_client.clone())
            .await
            .context("finding llama-server")?;
        let signature = config.signature(&server_path);

        {
            let mut running = self.running.lock().unwrap();
            if let Some(running_server) = running.as_mut() {
                if running_server.child.try_wait()?.is_none()
                    && running_server.signature == signature
                {
                    return Ok(());
                }
                log::info!(
                    "restarting llama.cpp server for selected model {}",
                    config.model.name
                );
                _ = running_server.child.kill();
                _ = running_server.child.wait();
                *running = None;
            }
        }

        if server_is_ready(http_client.as_ref(), &config.api_url).await {
            let loaded_models = server_model_names(http_client.as_ref(), &config.api_url)
                .await
                .unwrap_or_default();
            let loaded_model_names = loaded_models.join(", ");
            if server_models_include(&loaded_models, &config.model) {
                let running_props = server_props(http_client.as_ref(), &config.api_url)
                    .await
                    .ok();
                let running_settings = running_props
                    .as_ref()
                    .map(|props| props.describe())
                    .unwrap_or_else(|| "unknown server settings".to_string());
                if running_props
                    .as_ref()
                    .is_some_and(|props| props.matches_config(&config))
                {
                    log::warn!(
                        "reusing already-running llama.cpp server at {} with model(s) [{}] and settings ({}); requested {} ctx and {} parallel slot",
                        config.api_url,
                        loaded_model_names,
                        running_settings,
                        config.context_window,
                        LOCAL_LLAMA_SERVER_PARALLEL_SLOTS
                    );
                    return Ok(());
                }

                if stop_existing_local_llama_server_for_switch(&config, &server_path)? {
                    log::info!(
                        "stopped existing local llama-server at {} with model(s) [{}] and settings ({}) so {} can restart with {} ctx and {} parallel slot",
                        config.api_url,
                        loaded_model_names,
                        running_settings,
                        config.model.name,
                        config.context_window,
                        LOCAL_LLAMA_SERVER_PARALLEL_SLOTS
                    );
                    async_io::Timer::after(Duration::from_millis(500)).await;
                } else {
                    anyhow::bail!(
                        "llama-server is already running at {} with model(s) [{}] and settings ({}), but {} needs {} ctx and {} parallel slot. Stop the existing llama-server or use a different port.",
                        config.api_url,
                        loaded_model_names,
                        running_settings,
                        config.model.name,
                        config.context_window,
                        LOCAL_LLAMA_SERVER_PARALLEL_SLOTS
                    );
                }
            } else {
                if stop_existing_local_llama_server_for_switch(&config, &server_path)? {
                    log::info!(
                        "stopped existing local llama-server at {} with model(s) [{}] so {} can start",
                        config.api_url,
                        loaded_model_names,
                        config.model.name
                    );
                    async_io::Timer::after(Duration::from_millis(500)).await;
                } else {
                    anyhow::bail!(
                        "llama-server is already running at {} with model(s) [{}], but {} was selected. Stop the existing llama-server or use a different port.",
                        config.api_url,
                        loaded_model_names,
                        config.model.name
                    );
                }
            }
        }

        std::fs::create_dir_all(&config.models_dir)
            .with_context(|| format!("creating {}", config.models_dir.display()))?;
        std::fs::create_dir_all(&config.cache_dir)
            .with_context(|| format!("creating {}", config.cache_dir.display()))?;
        std::fs::create_dir_all(&config.tools_dir)
            .with_context(|| format!("creating {}", config.tools_dir.display()))?;

        let model_path = if let Some(model_path) = resolve_local_model_path(&config) {
            Some(model_path)
        } else if config.auto_download_models {
            download_configured_model_file(&config, http_client.as_ref()).await?
        } else {
            None
        };

        let log_path = config.tools_dir.join("llama-server.log");
        let stdout = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .with_context(|| format!("opening {}", log_path.display()))?;
        let stderr = stdout
            .try_clone()
            .with_context(|| format!("cloning {}", log_path.display()))?;

        let mut command = Command::new(&server_path);
        command
            .arg("--host")
            .arg(&config.host)
            .arg("--port")
            .arg(config.port.to_string())
            .arg("-c")
            .arg(config.context_window.to_string())
            .arg("--parallel")
            .arg(LOCAL_LLAMA_SERVER_PARALLEL_SLOTS.to_string())
            .arg("--alias")
            .arg(&config.model.name)
            .arg("--jinja")
            .stdin(Stdio::null())
            .stdout(stdout)
            .stderr(stderr)
            .env("HF_HOME", &config.cache_dir)
            .env("HF_HUB_CACHE", config.cache_dir.join("hub"))
            .env("LLAMA_CACHE", config.cache_dir.join("llama.cpp"))
            .env("GIT_TERMINAL_PROMPT", "0");

        if let Some(parent) = server_path.parent() {
            command.current_dir(parent);
        }

        if let Some(threads) = config.threads {
            let threads = threads.to_string();
            command.arg("-t").arg(&threads).arg("-tb").arg(threads);
        }

        if let Some(gpu_layers) = config.gpu_layers {
            command.arg("-ngl").arg(gpu_layers.to_string());
        }

        if let Some(model_path) = model_path {
            command.arg("-m").arg(model_path);
        } else if config.auto_download_models {
            add_hugging_face_model_args(&mut command, &config.model)?;
        } else {
            anyhow::bail!(
                "model {} was not found in {} and auto_download_models is disabled",
                config.model.name,
                config.models_dir.display()
            );
        }

        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x08000000);
        }

        log::info!(
            "launching llama.cpp server for model {} at {}",
            config.model.name,
            config.api_url
        );

        let child = command
            .spawn()
            .with_context(|| format!("starting {}", server_path.display()))?;

        {
            let mut running = self.running.lock().unwrap();
            *running = Some(RunningServer { signature, child });
        }

        for _ in 0..900 {
            if server_is_ready_for_model(http_client.as_ref(), &config).await {
                warm_up_llama_server(http_client.as_ref(), &config).await;
                return Ok(());
            }
            {
                let mut running = self.running.lock().unwrap();
                if let Some(running_server) = running.as_mut()
                    && let Some(status) = running_server.child.try_wait()?
                {
                    *running = None;
                    anyhow::bail!("llama-server exited before becoming ready: {status}");
                }
            }
            async_io::Timer::after(Duration::from_secs(1)).await;
        }

        anyhow::bail!(
            "timed out waiting for llama-server at {}. The first model download can be large; try again after it finishes.",
            config.api_url
        );
    }

    async fn resolve_server_path(
        &self,
        config: &LlamaCppLaunchConfig,
        http_client: Arc<dyn HttpClient>,
    ) -> Result<PathBuf> {
        if let Some(path) = config.server_path.as_ref()
            && (path.exists() || path.components().count() == 1)
        {
            return Ok(path.clone());
        }

        if let Some(path) = find_server_binary(&config.tools_dir) {
            return Ok(path);
        }

        if let Some(path) = find_server_on_path() {
            return Ok(path);
        }

        if !config.auto_install_server {
            anyhow::bail!(
                "llama-server was not found. Install llama.cpp or set language_models.llama_cpp.server_path"
            );
        }

        download_llama_server(http_client, &config.tools_dir).await
    }
}

async fn server_is_ready(http_client: &dyn HttpClient, api_url: &str) -> bool {
    let url = format!("{}/health", api_url.trim_end_matches('/'));
    let Ok(response) = http_client.get(&url, AsyncBody::default(), true).await else {
        return false;
    };
    response.status().is_success()
}

async fn server_is_ready_for_model(
    http_client: &dyn HttpClient,
    config: &LlamaCppLaunchConfig,
) -> bool {
    if !server_is_ready(http_client, &config.api_url).await {
        return false;
    }

    if !server_model_names(http_client, &config.api_url)
        .await
        .is_ok_and(|models| server_models_include(&models, &config.model))
    {
        return false;
    }

    server_props(http_client, &config.api_url)
        .await
        .map(|props| props.matches_config(config))
        .unwrap_or(true)
}

async fn server_model_names(http_client: &dyn HttpClient, api_url: &str) -> Result<Vec<String>> {
    let url = format!("{}/models", api_url.trim_end_matches('/'));
    let mut response = http_client
        .get(&url, AsyncBody::default(), true)
        .await
        .with_context(|| format!("checking loaded llama.cpp models at {url}"))?;
    anyhow::ensure!(
        response.status().is_success(),
        "checking loaded llama.cpp models returned {}",
        response.status()
    );

    let mut body = String::new();
    response
        .body_mut()
        .read_to_string(&mut body)
        .await
        .with_context(|| format!("reading {url}"))?;

    let response: LlamaModelsResponse =
        serde_json::from_str(&body).with_context(|| format!("parsing {url} response"))?;
    let mut names = Vec::new();
    for model in response.data.into_iter().chain(response.models) {
        if let Some(id) = model.id {
            names.push(id);
        }
        if let Some(name) = model.name {
            names.push(name);
        }
        if let Some(model_name) = model.model {
            names.push(model_name);
        }
        names.extend(model.aliases);
    }

    names.sort();
    names.dedup();
    Ok(names)
}

fn server_models_include(models: &[String], requested: &AvailableModel) -> bool {
    let requested_names = requested_model_names(requested);
    models
        .iter()
        .map(|model| model.to_lowercase())
        .any(|model| requested_names.iter().any(|requested| requested == &model))
}

fn requested_model_names(model: &AvailableModel) -> Vec<String> {
    let mut names = vec![model.name.to_lowercase()];
    if let Some(local_path) = model.local_path.as_ref().map(PathBuf::from)
        && let Some(stem) = local_path.file_stem().and_then(|stem| stem.to_str())
    {
        names.push(stem.to_lowercase());
    }
    if let Some(file) = model.hf_file.as_deref()
        && let Some(stem) = Path::new(file).file_stem().and_then(|stem| stem.to_str())
    {
        names.push(stem.to_lowercase());
    }
    names.sort();
    names.dedup();
    names
}

async fn server_props(http_client: &dyn HttpClient, api_url: &str) -> Result<LlamaServerProps> {
    let url = format!("{}/props", native_api_url(api_url));
    let mut response = http_client
        .get(&url, AsyncBody::default(), true)
        .await
        .with_context(|| format!("checking llama.cpp server props at {url}"))?;
    anyhow::ensure!(
        response.status().is_success(),
        "checking llama.cpp server props returned {}",
        response.status()
    );

    let mut body = String::new();
    response
        .body_mut()
        .read_to_string(&mut body)
        .await
        .with_context(|| format!("reading {url}"))?;

    serde_json::from_str(&body).with_context(|| format!("parsing {url} response"))
}

async fn effective_launch_config(
    http_client: &dyn HttpClient,
    mut config: LlamaCppLaunchConfig,
) -> LlamaCppLaunchConfig {
    let Ok(props) = server_props(http_client, &config.api_url).await else {
        return config;
    };
    if let Some(context_window) = props.context_window()
        && context_window != config.context_window
    {
        log::warn!(
            "using running llama.cpp context window {} for model {} instead of configured {}",
            context_window,
            config.model.name,
            config.context_window
        );
        config.context_window = context_window;
    }
    config
}

#[derive(Deserialize)]
struct LlamaServerProps {
    default_generation_settings: Option<LlamaDefaultGenerationSettings>,
    total_slots: Option<u64>,
}

impl LlamaServerProps {
    fn context_window(&self) -> Option<u64> {
        self.default_generation_settings
            .as_ref()
            .and_then(|settings| settings.n_ctx)
    }

    fn matches_config(&self, config: &LlamaCppLaunchConfig) -> bool {
        self.context_window()
            .map_or(true, |n_ctx| n_ctx == config.context_window)
            && self
                .total_slots
                .map_or(true, |slots| slots == LOCAL_LLAMA_SERVER_PARALLEL_SLOTS)
    }

    fn describe(&self) -> String {
        let context = self
            .context_window()
            .map(|context| context.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        let slots = self
            .total_slots
            .map(|slots| slots.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        format!("{context} ctx, {slots} parallel slots")
    }
}

#[derive(Deserialize)]
struct LlamaDefaultGenerationSettings {
    n_ctx: Option<u64>,
}

#[derive(Deserialize)]
struct LlamaModelsResponse {
    #[serde(default)]
    data: Vec<LlamaModelListEntry>,
    #[serde(default)]
    models: Vec<LlamaModelListEntry>,
}

#[derive(Deserialize)]
struct LlamaModelListEntry {
    id: Option<String>,
    name: Option<String>,
    model: Option<String>,
    #[serde(default)]
    aliases: Vec<String>,
}

async fn stream_llama_completion(
    http_client: &dyn HttpClient,
    config: &LlamaCppLaunchConfig,
    request: LanguageModelRequest,
) -> Result<
    BoxStream<'static, Result<LanguageModelCompletionEvent, LanguageModelCompletionError>>,
    LanguageModelCompletionError,
> {
    let prompt = apply_llama_chat_template(http_client, config, &request).await?;
    let native_url = native_api_url(&config.api_url);
    let url = format!("{native_url}/completion");
    let request = LlamaCompletionRequest {
        prompt,
        stream: true,
        n_predict: config
            .model
            .max_output_tokens
            .and_then(|tokens| i64::try_from(tokens).ok())
            .unwrap_or(1024),
        stop: request.stop,
        temperature: request.temperature,
        cache_prompt: true,
        timings_per_token: false,
        return_progress: false,
    };

    let mut response = send_json(http_client, &url, &request).await?;
    if !response.status().is_success() {
        let status = response.status();
        let body = response_body_string(&mut response).await?;
        return Err(map_llama_http_error(status, body));
    }

    let reader = BufReader::new(response.into_body());
    let stream = reader
        .lines()
        .filter_map(|line| async move {
            let line = match line {
                Ok(line) => line,
                Err(error) => {
                    return Some(vec![Err(
                        LanguageModelCompletionError::ApiReadResponseError {
                            provider: LOCAL_LLAMA_PROVIDER_NAME,
                            error,
                        },
                    )]);
                }
            };
            let Some(line) = line
                .strip_prefix("data: ")
                .or_else(|| line.strip_prefix("data:"))
            else {
                return None;
            };
            let line = line.trim();
            if line.is_empty() || line == "[DONE]" {
                return None;
            }

            let chunk = match serde_json::from_str::<LlamaCompletionChunk>(line) {
                Ok(chunk) => chunk,
                Err(error) => {
                    return Some(vec![Err(
                        LanguageModelCompletionError::DeserializeResponse {
                            provider: LOCAL_LLAMA_PROVIDER_NAME,
                            error,
                        },
                    )]);
                }
            };

            let mut events = Vec::new();
            if let Some(content) = chunk.content
                && !content.is_empty()
            {
                events.push(Ok(LanguageModelCompletionEvent::Text(content)));
            }

            if chunk.stop.unwrap_or(false) {
                events.push(Ok(LanguageModelCompletionEvent::UsageUpdate(TokenUsage {
                    input_tokens: chunk.tokens_evaluated.unwrap_or(0),
                    output_tokens: chunk.tokens_predicted.unwrap_or(0),
                    cache_creation_input_tokens: 0,
                    cache_read_input_tokens: chunk.tokens_cached.unwrap_or(0),
                })));

                let stop_reason = match chunk.stop_type.as_deref() {
                    Some("limit") => StopReason::MaxTokens,
                    _ if chunk.truncated.unwrap_or(false) => StopReason::MaxTokens,
                    _ => StopReason::EndTurn,
                };
                events.push(Ok(LanguageModelCompletionEvent::Stop(stop_reason)));
            }

            Some(events)
        })
        .flat_map(futures::stream::iter)
        .boxed();

    Ok(stream)
}

async fn apply_llama_chat_template(
    http_client: &dyn HttpClient,
    config: &LlamaCppLaunchConfig,
    request: &LanguageModelRequest,
) -> Result<String, LanguageModelCompletionError> {
    let native_url = native_api_url(&config.api_url);
    let messages = request
        .messages
        .iter()
        .filter_map(llama_chat_message_from_request)
        .collect::<Vec<_>>();
    let prompt =
        apply_llama_template_to_messages(http_client, &native_url, messages.clone()).await?;
    let prompt_budget = llama_prompt_token_budget(config);

    if llama_prompt_fits(http_client, &native_url, &prompt, prompt_budget).await {
        return Ok(prompt);
    }

    fit_llama_prompt_to_context(http_client, &native_url, messages, prompt_budget)
        .await
        .unwrap_or(Ok(prompt))
}

async fn apply_llama_template_to_messages(
    http_client: &dyn HttpClient,
    native_url: &str,
    messages: Vec<LlamaChatMessage>,
) -> Result<String, LanguageModelCompletionError> {
    let url = format!("{native_url}/apply-template");
    let request = LlamaApplyTemplateRequest {
        messages,
        add_generation_prompt: true,
        chat_template_kwargs: LlamaChatTemplateKwargs {
            enable_thinking: false,
        },
    };
    let mut response = send_json(http_client, &url, &request).await?;
    let status = response.status();
    let body = response_body_string(&mut response).await?;

    if !status.is_success() {
        return Err(map_llama_http_error(status, body));
    }

    let response: LlamaApplyTemplateResponse = serde_json::from_str(&body).map_err(|error| {
        LanguageModelCompletionError::DeserializeResponse {
            provider: LOCAL_LLAMA_PROVIDER_NAME,
            error,
        }
    })?;
    Ok(response.prompt)
}

async fn fit_llama_prompt_to_context(
    http_client: &dyn HttpClient,
    native_url: &str,
    messages: Vec<LlamaChatMessage>,
    prompt_budget: u64,
) -> Option<Result<String, LanguageModelCompletionError>> {
    if messages.len() <= 1 {
        return trim_last_llama_message_to_context(
            http_client,
            native_url,
            messages,
            prompt_budget,
        )
        .await;
    }

    let system_message = messages
        .first()
        .filter(|message| message.role == "system")
        .cloned();
    let first_conversation_index = usize::from(system_message.is_some());
    for start in (first_conversation_index + 1)..messages.len() {
        let mut candidate = Vec::new();
        if let Some(system_message) = system_message.clone() {
            candidate.push(system_message);
        }
        candidate.extend(messages[start..].iter().cloned());
        let prompt = match apply_llama_template_to_messages(
            http_client,
            native_url,
            candidate.clone(),
        )
        .await
        {
            Ok(prompt) => prompt,
            Err(error) => return Some(Err(error)),
        };
        if llama_prompt_fits(http_client, native_url, &prompt, prompt_budget).await {
            log::warn!(
                "trimmed older llama.cpp chat history to fit local context window ({} messages kept)",
                candidate.len()
            );
            return Some(Ok(prompt));
        }
    }

    trim_last_llama_message_to_context(http_client, native_url, messages, prompt_budget).await
}

async fn trim_last_llama_message_to_context(
    http_client: &dyn HttpClient,
    native_url: &str,
    messages: Vec<LlamaChatMessage>,
    prompt_budget: u64,
) -> Option<Result<String, LanguageModelCompletionError>> {
    let last_message = messages.last()?;
    let system_message = messages
        .first()
        .filter(|message| message.role == "system")
        .cloned();
    let mut keep_chars = last_message.content.chars().count().saturating_mul(3) / 4;
    keep_chars = keep_chars.max(512);

    while keep_chars >= 512 {
        let mut candidate = Vec::new();
        if let Some(system_message) = system_message.clone() {
            candidate.push(system_message);
        }
        candidate.push(LlamaChatMessage {
            role: last_message.role.clone(),
            content: tail_content_for_local_context(&last_message.content, keep_chars),
        });
        let prompt =
            match apply_llama_template_to_messages(http_client, native_url, candidate).await {
                Ok(prompt) => prompt,
                Err(error) => return Some(Err(error)),
            };
        if llama_prompt_fits(http_client, native_url, &prompt, prompt_budget).await {
            log::warn!("trimmed oversized llama.cpp prompt tail to fit local context window");
            return Some(Ok(prompt));
        }
        keep_chars /= 2;
    }

    None
}

fn tail_content_for_local_context(content: &str, keep_chars: usize) -> String {
    let total_chars = content.chars().count();
    if total_chars <= keep_chars {
        return content.to_string();
    }

    let tail = content
        .chars()
        .skip(total_chars.saturating_sub(keep_chars))
        .collect::<String>();
    format!("[Earlier local context was omitted to fit this llama.cpp model.]\n{tail}")
}

async fn llama_prompt_fits(
    http_client: &dyn HttpClient,
    native_url: &str,
    prompt: &str,
    prompt_budget: u64,
) -> bool {
    llama_prompt_token_count(http_client, native_url, prompt)
        .await
        .map(|tokens| tokens <= prompt_budget)
        .unwrap_or(true)
}

async fn llama_prompt_token_count(
    http_client: &dyn HttpClient,
    native_url: &str,
    prompt: &str,
) -> Result<u64, LanguageModelCompletionError> {
    let url = format!("{native_url}/tokenize");
    let request = LlamaTokenizeRequest {
        content: prompt.to_string(),
    };
    let mut response = send_json(http_client, &url, &request).await?;
    let status = response.status();
    let body = response_body_string(&mut response).await?;

    if !status.is_success() {
        return Err(map_llama_http_error(status, body));
    }

    let response: LlamaTokenizeResponse = serde_json::from_str(&body).map_err(|error| {
        LanguageModelCompletionError::DeserializeResponse {
            provider: LOCAL_LLAMA_PROVIDER_NAME,
            error,
        }
    })?;
    Ok(response.tokens.len() as u64)
}

fn llama_prompt_token_budget(config: &LlamaCppLaunchConfig) -> u64 {
    let output_tokens = config.model.max_output_tokens.unwrap_or(1024);
    config
        .context_window
        .saturating_sub(output_tokens)
        .saturating_sub(LOCAL_LLAMA_PROMPT_TOKEN_MARGIN)
        .max(config.context_window / 2)
}

async fn warm_up_llama_server(http_client: &dyn HttpClient, config: &LlamaCppLaunchConfig) {
    let native_url = native_api_url(&config.api_url);
    let request = LlamaCompletionRequest {
        prompt: "User: warm up.\nAssistant:".to_string(),
        stream: false,
        n_predict: 1,
        stop: Vec::new(),
        temperature: Some(0.0),
        cache_prompt: false,
        timings_per_token: false,
        return_progress: false,
    };

    match send_json(http_client, &format!("{native_url}/completion"), &request).await {
        Ok(mut response) => {
            let status = response.status();
            if status.is_success() {
                let _ = response_body_string(&mut response).await;
            } else {
                let body = response_body_string(&mut response)
                    .await
                    .unwrap_or_else(|_| String::new());
                log::warn!(
                    "llama.cpp warmup for {} returned {}: {}",
                    config.model.name,
                    status,
                    body
                );
            }
        }
        Err(error) => {
            log::warn!(
                "llama.cpp warmup for {} failed: {}",
                config.model.name,
                error
            );
        }
    }
}

fn llama_chat_message_from_request(
    message: &language_model::LanguageModelRequestMessage,
) -> Option<LlamaChatMessage> {
    let mut content = String::new();
    for part in &message.content {
        match part {
            MessageContent::Text(text) => content.push_str(text),
            MessageContent::Thinking { text, .. } => content.push_str(text),
            MessageContent::ToolResult(tool_result) => {
                content.push_str(&tool_result.text_contents());
            }
            MessageContent::ToolUse(tool_use) => {
                content.push_str(&format!(
                    "\n[tool call: {} {}]\n",
                    tool_use.name, tool_use.raw_input
                ));
            }
            MessageContent::RedactedThinking(_) | MessageContent::Image(_) => {}
        }
    }

    if content.trim().is_empty() {
        return None;
    }

    let role = match message.role {
        Role::User => "user",
        Role::Assistant => "assistant",
        Role::System => "system",
    };

    Some(LlamaChatMessage {
        role: role.to_string(),
        content,
    })
}

async fn send_json<T: Serialize>(
    http_client: &dyn HttpClient,
    url: &str,
    body: &T,
) -> Result<http_client::Response<AsyncBody>, LanguageModelCompletionError> {
    let request = HttpRequest::builder()
        .method(Method::POST)
        .uri(url)
        .header("Content-Type", "application/json")
        .body(AsyncBody::from(serde_json::to_string(body).map_err(
            |error| LanguageModelCompletionError::SerializeRequest {
                provider: LOCAL_LLAMA_PROVIDER_NAME,
                error,
            },
        )?))
        .map_err(|error| LanguageModelCompletionError::BuildRequestBody {
            provider: LOCAL_LLAMA_PROVIDER_NAME,
            error,
        })?;

    http_client
        .send(request)
        .await
        .map_err(|error| LanguageModelCompletionError::HttpSend {
            provider: LOCAL_LLAMA_PROVIDER_NAME,
            error,
        })
}

async fn response_body_string(
    response: &mut http_client::Response<AsyncBody>,
) -> Result<String, LanguageModelCompletionError> {
    let mut body = String::new();
    response
        .body_mut()
        .read_to_string(&mut body)
        .await
        .map_err(|error| LanguageModelCompletionError::ApiReadResponseError {
            provider: LOCAL_LLAMA_PROVIDER_NAME,
            error,
        })?;
    Ok(body)
}

fn map_llama_http_error(status: StatusCode, body: String) -> LanguageModelCompletionError {
    if body.contains("exceed_context_size_error")
        || body.contains("exceeds the available context size")
    {
        return LanguageModelCompletionError::PromptTooLarge { tokens: None };
    }

    LanguageModelCompletionError::from_http_status(LOCAL_LLAMA_PROVIDER_NAME, status, body, None)
}

fn native_api_url(api_url: &str) -> String {
    api_url
        .trim_end_matches('/')
        .strip_suffix("/v1")
        .unwrap_or_else(|| api_url.trim_end_matches('/'))
        .to_string()
}

#[derive(Serialize)]
struct LlamaApplyTemplateRequest {
    messages: Vec<LlamaChatMessage>,
    add_generation_prompt: bool,
    chat_template_kwargs: LlamaChatTemplateKwargs,
}

#[derive(Serialize)]
struct LlamaChatTemplateKwargs {
    enable_thinking: bool,
}

#[derive(Clone, Serialize)]
struct LlamaChatMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct LlamaApplyTemplateResponse {
    prompt: String,
}

#[derive(Serialize)]
struct LlamaTokenizeRequest {
    content: String,
}

#[derive(Deserialize)]
struct LlamaTokenizeResponse {
    tokens: Vec<i64>,
}

#[derive(Serialize)]
struct LlamaCompletionRequest {
    prompt: String,
    stream: bool,
    n_predict: i64,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    stop: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    cache_prompt: bool,
    timings_per_token: bool,
    return_progress: bool,
}

#[derive(Deserialize)]
struct LlamaCompletionChunk {
    content: Option<String>,
    stop: Option<bool>,
    stop_type: Option<String>,
    truncated: Option<bool>,
    tokens_cached: Option<u64>,
    tokens_evaluated: Option<u64>,
    tokens_predicted: Option<u64>,
}

fn add_hugging_face_model_args(command: &mut Command, model: &AvailableModel) -> Result<()> {
    let repo = model
        .hf_repo
        .as_deref()
        .ok_or_else(|| anyhow!("{} has no hf_repo configured", model.name))?;

    if let Some(quant) = model.hf_quant.as_deref() {
        command.arg("-hf").arg(format!("{repo}:{quant}"));
    } else if let Some(file) = model.hf_file.as_deref() {
        command
            .arg("--hf-repo")
            .arg(repo)
            .arg("--hf-file")
            .arg(file);
    } else {
        command.arg("-hf").arg(repo);
    }

    if !model.supports_images {
        command.arg("--no-mmproj");
    }

    Ok(())
}

async fn download_configured_model_file(
    config: &LlamaCppLaunchConfig,
    http_client: &dyn HttpClient,
) -> Result<Option<PathBuf>> {
    let Some(file_name) = config.model.hf_file.as_deref() else {
        return Ok(None);
    };
    let Some(repo) = config.model.hf_repo.as_deref() else {
        anyhow::bail!("{} has no hf_repo configured", config.model.name);
    };

    let target_path = config
        .model
        .local_path
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or_else(|| config.models_dir.join(file_name));
    if target_path.exists() {
        return Ok(Some(target_path));
    }

    let Some(parent) = target_path.parent() else {
        anyhow::bail!("model path has no parent: {}", target_path.display());
    };
    async_fs::create_dir_all(parent)
        .await
        .with_context(|| format!("creating {}", parent.display()))?;

    let temp_path = target_path.with_file_name(format!(
        ".{}.download",
        target_path
            .file_name()
            .map(|file_name| file_name.to_string_lossy())
            .unwrap_or_else(|| config.model.name.clone().into())
    ));
    _ = async_fs::remove_file(&temp_path).await;

    let url = format!("https://huggingface.co/{repo}/resolve/main/{file_name}");
    log::info!(
        "downloading llama.cpp model from {} to {}",
        url,
        target_path.display()
    );
    let mut response = http_client
        .get(&url, AsyncBody::default(), true)
        .await
        .with_context(|| format!("downloading {url}"))?;
    anyhow::ensure!(
        response.status().is_success(),
        "downloading {url} returned {}",
        response.status()
    );

    let mut output = async_fs::File::create(&temp_path)
        .await
        .with_context(|| format!("creating {}", temp_path.display()))?;
    let bytes = futures::io::copy(response.body_mut(), &mut output)
        .await
        .with_context(|| format!("saving {url} to {}", temp_path.display()))?;
    output
        .close()
        .await
        .with_context(|| format!("closing {}", temp_path.display()))?;
    anyhow::ensure!(bytes > 0, "downloaded zero bytes from {url}");

    async_fs::rename(&temp_path, &target_path)
        .await
        .with_context(|| {
            format!(
                "renaming {} to {}",
                temp_path.display(),
                target_path.display()
            )
        })?;

    Ok(Some(target_path))
}

fn resolve_local_model_path(config: &LlamaCppLaunchConfig) -> Option<PathBuf> {
    let mut expected_files = Vec::new();

    if let Some(local_path) = config.model.local_path.as_ref().map(PathBuf::from)
        && local_path.exists()
    {
        return Some(local_path);
    } else if let Some(file_name) = config
        .model
        .local_path
        .as_ref()
        .and_then(|path| Path::new(path).file_name())
        .and_then(|file_name| file_name.to_str())
    {
        expected_files.push(file_name.to_string());
    }

    if let Some(hf_file) = config.model.hf_file.as_ref()
        && !expected_files.contains(hf_file)
    {
        expected_files.push(hf_file.clone());
    }
    if let Some(quant) = config.model.hf_quant.as_ref() {
        let quant_file_name = format!("{}-{quant}.gguf", config.model.name);
        if !expected_files.contains(&quant_file_name) {
            expected_files.push(quant_file_name);
        }
    }

    for dir in candidate_model_dirs(&config.models_dir) {
        for expected_file in &expected_files {
            if let Some(path) = find_gguf_in_dir(&dir, Some(expected_file), &config.model.name, 4) {
                return Some(path);
            }
        }

        if let Some(path) = find_gguf_in_dir(&dir, None, &config.model.name, 4) {
            return Some(path);
        }
    }

    None
}

fn discover_local_gguf_models(
    primary: &Path,
    known_local_paths: &[PathBuf],
    context_window: u64,
) -> Vec<AvailableModel> {
    let known_local_paths = known_local_paths
        .iter()
        .map(normalized_path_key)
        .collect::<Vec<_>>();
    let mut models = BTreeMap::new();

    for dir in candidate_model_dirs(primary) {
        let mut files = Vec::new();
        collect_gguf_files(&dir, 4, &mut files);
        for path in files {
            if known_local_paths
                .iter()
                .any(|known_path| known_path == &normalized_path_key(&path))
            {
                continue;
            }

            let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
                continue;
            };
            let name = stem.to_string();
            let inferred_context_window = infer_gguf_context_window(&path)
                .map(|window| window.min(context_window))
                .unwrap_or(context_window);
            let model = apply_local_model_override(
                AvailableModel {
                    name,
                    display_name: Some(display_name_from_file_stem(stem)),
                    max_tokens: inferred_context_window,
                    max_output_tokens: Some(2048),
                    hf_repo: None,
                    hf_quant: None,
                    hf_file: path
                        .file_name()
                        .and_then(|file_name| file_name.to_str())
                        .map(str::to_owned),
                    local_path: Some(path.to_string_lossy().replace('\\', "/")),
                    supports_tools: false,
                    supports_images: false,
                },
                &path,
            );
            models.entry(model.name.clone()).or_insert(model);
        }
    }

    models.into_values().collect()
}

#[derive(Default, Deserialize)]
struct LocalModelOverride {
    name: Option<String>,
    display_name: Option<String>,
    max_tokens: Option<u64>,
    max_output_tokens: Option<u64>,
    hf_repo: Option<String>,
    hf_quant: Option<String>,
    hf_file: Option<String>,
    local_path: Option<String>,
    supports_tools: Option<bool>,
    supports_images: Option<bool>,
}

fn apply_local_model_override(mut model: AvailableModel, path: &Path) -> AvailableModel {
    let Some(override_settings) = read_local_model_override(path) else {
        return model;
    };

    if let Some(name) = override_settings.name {
        model.name = name;
    }
    if let Some(display_name) = override_settings.display_name {
        model.display_name = Some(display_name);
    }
    if let Some(max_tokens) = override_settings.max_tokens {
        model.max_tokens = max_tokens;
    }
    if let Some(max_output_tokens) = override_settings.max_output_tokens {
        model.max_output_tokens = Some(max_output_tokens);
    }
    if let Some(hf_repo) = override_settings.hf_repo {
        model.hf_repo = Some(hf_repo);
    }
    if let Some(hf_quant) = override_settings.hf_quant {
        model.hf_quant = Some(hf_quant);
    }
    if let Some(hf_file) = override_settings.hf_file {
        model.hf_file = Some(hf_file);
    }
    if let Some(local_path) = override_settings.local_path {
        model.local_path = Some(local_path);
    }
    if let Some(supports_tools) = override_settings.supports_tools {
        model.supports_tools = supports_tools;
    }
    if let Some(supports_images) = override_settings.supports_images {
        model.supports_images = supports_images;
    }

    model
}

fn read_local_model_override(path: &Path) -> Option<LocalModelOverride> {
    for sidecar_path in [
        path.with_extension("gguf.json"),
        path.with_extension("json"),
    ] {
        if !sidecar_path.exists() {
            continue;
        }
        match std::fs::read_to_string(&sidecar_path)
            .with_context(|| format!("reading {}", sidecar_path.display()))
            .and_then(|content| {
                serde_json::from_str(&content)
                    .with_context(|| format!("parsing {}", sidecar_path.display()))
            }) {
            Ok(override_settings) => return Some(override_settings),
            Err(error) => {
                log::warn!("failed to read llama.cpp local model override: {error:#}");
            }
        }
    }

    None
}

const GGUF_METADATA_READ_LIMIT: u64 = 1024 * 1024;
const GGUF_TYPE_UINT8: u32 = 0;
const GGUF_TYPE_INT8: u32 = 1;
const GGUF_TYPE_UINT16: u32 = 2;
const GGUF_TYPE_INT16: u32 = 3;
const GGUF_TYPE_UINT32: u32 = 4;
const GGUF_TYPE_INT32: u32 = 5;
const GGUF_TYPE_FLOAT32: u32 = 6;
const GGUF_TYPE_BOOL: u32 = 7;
const GGUF_TYPE_STRING: u32 = 8;
const GGUF_TYPE_ARRAY: u32 = 9;
const GGUF_TYPE_UINT64: u32 = 10;
const GGUF_TYPE_INT64: u32 = 11;
const GGUF_TYPE_FLOAT64: u32 = 12;

fn infer_gguf_context_window(path: &Path) -> Option<u64> {
    let mut file = File::open(path).ok()?;
    let mut buffer = vec![0; GGUF_METADATA_READ_LIMIT as usize];
    let len = file.read(&mut buffer).ok()?;
    buffer.truncate(len);

    let mut cursor = Cursor::new(buffer.as_slice());
    let mut magic = [0; 4];
    cursor.read_exact(&mut magic).ok()?;
    if &magic != b"GGUF" {
        return None;
    }

    let _version = read_gguf_u32(&mut cursor)?;
    let _tensor_count = read_gguf_u64(&mut cursor)?;
    let metadata_count = read_gguf_u64(&mut cursor)?;
    for _ in 0..metadata_count.min(4096) {
        let key = read_gguf_string(&mut cursor)?;
        let value_type = read_gguf_u32(&mut cursor)?;
        if key.ends_with(".context_length") || key == "context_length" {
            return read_gguf_numeric_value(value_type, &mut cursor);
        }
        skip_gguf_value(value_type, &mut cursor)?;
    }

    None
}

fn read_gguf_u32(cursor: &mut Cursor<&[u8]>) -> Option<u32> {
    let mut bytes = [0; 4];
    cursor.read_exact(&mut bytes).ok()?;
    Some(u32::from_le_bytes(bytes))
}

fn read_gguf_u64(cursor: &mut Cursor<&[u8]>) -> Option<u64> {
    let mut bytes = [0; 8];
    cursor.read_exact(&mut bytes).ok()?;
    Some(u64::from_le_bytes(bytes))
}

fn read_gguf_string(cursor: &mut Cursor<&[u8]>) -> Option<String> {
    let len = usize::try_from(read_gguf_u64(cursor)?).ok()?;
    let position = usize::try_from(cursor.position()).ok()?;
    let end = position.checked_add(len)?;
    if end > cursor.get_ref().len() {
        return None;
    }
    let value = String::from_utf8_lossy(&cursor.get_ref()[position..end]).into_owned();
    cursor.set_position(end as u64);
    Some(value)
}

fn read_gguf_numeric_value(value_type: u32, cursor: &mut Cursor<&[u8]>) -> Option<u64> {
    match value_type {
        GGUF_TYPE_UINT8 => {
            let mut bytes = [0; 1];
            cursor.read_exact(&mut bytes).ok()?;
            Some(bytes[0] as u64)
        }
        GGUF_TYPE_INT8 => {
            let mut bytes = [0; 1];
            cursor.read_exact(&mut bytes).ok()?;
            Some(i8::from_le_bytes(bytes).max(0) as u64)
        }
        GGUF_TYPE_UINT16 => {
            let mut bytes = [0; 2];
            cursor.read_exact(&mut bytes).ok()?;
            Some(u16::from_le_bytes(bytes) as u64)
        }
        GGUF_TYPE_INT16 => {
            let mut bytes = [0; 2];
            cursor.read_exact(&mut bytes).ok()?;
            Some(i16::from_le_bytes(bytes).max(0) as u64)
        }
        GGUF_TYPE_UINT32 => {
            let mut bytes = [0; 4];
            cursor.read_exact(&mut bytes).ok()?;
            Some(u32::from_le_bytes(bytes) as u64)
        }
        GGUF_TYPE_INT32 => {
            let mut bytes = [0; 4];
            cursor.read_exact(&mut bytes).ok()?;
            Some(i32::from_le_bytes(bytes).max(0) as u64)
        }
        GGUF_TYPE_UINT64 => read_gguf_u64(cursor),
        GGUF_TYPE_INT64 => {
            let mut bytes = [0; 8];
            cursor.read_exact(&mut bytes).ok()?;
            Some(i64::from_le_bytes(bytes).max(0) as u64)
        }
        _ => None,
    }
}

fn skip_gguf_value(value_type: u32, cursor: &mut Cursor<&[u8]>) -> Option<()> {
    match value_type {
        GGUF_TYPE_UINT8 | GGUF_TYPE_INT8 | GGUF_TYPE_BOOL => skip_gguf_bytes(cursor, 1),
        GGUF_TYPE_UINT16 | GGUF_TYPE_INT16 => skip_gguf_bytes(cursor, 2),
        GGUF_TYPE_UINT32 | GGUF_TYPE_INT32 | GGUF_TYPE_FLOAT32 => skip_gguf_bytes(cursor, 4),
        GGUF_TYPE_UINT64 | GGUF_TYPE_INT64 | GGUF_TYPE_FLOAT64 => skip_gguf_bytes(cursor, 8),
        GGUF_TYPE_STRING => {
            read_gguf_string(cursor)?;
            Some(())
        }
        GGUF_TYPE_ARRAY => {
            let element_type = read_gguf_u32(cursor)?;
            let len = read_gguf_u64(cursor)?;
            if let Some(element_size) = fixed_gguf_value_size(element_type) {
                let byte_len = usize::try_from(len).ok()?.checked_mul(element_size)?;
                return skip_gguf_bytes(cursor, byte_len);
            }
            None
        }
        _ => None,
    }
}

fn fixed_gguf_value_size(value_type: u32) -> Option<usize> {
    match value_type {
        GGUF_TYPE_UINT8 | GGUF_TYPE_INT8 | GGUF_TYPE_BOOL => Some(1),
        GGUF_TYPE_UINT16 | GGUF_TYPE_INT16 => Some(2),
        GGUF_TYPE_UINT32 | GGUF_TYPE_INT32 | GGUF_TYPE_FLOAT32 => Some(4),
        GGUF_TYPE_UINT64 | GGUF_TYPE_INT64 | GGUF_TYPE_FLOAT64 => Some(8),
        _ => None,
    }
}

fn skip_gguf_bytes(cursor: &mut Cursor<&[u8]>, count: usize) -> Option<()> {
    let position = usize::try_from(cursor.position()).ok()?;
    let end = position.checked_add(count)?;
    if end > cursor.get_ref().len() {
        return None;
    }
    cursor.set_position(end as u64);
    Some(())
}

fn collect_gguf_files(dir: &Path, max_depth: usize, files: &mut Vec<PathBuf>) {
    if max_depth == 0 || !dir.is_dir() {
        return;
    }

    let Ok(read_dir) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in read_dir.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_gguf_files(&path, max_depth - 1, files);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("gguf") {
            files.push(path);
        }
    }
}

fn display_name_from_file_stem(stem: &str) -> String {
    stem.replace(['_', '-'], " ")
}

fn normalized_path_key(path: &PathBuf) -> String {
    path.to_string_lossy().replace('\\', "/").to_lowercase()
}

fn candidate_model_dirs(primary: &Path) -> Vec<PathBuf> {
    let mut dirs = vec![primary.to_path_buf()];
    if cfg!(target_os = "windows") && Path::new("G:/").exists() {
        for path in [
            "G:/Dx/code/models",
            "G:/Dx/code/models/llama.cpp",
            "G:/Models",
            "G:/models",
            "G:/AI/models",
        ] {
            let path = PathBuf::from(path);
            if path != primary && path.exists() {
                dirs.push(path);
            }
        }
    }
    dirs
}

fn find_gguf_in_dir(
    dir: &Path,
    expected_file: Option<&str>,
    model_name: &str,
    max_depth: usize,
) -> Option<PathBuf> {
    if max_depth == 0 || !dir.is_dir() {
        return None;
    }

    let expected_file = expected_file.map(|file| file.to_lowercase());
    let model_tokens = model_name
        .to_lowercase()
        .split(['-', '_', '.', ' '])
        .filter(|token| !token.is_empty() && *token != "gguf")
        .map(str::to_owned)
        .collect::<Vec<_>>();

    let read_dir = std::fs::read_dir(dir).ok()?;
    for entry in read_dir.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(path) =
                find_gguf_in_dir(&path, expected_file.as_deref(), model_name, max_depth - 1)
            {
                return Some(path);
            }
            continue;
        }

        if path.extension().and_then(|ext| ext.to_str()) != Some("gguf") {
            continue;
        }

        let file_name = path.file_name()?.to_string_lossy().to_lowercase();
        if expected_file
            .as_ref()
            .is_some_and(|expected| &file_name == expected)
            || model_tokens.iter().all(|token| file_name.contains(token))
        {
            return Some(path);
        }
    }

    None
}

async fn download_llama_server(
    http_client: Arc<dyn HttpClient>,
    tools_dir: &Path,
) -> Result<PathBuf> {
    #[cfg(target_family = "wasm")]
    {
        let _ = http_client;
        let _ = tools_dir;
        anyhow::bail!("automatic llama.cpp server installation is not supported on wasm");
    }

    #[cfg(not(target_family = "wasm"))]
    {
        use http_client::{github, github_download};

        std::fs::create_dir_all(tools_dir)
            .with_context(|| format!("creating {}", tools_dir.display()))?;
        let release =
            github::latest_github_release("ggml-org/llama.cpp", true, false, http_client.clone())
                .await?;
        let (asset, asset_kind) = select_llama_release_asset(&release.assets)
            .ok_or_else(|| anyhow!("no llama.cpp release asset matched this OS"))?;
        let destination = tools_dir.join(&release.tag_name);
        github_download::download_server_binary(
            http_client.as_ref(),
            &asset.browser_download_url,
            asset.digest.as_deref(),
            &destination,
            asset_kind,
        )
        .await?;
        find_server_binary(&destination)
            .ok_or_else(|| anyhow!("downloaded llama.cpp release did not contain llama-server"))
    }
}

#[cfg(not(target_family = "wasm"))]
fn select_llama_release_asset(
    assets: &[http_client::github::GithubReleaseAsset],
) -> Option<(
    &http_client::github::GithubReleaseAsset,
    http_client::github::AssetKind,
)> {
    fn matches(name: &str, required: &[&str], forbidden: &[&str]) -> bool {
        required.iter().all(|part| name.contains(part))
            && forbidden.iter().all(|part| !name.contains(part))
    }

    let forbidden_accelerators = [
        "cuda", "cudart", "vulkan", "sycl", "hip", "rocm", "openvino",
    ];
    let candidates: &[&[&str]] = if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
        &[
            &["bin-win-cpu-x64"],
            &["bin-win-avx2-x64"],
            &["bin-win", "x64"],
        ]
    } else if cfg!(all(target_os = "windows", target_arch = "aarch64")) {
        &[&["bin-win-cpu-arm64"], &["bin-win", "arm64"]]
    } else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        &[&["bin-ubuntu-cpu-x64"], &["bin-ubuntu-x64"]]
    } else if cfg!(all(target_os = "linux", target_arch = "aarch64")) {
        &[&["bin-ubuntu-cpu-arm64"], &["bin-ubuntu-arm64"]]
    } else {
        &[]
    };

    for required in candidates {
        if let Some(asset) = assets.iter().find(|asset| {
            let name = asset.name.to_lowercase();
            (name.ends_with(".zip") || name.ends_with(".tar.gz"))
                && matches(&name, required, &forbidden_accelerators)
        }) {
            let name = asset.name.to_lowercase();
            let kind = if name.ends_with(".tar.gz") {
                http_client::github::AssetKind::TarGz
            } else {
                http_client::github::AssetKind::Zip
            };
            return Some((asset, kind));
        }
    }

    None
}

fn find_server_binary(root: &Path) -> Option<PathBuf> {
    find_named_binary(root, server_binary_name(), 5)
}

fn find_named_binary(root: &Path, name: &str, max_depth: usize) -> Option<PathBuf> {
    if max_depth == 0 || !root.is_dir() {
        return None;
    }

    for entry in std::fs::read_dir(root).ok()?.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(path) = find_named_binary(&path, name, max_depth - 1) {
                return Some(path);
            }
        } else if path.file_name().and_then(|name| name.to_str()) == Some(name) {
            return Some(path);
        }
    }

    None
}

fn find_server_on_path() -> Option<PathBuf> {
    env::var_os("PATH").and_then(|paths| {
        env::split_paths(&paths)
            .map(|path| path.join(server_binary_name()))
            .find(|path| path.exists())
    })
}

fn server_binary_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "llama-server.exe"
    } else {
        "llama-server"
    }
}

fn stop_existing_local_llama_server_for_switch(
    config: &LlamaCppLaunchConfig,
    server_path: &Path,
) -> Result<bool> {
    let process_refresh_kind = sysinfo::ProcessRefreshKind::nothing()
        .with_cmd(sysinfo::UpdateKind::Always)
        .with_exe(sysinfo::UpdateKind::Always);
    let system = sysinfo::System::new_with_specifics(
        sysinfo::RefreshKind::nothing().with_processes(process_refresh_kind),
    );
    let mut stopped_any = false;

    for process in system.processes().values() {
        if !is_llama_server_process_name(process.name()) {
            continue;
        }

        let Some(exe) = process.exe() else {
            continue;
        };

        if !is_local_managed_llama_server(exe, server_path, &config.tools_dir) {
            continue;
        }

        log::info!(
            "stopping local llama-server process {} at {} before switching to {}",
            process.pid(),
            exe.display(),
            config.model.name
        );
        stopped_any |= process.kill();
    }

    Ok(stopped_any)
}

fn is_llama_server_process_name(name: &std::ffi::OsStr) -> bool {
    name.to_string_lossy()
        .eq_ignore_ascii_case(server_binary_name())
}

fn is_local_managed_llama_server(exe: &Path, server_path: &Path, tools_dir: &Path) -> bool {
    let exe = comparable_path(exe);
    let server_path = comparable_path(server_path);
    let tools_dir = comparable_path(tools_dir);

    exe == server_path || exe.starts_with(&format!("{tools_dir}/"))
}

fn comparable_path(path: &Path) -> String {
    path.canonicalize()
        .unwrap_or_else(|_| path.to_path_buf())
        .to_string_lossy()
        .replace('\\', "/")
        .trim_end_matches('/')
        .to_ascii_lowercase()
}

fn is_low_spec_system() -> bool {
    let system = sysinfo::System::new_all();
    system.total_memory() < LOW_SPEC_MEMORY_THRESHOLD_BYTES
}

fn default_models_dir() -> PathBuf {
    if cfg!(target_os = "windows") && Path::new("G:/").exists() {
        PathBuf::from("G:/Dx/code/models/llama.cpp")
    } else {
        default_user_data_dir().join("models").join("llama.cpp")
    }
}

fn default_cache_dir() -> PathBuf {
    if cfg!(target_os = "windows") && Path::new("G:/").exists() {
        PathBuf::from("G:/Dx/code/models/huggingface")
    } else {
        default_user_data_dir().join("models").join("huggingface")
    }
}

fn default_tools_dir() -> PathBuf {
    if cfg!(target_os = "windows") && Path::new("G:/").exists() {
        PathBuf::from("G:/Dx/code/tools/llama.cpp")
    } else {
        default_user_data_dir().join("tools").join("llama.cpp")
    }
}

fn default_user_data_dir() -> PathBuf {
    env::var_os("LOCALAPPDATA")
        .or_else(|| env::var_os("HOME"))
        .map(PathBuf::from)
        .unwrap_or_else(env::temp_dir)
        .join("Zed")
}

struct LocalLlamaCppConfigurationView {
    state: Entity<LocalLlamaCppState>,
    http_client: Arc<dyn HttpClient>,
    load_task: Option<Task<()>>,
}

impl LocalLlamaCppConfigurationView {
    fn new(state: Entity<LocalLlamaCppState>, http_client: Arc<dyn HttpClient>, cx: &mut Context<Self>) -> Self {
        cx.observe(&state, |_, _, cx| cx.notify()).detach();
        Self {
            state,
            http_client,
            load_task: None,
        }
    }

    fn connect(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        let (runtime, config) = self.state.read_with(cx, |state, _| {
            let model = state.selected_model();
            (state.runtime.clone(), state.launch_config(&model))
        });
        let http_client = self.http_client.clone();

        self.load_task = Some(cx.spawn(async move |this, cx| {
            let result = runtime.ensure_server(config, http_client).await;
            if let Err(error) = result {
                log::error!("failed to start llama.cpp: {error:?}");
            }
            this.update(cx, |this, cx| {
                this.load_task = None;
                cx.notify();
            })
            .log_err();
        }));
    }
}

impl Render for LocalLlamaCppConfigurationView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.state.read(cx);
        let selected_model = state.selected_model();
        let config = state.launch_config(&selected_model);
        let is_loading = self.load_task.is_some();

        v_flex()
            .gap_2()
            .child(Label::new("Run the agent with a local llama.cpp server."))
            .child(
                ConfiguredApiCard::new(format!(
                    "{} at {}",
                    selected_model
                        .display_name
                        .clone()
                        .unwrap_or_else(|| selected_model.name.clone()),
                    config.api_url
                ))
                .disabled(true),
            )
            .child(
                List::new()
                    .child(ListBulletItem::new(format!(
                        "Models and Hugging Face cache use {}",
                        config.cache_dir.display()
                    )))
                    .child(ListBulletItem::new(format!(
                        "llama.cpp tools use {}",
                        config.tools_dir.display()
                    )))
                    .child(ListBulletItem::new(
                        "Gemma 4 E4B is the normal local default; Qwen 3.5 0.8B stays available as the low-spec fallback with a larger configured context.",
                    ))
                    .child(ListBulletItem::new(format!(
                        "Drop GGUF files into {} and optionally add a matching .gguf.json sidecar for custom display names or limits.",
                        config.models_dir.display()
                    ))),
            )
            .child(
                h_flex()
                    .gap_2()
                    .child(
                        Button::new(
                            "connect-llama-cpp",
                            if is_loading {
                                "Starting..."
                            } else {
                                "Start Local Model"
                            },
                        )
                        .disabled(is_loading)
                        .start_icon(Icon::new(IconName::PlayFilled).size(IconSize::XSmall))
                        .on_click(cx.listener(|this, _, window, cx| this.connect(window, cx))),
                    )
                    .child(
                        ButtonLike::new("llama-cpp-status")
                            .disabled(true)
                            .child(
                                h_flex()
                                    .gap_2()
                                    .child(Icon::new(IconName::Check).color(Color::Success))
                                    .child(Label::new("Local Provider"))
                                    .into_any_element(),
                            ),
                    ),
            )
    }
}

pub struct LlamaCppLanguageModelProvider {
    http_client: Arc<dyn HttpClient>,
    state: Entity<State>,
    /// Live capabilities shared with the agent's models (see [`LiveCapabilities`]).
    capability_cells: CapabilityCells,
    /// Live model-load progress shared with the models (see [`LoadingProgress`]).
    loading_progress: LoadingProgress,
}

pub struct State {
    api_key_state: ApiKeyState,
    credentials_provider: Arc<dyn CredentialsProvider>,
    http_client: Arc<dyn HttpClient>,
    fetched_models: Vec<llama_cpp::Model>,
    fetch_model_task: Option<Task<Result<()>>>,
    /// Router-mode task on `/models/sse`; re-runs discovery as models load/unload.
    model_event_task: Option<Task<()>>,
    /// Same `Arc` as the provider's; re-discovery keeps these cells in sync.
    capability_cells: CapabilityCells,
    /// Same `Arc` as the provider's; the event stream updates it as a model loads.
    loading_progress: LoadingProgress,
}

impl State {
    fn is_authenticated(&self) -> bool {
        !self.fetched_models.is_empty()
    }

    fn set_api_key(&mut self, api_key: Option<String>, cx: &mut Context<Self>) -> Task<Result<()>> {
        let credentials_provider = self.credentials_provider.clone();
        let api_url = LlamaCppLanguageModelProvider::api_url(cx);
        let task = self.api_key_state.store(
            api_url,
            api_key,
            |this| &mut this.api_key_state,
            credentials_provider,
            cx,
        );

        self.fetched_models.clear();
        // Drop the event stream so it reconnects with the new key (re-fetch
        // below restarts it).
        self.model_event_task = None;
        write_recover(&self.loading_progress).clear();
        cx.spawn(async move |this, cx| {
            let result = task.await;
            this.update(cx, |this, cx| this.restart_fetch_models_task(cx))
                .ok();
            result
        })
    }

    fn authenticate(&mut self, cx: &mut Context<Self>) -> Task<Result<(), AuthenticateError>> {
        let credentials_provider = self.credentials_provider.clone();
        let api_url = LlamaCppLanguageModelProvider::api_url(cx);
        let load_key_task = self.api_key_state.load_if_needed(
            api_url,
            |this| &mut this.api_key_state,
            credentials_provider,
            cx,
        );

        if self.is_authenticated() {
            return Task::ready(Ok(()));
        }

        cx.spawn(async move |this, cx| {
            match load_key_task.await {
                Ok(()) | Err(AuthenticateError::CredentialsNotFound) => {}
                Err(error) => {
                    log::warn!("failed to load llama.cpp API key: {error}");
                }
            }
            let fetch_models_task = this.update(cx, |this, cx| this.fetch_models(cx))?;
            match fetch_models_task.await {
                Ok(()) => Ok(()),
                Err(err) => {
                    // A refused connection means the server isn't running yet, not an error.
                    let connection_refused = err.chain().any(|cause| {
                        cause
                            .downcast_ref::<std::io::Error>()
                            .is_some_and(|io_err| {
                                io_err.kind() == std::io::ErrorKind::ConnectionRefused
                            })
                    });
                    if connection_refused {
                        Err(AuthenticateError::ConnectionRefused)
                    } else {
                        Err(AuthenticateError::Other(err))
                    }
                }
            }
        })
    }

    fn fetch_models(&mut self, cx: &mut Context<Self>) -> Task<Result<()>> {
        let http_client = Arc::clone(&self.http_client);
        let settings = LlamaCppLanguageModelProvider::settings(cx);
        let api_url = LlamaCppLanguageModelProvider::api_url(cx);
        let api_key = self.api_key_state.key(&api_url);
        let extra_headers = settings.custom_headers.clone();

        cx.spawn(async move |this, cx| {
            let entries = get_models(
                http_client.as_ref(),
                &api_url,
                api_key.as_deref(),
                &extra_headers,
            )
            .await?;

            let is_router = entries.iter().any(ModelEntry::is_router_entry);

            // Models the server reports as loading, used below to prune stale
            // progress labels by reconciling against the live listing (a preempted
            // load or missed SSE event can skip the terminal event).
            let loading_ids: HashSet<String> = entries
                .iter()
                .filter(|entry| entry.is_loading())
                .map(|entry| entry.id.clone())
                .collect();

            let models: Vec<llama_cpp::Model> = if is_router {
                // Router mode: metadata comes from `/v1/models`. We probe
                // `/props` only for loaded models so listing never triggers a
                // load; unloaded models use the listing's hints and overrides.
                let tasks = entries.into_iter().map(|entry| {
                    let http_client = Arc::clone(&http_client);
                    let api_url = api_url.clone();
                    let api_key = api_key.clone();
                    let extra_headers = extra_headers.clone();
                    async move {
                        let props = if entry.is_loaded() {
                            get_props(
                                http_client.as_ref(),
                                &api_url,
                                api_key.as_deref(),
                                Some(&entry.id),
                                &extra_headers,
                            )
                            .await
                            .log_err()
                        } else {
                            None
                        };
                        model_from_entry(&entry, props.as_ref())
                    }
                });
                futures::stream::iter(tasks)
                    .buffer_unordered(5)
                    .collect()
                    .await
            } else {
                // Single-model mode: one `/props` call describes the loaded model.
                let props = get_props(
                    http_client.as_ref(),
                    &api_url,
                    api_key.as_deref(),
                    None,
                    &extra_headers,
                )
                .await
                .log_err();
                entries
                    .iter()
                    .map(|entry| model_from_entry(entry, props.as_ref()))
                    .collect()
            };

            this.update(cx, |this, cx| {
                this.fetched_models = models;
                let effective = compute_effective_models(
                    &this.fetched_models,
                    LlamaCppLanguageModelProvider::settings(cx),
                );
                sync_capability_cells(&this.capability_cells, &effective);
                // Drop progress labels for models no longer loading, so a stale
                // "Loading …" can't stick after a preempted load or missed event.
                write_recover(&this.loading_progress).retain(|id, _| loading_ids.contains(id));
                // Router mode loads models on demand: subscribe so capabilities
                // self-correct as they load/unload. Start it once (events trigger
                // re-discovery, not a re-spawn); single-model mode needs no stream.
                if is_router {
                    if this.model_event_task.is_none() {
                        this.start_model_event_stream(cx);
                    }
                } else {
                    this.model_event_task = None;
                }
                cx.notify();
            })
        })
    }

    /// Subscribes to `/models/sse` and re-runs discovery as models load, unload,
    /// or the list changes, so capabilities stay current. Reconnects if the stream
    /// drops; on builds without `/models/sse` the refresh is simply skipped.
    fn start_model_event_stream(&mut self, cx: &mut Context<Self>) {
        let http_client = Arc::clone(&self.http_client);
        let api_url = LlamaCppLanguageModelProvider::api_url(cx);
        let api_key = self.api_key_state.key(&api_url);
        let extra_headers = LlamaCppLanguageModelProvider::settings(cx)
            .custom_headers
            .clone();

        self.model_event_task = Some(cx.spawn(async move |this, cx| {
            loop {
                match stream_model_events(
                    http_client.as_ref(),
                    &api_url,
                    api_key.as_deref(),
                    &extra_headers,
                )
                .await
                {
                    Ok(mut events) => {
                        while let Some(event) = events.next().await {
                            let Some(event) = event.log_err() else {
                                continue;
                            };
                            if let Some(exit_code) = event.load_failure() {
                                log::error!(
                                    "llama.cpp model {} failed to load (exit code {exit_code})",
                                    event.model
                                );
                            }
                            // Loading-progress tick: record it for the selector (no
                            // re-discovery). `cx.notify()` drives `ProviderStateChanged`.
                            if let Some(progress) = event.load_progress() {
                                let label = SharedString::from(progress.progress_label());
                                if this
                                    .update(cx, |this, cx| {
                                        write_recover(&this.loading_progress)
                                            .insert(event.model.clone(), label);
                                        cx.notify();
                                    })
                                    .is_err()
                                {
                                    return;
                                }
                                continue;
                            }
                            if !event.changes_model_state() {
                                continue;
                            }
                            // Terminal load/unload (or list change): drop the
                            // progress label and re-discover to refresh capabilities.
                            if this
                                .update(cx, |this, cx| {
                                    write_recover(&this.loading_progress).remove(&event.model);
                                    this.restart_fetch_models_task(cx);
                                })
                                .is_err()
                            {
                                return;
                            }
                        }
                    }
                    // Endpoint missing (older build) or connection failed; retry after a backoff.
                    Err(error) => {
                        log::warn!("llama.cpp model event stream unavailable: {error:#}");
                    }
                }

                cx.background_executor()
                    .timer(MODEL_EVENT_RECONNECT_INTERVAL)
                    .await;
                if this.update(cx, |_, _| ()).is_err() {
                    return;
                }
            }
        }));
    }

    fn restart_fetch_models_task(&mut self, cx: &mut Context<Self>) {
        let task = self.fetch_models(cx);
        self.fetch_model_task.replace(task);
    }
}

/// Capabilities that only become known once a router model loads. `LanguageModel`'s
/// methods take no `cx`, yet the agent reads them live each turn, so we share
/// them through a map that re-discovery updates — an already-selected model
/// picks up real values without re-selection. Image support is available from
/// `/v1/models` hints, so it stays a field.
#[derive(Clone, Copy, Debug, PartialEq)]
struct LiveCapabilities {
    max_tokens: u64,
    supports_tools: bool,
    supports_thinking: bool,
}

impl LiveCapabilities {
    fn of(model: &llama_cpp::Model) -> Self {
        Self {
            max_tokens: model.max_tokens,
            supports_tools: model.supports_tools,
            supports_thinking: model.supports_thinking,
        }
    }
}

/// Live capabilities keyed by model name, shared by the provider and its models.
type CapabilityCells = Arc<RwLock<HashMap<String, LiveCapabilities>>>;

/// Model name → load-status label (e.g. `"Loading weights 42%"`) while a router
/// model loads, shared so the model selector can show progress. Absent once loaded.
type LoadingProgress = Arc<RwLock<HashMap<String, SharedString>>>;

/// Locks for reading, recovering instead of panicking on a poisoned lock. The
/// critical sections are infallible map ops, so poisoning is unreachable anyway.
fn read_recover<T>(lock: &RwLock<T>) -> RwLockReadGuard<'_, T> {
    lock.read().unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Locks for writing; see [`read_recover`].
fn write_recover<T>(lock: &RwLock<T>) -> RwLockWriteGuard<'_, T> {
    lock.write()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// The exact set of models `provided_models` exposes: discovery merged with the
/// `available_models` and `context_window` overrides. Shared with re-discovery.
fn compute_effective_models(
    fetched_models: &[llama_cpp::Model],
    settings: &LlamaCppSettings,
) -> HashMap<String, llama_cpp::Model> {
    let mut models: HashMap<String, llama_cpp::Model> = HashMap::default();
    if settings.auto_discover {
        for model in fetched_models {
            let mut model = model.clone();
            if let Some(context_window) = settings.remote_context_window {
                model.max_tokens = context_window;
            }
            models.insert(model.name.clone(), model);
        }
    }
    merge_settings_into_models(
        &mut models,
        &settings.available_models,
        settings.remote_context_window,
    );
    models
}

/// Updates the shared capability map from the effective models, so a model held
/// by an open conversation observes the new values (it reads the map by name).
fn sync_capability_cells(cells: &CapabilityCells, effective: &HashMap<String, llama_cpp::Model>) {
    let mut cells = write_recover(cells);
    for model in effective.values() {
        cells.insert(model.name.clone(), LiveCapabilities::of(model));
    }
}

/// Builds a model from a `/v1/models` entry, refined by `/props` when the model
/// is loaded. An unloaded router model can't be probed, so we assume optimistic
/// capabilities and let re-discovery refine them on load.
fn model_from_entry(entry: &ModelEntry, props: Option<&Props>) -> llama_cpp::Model {
    let max_tokens = props
        .and_then(Props::context_length)
        .or_else(|| entry.meta.as_ref().and_then(|meta| meta.n_ctx))
        .or_else(|| entry.meta.as_ref().and_then(|meta| meta.n_ctx_train))
        .unwrap_or(ASSUMED_UNLOADED_CONTEXT);
    // Trust `/props` when present. Without it, assume tools for an unloaded model
    // (re-discovery corrects on load) but not for a loaded model whose probe failed.
    let supports_tools = match props {
        Some(props) => props.supports_tools(),
        None => !entry.is_loaded(),
    };
    let supports_images = props.is_some_and(Props::supports_images) || entry.supports_images_hint();
    let supports_thinking = props.is_some_and(Props::supports_thinking);

    llama_cpp::Model::new(
        &entry.id,
        Some(&display_name_for(&entry.id)),
        Some(max_tokens),
        supports_tools,
        supports_images,
        supports_thinking,
    )
}

/// Friendly display name from a model id, which is often a `.gguf` file path.
fn display_name_for(id: &str) -> String {
    let base = id.rsplit(['/', '\\']).next().unwrap_or(id);
    base.strip_suffix(".gguf").unwrap_or(base).to_string()
}

fn telemetry_id_for(id: &str) -> String {
    format!("{PROVIDER_ID}/{}", display_name_for(id))
}

impl LlamaCppLanguageModelProvider {
    pub fn new(
        http_client: Arc<dyn HttpClient>,
        credentials_provider: Arc<dyn CredentialsProvider>,
        cx: &mut App,
    ) -> Self {
        let capability_cells: CapabilityCells = Arc::new(RwLock::new(HashMap::default()));
        let loading_progress: LoadingProgress = Arc::new(RwLock::new(HashMap::default()));
        let this = Self {
            http_client: http_client.clone(),
            capability_cells: capability_cells.clone(),
            loading_progress: loading_progress.clone(),
            state: cx.new(|cx| {
                cx.observe_global::<SettingsStore>({
                    let mut last_settings = LlamaCppLanguageModelProvider::settings(cx).clone();
                    move |this: &mut State, cx| {
                        let current_settings = LlamaCppLanguageModelProvider::settings(cx);
                        let settings_changed = current_settings != &last_settings;
                        if settings_changed {
                            let url_changed = last_settings.api_url != current_settings.api_url;
                            last_settings = current_settings.clone();
                            if url_changed {
                                let credentials_provider = this.credentials_provider.clone();
                                let api_url = Self::api_url(cx);
                                this.api_key_state.handle_url_change(
                                    api_url,
                                    |this| &mut this.api_key_state,
                                    credentials_provider,
                                    cx,
                                );
                                this.fetched_models.clear();
                                // Drop the event stream so it reconnects against
                                // the new URL (re-auth below restarts it).
                                this.model_event_task = None;
                                write_recover(&this.loading_progress).clear();
                                this.authenticate(cx).detach();
                            }
                            cx.notify();
                        }
                    }
                })
                .detach();

                State {
                    http_client,
                    fetched_models: Default::default(),
                    fetch_model_task: None,
                    model_event_task: None,
                    capability_cells,
                    loading_progress,
                    api_key_state: ApiKeyState::new(Self::api_url(cx), (*API_KEY_ENV_VAR).clone()),
                    credentials_provider,
                }
            }),
        };
        // Discover eagerly so a running server is picked up without opening settings.
        this.state
            .update(cx, |state, cx| state.restart_fetch_models_task(cx));
        this
    }

    fn settings(cx: &App) -> &LlamaCppSettings {
        &AllLanguageModelSettings::get_global(cx).llama_cpp
    }

    fn api_url(cx: &App) -> SharedString {
        let api_url = &Self::settings(cx).api_url;
        if api_url.is_empty() {
            LLAMA_CPP_API_URL.into()
        } else {
            SharedString::new(api_url.as_str())
        }
    }

    fn has_custom_url(cx: &App) -> bool {
        Self::settings(cx).api_url != LLAMA_CPP_API_URL
    }
}

impl LanguageModelProviderState for LlamaCppLanguageModelProvider {
    type ObservableEntity = State;

    fn observable_entity(&self) -> Option<Entity<Self::ObservableEntity>> {
        Some(self.state.clone())
    }
}

impl LanguageModelProvider for LlamaCppLanguageModelProvider {
    fn id(&self) -> LanguageModelProviderId {
        PROVIDER_ID
    }

    fn name(&self) -> LanguageModelProviderName {
        PROVIDER_NAME
    }

    fn icon(&self) -> IconOrSvg {
        IconOrSvg::Icon(IconName::AiLlamaCpp)
    }

    fn default_model(&self, _: &App) -> Option<Arc<dyn LanguageModel>> {
        // No default model: in router mode it could trigger an expensive load of
        // an unloaded model on a constrained machine.
        None
    }

    fn default_fast_model(&self, _: &App) -> Option<Arc<dyn LanguageModel>> {
        // See explanation for default_model.
        None
    }

    fn provided_models(&self, cx: &App) -> Vec<Arc<dyn LanguageModel>> {
        let settings = LlamaCppLanguageModelProvider::settings(cx);
        let effective = compute_effective_models(&self.state.read(cx).fetched_models, settings);

        // Refresh the shared capability map so open conversations pick up settings changes.
        sync_capability_cells(&self.capability_cells, &effective);
        let mut models = effective
            .into_values()
            .map(|model| {
                Arc::new(LlamaCppLanguageModel {
                    id: LanguageModelId::from(model.name.clone()),
                    name: model.name.clone(),
                    display_name: model.display_name().to_string(),
                    fallback_capabilities: LiveCapabilities::of(&model),
                    supports_images: model.supports_images,
                    capability_cells: self.capability_cells.clone(),
                    loading_progress: self.loading_progress.clone(),
                    http_client: self.http_client.clone(),
                    request_limiter: RateLimiter::new(4),
                    state: self.state.clone(),
                }) as Arc<dyn LanguageModel>
            })
            .collect::<Vec<_>>();
        models.sort_by_key(|model| model.name());
        models
    }

    fn is_authenticated(&self, cx: &App) -> bool {
        self.state.read(cx).is_authenticated()
    }

    fn authenticate(&self, cx: &mut App) -> Task<Result<(), AuthenticateError>> {
        self.state.update(cx, |state, cx| state.authenticate(cx))
    }

    fn settings_view(&self, _cx: &mut App) -> Option<ProviderSettingsView> {
        let state = self.state.clone();
        Some(ProviderSettingsView::SubPage(
            SubPageProviderSettings::new(move |window, cx| {
                cx.new(|cx| ConfigurationView::new(state.clone(), window, cx))
                    .into()
            })
            .description(InlineDescription::Text(
                "Run local models on your machine with LlamaCpp.".into(),
            )),
        ))
    }
}

pub struct LlamaCppLanguageModel {
    id: LanguageModelId,
    /// The model id sent to the server.
    name: String,
    display_name: String,
    /// Live capabilities shared with the provider, read fresh on each access so an
    /// open conversation reflects the model's real values once it has loaded.
    capability_cells: CapabilityCells,
    /// Used when `capability_cells` has no entry (e.g. model removed mid-conversation).
    fallback_capabilities: LiveCapabilities,
    /// Available from `/v1/models` hints, so captured at build time.
    supports_images: bool,
    /// Shared with the provider; this model's load progress, read by `name` so the
    /// selector can show a loading indicator.
    loading_progress: LoadingProgress,
    http_client: Arc<dyn HttpClient>,
    request_limiter: RateLimiter,
    state: Entity<State>,
}

impl LlamaCppLanguageModel {
    /// The model's live capabilities, or the build-time fallback if the map lacks it.
    fn capabilities(&self) -> LiveCapabilities {
        read_recover(&self.capability_cells)
            .get(&self.name)
            .copied()
            .unwrap_or(self.fallback_capabilities)
    }

    /// This model's load-status label while loading, read live from the shared map.
    fn loading_label(&self) -> Option<SharedString> {
        read_recover(&self.loading_progress)
            .get(&self.name)
            .cloned()
    }

    fn to_llama_cpp_request(
        &self,
        request: LanguageModelRequest,
    ) -> Result<llama_cpp::ChatCompletionRequest> {
        build_llama_cpp_request(
            &self.name,
            self.supports_images,
            self.capabilities(),
            request,
        )
    }

    fn stream_completion(
        &self,
        request: llama_cpp::ChatCompletionRequest,
        cx: &AsyncApp,
    ) -> BoxFuture<
        'static,
        Result<futures::stream::BoxStream<'static, Result<llama_cpp::ResponseStreamEvent>>>,
    > {
        let http_client = self.http_client.clone();
        let (api_key, api_url, extra_headers) = self.state.read_with(cx, |state, cx| {
            let api_url = LlamaCppLanguageModelProvider::api_url(cx);
            let extra_headers = LlamaCppLanguageModelProvider::settings(cx)
                .custom_headers
                .clone();
            (state.api_key_state.key(&api_url), api_url, extra_headers)
        });

        let future = self.request_limiter.stream(async move {
            let stream = stream_chat_completion(
                http_client.as_ref(),
                &api_url,
                api_key.as_deref(),
                request,
                &extra_headers,
            )
            .await?;
            Ok(stream)
        });

        async move { Ok(future.await?.boxed()) }.boxed()
    }
}

fn build_llama_cpp_request(
    model_name: &str,
    supports_images: bool,
    capabilities: LiveCapabilities,
    request: LanguageModelRequest,
) -> Result<llama_cpp::ChatCompletionRequest> {
    if request.contains_custom_tool_input() {
        anyhow::bail!("llama.cpp does not support custom tools");
    }

    let supports_tools = capabilities.supports_tools;
    let supports_thinking = capabilities.supports_thinking;
    let mut messages = Vec::new();

    for message in request.messages {
        let mut reasoning_content: Option<String> = None;
        for content in message.content {
            match content {
                MessageContent::Text(text) => add_message_content_part(
                    llama_cpp::MessagePart::Text { text },
                    message.role,
                    &mut messages,
                    if supports_thinking && message.role == Role::Assistant {
                        reasoning_content.take()
                    } else {
                        None
                    },
                ),
                MessageContent::Thinking { text, .. } => {
                    if supports_thinking && message.role == Role::Assistant && !text.is_empty() {
                        reasoning_content.get_or_insert_default().push_str(&text);
                    }
                }
                MessageContent::RedactedThinking(_) => {}
                MessageContent::Compaction(_) => {}
                MessageContent::Image(image) => {
                    if supports_images {
                        add_message_content_part(
                            llama_cpp::MessagePart::Image {
                                image_url: llama_cpp::ImageUrl {
                                    url: image.to_base64_url(),
                                    detail: None,
                                },
                            },
                            message.role,
                            &mut messages,
                            if supports_thinking && message.role == Role::Assistant {
                                reasoning_content.take()
                            } else {
                                None
                            },
                        );
                    }
                }
                MessageContent::ToolUse(tool_use) => {
                    let input = tool_use.input.as_json().ok_or_else(|| {
                        anyhow::anyhow!("llama.cpp does not support custom tool calls")
                    })?;
                    let tool_call = llama_cpp::ToolCall {
                        id: tool_use.id.to_string(),
                        content: llama_cpp::ToolCallContent::Function {
                            function: llama_cpp::FunctionContent {
                                name: tool_use.name.to_string(),
                                arguments: serde_json::to_string(input).unwrap_or_default(),
                            },
                        },
                    };

                    if let Some(llama_cpp::ChatMessage::Assistant {
                        tool_calls,
                        reasoning_content: message_reasoning_content,
                        ..
                    }) = messages.last_mut()
                    {
                        append_reasoning_content(
                            message_reasoning_content,
                            reasoning_content.take(),
                        );
                        tool_calls.push(tool_call);
                    } else {
                        messages.push(llama_cpp::ChatMessage::Assistant {
                            content: None,
                            reasoning_content: reasoning_content.take(),
                            tool_calls: vec![tool_call],
                        });
                    }
                }
                MessageContent::ToolResult(tool_result) => {
                    let content: Vec<llama_cpp::MessagePart> = tool_result
                        .content
                        .iter()
                        .filter_map(|part| match part {
                            LanguageModelToolResultContent::Text(text) => {
                                Some(llama_cpp::MessagePart::Text {
                                    text: text.to_string(),
                                })
                            }
                            LanguageModelToolResultContent::Image(image) => {
                                if supports_images {
                                    Some(llama_cpp::MessagePart::Image {
                                        image_url: llama_cpp::ImageUrl {
                                            url: image.to_base64_url(),
                                            detail: None,
                                        },
                                    })
                                } else {
                                    None
                                }
                            }
                        })
                        .collect();

                    messages.push(llama_cpp::ChatMessage::Tool {
                        content: content.into(),
                        tool_call_id: tool_result.tool_use_id.to_string(),
                    });
                }
            }
        }
    }

    let tools: Vec<llama_cpp::ToolDefinition> = if supports_tools {
        request
            .tools
            .into_iter()
            .map(|tool| {
                let input_schema = match tool.input {
                    language_model::LanguageModelRequestToolInput::Function {
                        input_schema,
                        ..
                    } => input_schema,
                    language_model::LanguageModelRequestToolInput::Custom { .. } => {
                        return Err(anyhow::anyhow!("llama.cpp does not support custom tools"));
                    }
                };
                Ok(llama_cpp::ToolDefinition::Function {
                    function: llama_cpp::FunctionDefinition {
                        name: tool.name,
                        description: Some(tool.description),
                        parameters: Some(input_schema),
                    },
                })
            })
            .collect::<Result<_>>()?
    } else {
        Vec::new()
    };
    // Only send `tool_choice` with actual tools; some OpenAI-compatible servers
    // reject it otherwise.
    let tool_choice = if tools.is_empty() {
        None
    } else {
        request.tool_choice.map(|choice| match choice {
            LanguageModelToolChoice::Auto => llama_cpp::ToolChoice::Auto,
            LanguageModelToolChoice::Any => llama_cpp::ToolChoice::Required,
            LanguageModelToolChoice::None => llama_cpp::ToolChoice::None,
        })
    };

    Ok(llama_cpp::ChatCompletionRequest {
        model: model_name.to_string(),
        messages,
        stream: true,
        // Without an explicit limit, preserve the server's `n_predict` default.
        max_tokens: request.max_output_tokens.map(i32::try_from).transpose()?,
        stop: if request.stop.is_empty() {
            None
        } else {
            Some(request.stop)
        },
        // llama.cpp models often ship recommended sampler settings, so override
        // temperature only when the request sets one.
        temperature: request.temperature,
        tools,
        tool_choice,
        stream_options: Some(llama_cpp::StreamOptions {
            include_usage: true,
        }),
    })
}

impl LanguageModel for LlamaCppLanguageModel {
    fn id(&self) -> LanguageModelId {
        self.id.clone()
    }

    fn name(&self) -> LanguageModelName {
        match self.loading_label() {
            // Surface load progress in the display name so it shows wherever the
            // model is named, without provider-agnostic UI changes. The agent
            // rebuilds the name on `ProviderStateChanged`, which our ticks emit.
            Some(label) => LanguageModelName::from(format!("{} · {}", self.display_name, label)),
            None => LanguageModelName::from(self.display_name.clone()),
        }
    }

    fn provider_id(&self) -> LanguageModelProviderId {
        PROVIDER_ID
    }

    fn provider_name(&self) -> LanguageModelProviderName {
        PROVIDER_NAME
    }

    fn supports_tools(&self) -> bool {
        self.capabilities().supports_tools
    }

    fn supports_tool_choice(&self, choice: LanguageModelToolChoice) -> bool {
        self.supports_tools()
            && match choice {
                LanguageModelToolChoice::Auto => true,
                LanguageModelToolChoice::Any => true,
                LanguageModelToolChoice::None => true,
            }
    }

    fn supports_images(&self) -> bool {
        self.supports_images
    }

    fn supports_thinking(&self) -> bool {
        self.capabilities().supports_thinking
    }

    fn telemetry_id(&self) -> String {
        telemetry_id_for(&self.name)
    }

    fn max_token_count(&self) -> u64 {
        self.capabilities().max_tokens
    }

    fn stream_completion(
        &self,
        request: LanguageModelRequest,
        cx: &AsyncApp,
    ) -> BoxFuture<
        'static,
        Result<
            BoxStream<'static, Result<LanguageModelCompletionEvent, LanguageModelCompletionError>>,
            LanguageModelCompletionError,
        >,
    > {
        let request = match self.to_llama_cpp_request(request) {
            Ok(request) => request,
            Err(error) => return async move { Err(error.into()) }.boxed(),
        };
        let completions = self.stream_completion(request, cx);
        async move {
            let mapper = ChatCompletionEventMapper::new();
            Ok(mapper.map_stream(completions.await?).boxed())
        }
        .boxed()
    }
}

fn add_message_content_part(
    new_part: llama_cpp::MessagePart,
    role: Role,
    messages: &mut Vec<llama_cpp::ChatMessage>,
    reasoning_content: Option<String>,
) {
    match (role, messages.last_mut()) {
        (Role::User, Some(llama_cpp::ChatMessage::User { content }))
        | (Role::System, Some(llama_cpp::ChatMessage::System { content })) => {
            content.push_part(new_part);
        }
        (
            Role::Assistant,
            Some(llama_cpp::ChatMessage::Assistant {
                content: Some(content),
                reasoning_content: message_reasoning_content,
                ..
            }),
        ) => {
            append_reasoning_content(message_reasoning_content, reasoning_content);
            content.push_part(new_part);
        }
        _ => {
            messages.push(match role {
                Role::User => llama_cpp::ChatMessage::User {
                    content: llama_cpp::MessageContent::from(vec![new_part]),
                },
                Role::Assistant => llama_cpp::ChatMessage::Assistant {
                    content: Some(llama_cpp::MessageContent::from(vec![new_part])),
                    reasoning_content,
                    tool_calls: Vec::new(),
                },
                Role::System => llama_cpp::ChatMessage::System {
                    content: llama_cpp::MessageContent::from(vec![new_part]),
                },
            });
        }
    }
}

fn append_reasoning_content(target: &mut Option<String>, content: Option<String>) {
    let Some(content) = content else {
        return;
    };
    if content.is_empty() {
        return;
    }
    target.get_or_insert_default().push_str(&content);
}

fn merge_settings_into_models(
    models: &mut HashMap<String, llama_cpp::Model>,
    available_models: &[AvailableModel],
    context_window: Option<u64>,
) {
    for setting_model in available_models {
        if let Some(model) = models.get_mut(&setting_model.name) {
            if context_window.is_none() {
                model.max_tokens = setting_model.max_tokens;
            }
            if setting_model.display_name.is_some() {
                model.display_name = setting_model.display_name.clone();
            }
            if let Some(supports_tools) = setting_model.remote_supports_tools {
                model.supports_tools = supports_tools;
            }
            if let Some(supports_images) = setting_model.remote_supports_images {
                model.supports_images = supports_images;
            }
            if let Some(supports_thinking) = setting_model.supports_thinking {
                model.supports_thinking = supports_thinking;
            }
        } else {
            models.insert(
                setting_model.name.clone(),
                llama_cpp::Model {
                    name: setting_model.name.clone(),
                    display_name: setting_model.display_name.clone(),
                    max_tokens: context_window.unwrap_or(setting_model.max_tokens),
                    supports_tools: setting_model.remote_supports_tools.unwrap_or(false),
                    supports_images: setting_model.remote_supports_images.unwrap_or(false),
                    supports_thinking: setting_model.supports_thinking.unwrap_or(false),
                },
            );
        }
    }
}

struct ConfigurationView {
    api_key_editor: Entity<InputField>,
    api_url_editor: Entity<InputField>,
    context_window_editor: Entity<InputField>,
    state: Entity<State>,
}

impl ConfigurationView {
    pub fn new(state: Entity<State>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let api_key_editor = cx.new(|cx| InputField::new(window, cx, "sk-...").label("API key"));

        let api_url_editor = cx.new(|cx| {
            let input = InputField::new(window, cx, LLAMA_CPP_API_URL).label("API URL");
            input.set_text(&LlamaCppLanguageModelProvider::api_url(cx), window, cx);
            input
        });

        let context_window_editor = cx.new(|cx| {
            let input = InputField::new(window, cx, "8192").label("Context Window");
            if let Some(context_window) = LlamaCppLanguageModelProvider::settings(cx).remote_context_window
            {
                input.set_text(&context_window.to_string(), window, cx);
            }
            input
        });

        cx.observe(&state, |_, _, cx| {
            cx.notify();
        })
        .detach();

        Self {
            api_key_editor,
            api_url_editor,
            context_window_editor,
            state,
        }
    }

    fn retry_connection(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let has_api_url = LlamaCppLanguageModelProvider::has_custom_url(cx);
        let has_api_key = self
            .state
            .read_with(cx, |state, _| state.api_key_state.has_key());
        if !has_api_url {
            self.save_api_url(cx);
        }
        if !has_api_key {
            self.save_api_key(&Default::default(), window, cx);
        }

        self.state.update(cx, |state, cx| {
            state.restart_fetch_models_task(cx);
        });
    }

    fn save_api_key(&mut self, _: &menu::Confirm, window: &mut Window, cx: &mut Context<Self>) {
        let api_key = self.api_key_editor.read(cx).text(cx).trim().to_string();
        if api_key.is_empty() {
            return;
        }

        // A URL change can cause the editor to be shown again.
        self.api_key_editor
            .update(cx, |input, cx| input.set_text("", window, cx));

        let state = self.state.clone();
        cx.spawn_in(window, async move |_, cx| {
            state
                .update(cx, |state, cx| state.set_api_key(Some(api_key), cx))
                .await
        })
        .detach_and_log_err(cx);
    }

    fn reset_api_key(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.api_key_editor
            .update(cx, |input, cx| input.set_text("", window, cx));

        let state = self.state.clone();
        cx.spawn_in(window, async move |_, cx| {
            state
                .update(cx, |state, cx| state.set_api_key(None, cx))
                .await
        })
        .detach_and_log_err(cx);

        cx.notify();
    }

    fn save_api_url(&self, cx: &mut Context<Self>) {
        let api_url = self.api_url_editor.read(cx).text(cx).trim().to_string();
        let current_url = LlamaCppLanguageModelProvider::api_url(cx);
        if !api_url.is_empty() && &api_url != &current_url {
            let fs = <dyn Fs>::global(cx);
            update_settings_file(fs, cx, move |settings, _| {
                settings
                    .language_models
                    .get_or_insert_default()
                    .llama_cpp
                    .get_or_insert_default()
                    .api_url = Some(api_url);
            });
        }
    }

    fn reset_api_url(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.api_url_editor
            .update(cx, |input, cx| input.set_text("", window, cx));
        let fs = <dyn Fs>::global(cx);
        update_settings_file(fs, cx, |settings, _cx| {
            if let Some(settings) = settings
                .language_models
                .as_mut()
                .and_then(|models| models.llama_cpp.as_mut())
            {
                settings.api_url = Some(LLAMA_CPP_API_URL.into());
            }
        });
        cx.notify();
    }

    fn save_context_window(&mut self, cx: &mut Context<Self>) {
        let context_window_str = self
            .context_window_editor
            .read(cx)
            .text(cx)
            .trim()
            .to_string();
        let current_context_window = LlamaCppLanguageModelProvider::settings(cx).remote_context_window;

        if let Ok(context_window) = context_window_str.parse::<u64>() {
            if Some(context_window) != current_context_window {
                let fs = <dyn Fs>::global(cx);
                update_settings_file(fs, cx, move |settings, _| {
                    settings
                        .language_models
                        .get_or_insert_default()
                        .llama_cpp
                        .get_or_insert_default()
                        .context_window = Some(context_window);
                });
            }
        } else if context_window_str.is_empty() && current_context_window.is_some() {
            let fs = <dyn Fs>::global(cx);
            update_settings_file(fs, cx, move |settings, _| {
                settings
                    .language_models
                    .get_or_insert_default()
                    .llama_cpp
                    .get_or_insert_default()
                    .context_window = None;
            });
        }
    }

    fn reset_context_window(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.context_window_editor
            .update(cx, |input, cx| input.set_text("", window, cx));
        let fs = <dyn Fs>::global(cx);
        update_settings_file(fs, cx, |settings, _cx| {
            if let Some(settings) = settings
                .language_models
                .as_mut()
                .and_then(|models| models.llama_cpp.as_mut())
            {
                settings.remote_context_window = None;
            }
        });
        cx.notify();
    }

    fn render_instructions(cx: &App) -> Div {
        v_flex()
            .gap_2()
            .child(
                Label::new(
                    "Run open models locally with llama.cpp's built-in server, or connect to a \
                remote llama.cpp server.",
                )
                .color(Color::Muted),
            )
            .child(Label::new("To use a local llama.cpp server:").color(Color::Muted))
            .child(
                List::new()
                    .child(
                        ListBulletItem::new("")
                            .child(Label::new("Install llama.cpp from").color(Color::Muted))
                            .child(ButtonLink::new("llama.app", LLAMA_CPP_DOWNLOAD_URL)),
                    )
                    .child(
                        ListBulletItem::new("")
                            .child(
                                Label::new("Start the server in router mode:").color(Color::Muted),
                            )
                            .child(Label::new("llama serve").inline_code(cx)),
                    )
                    .child(
                        ListBulletItem::new(
                            "Click 'Connect' below to start using llama.cpp in Zed",
                        )
                        .label_color(Color::Muted),
                    ),
            )
            .child(
                Label::new(
                    "Alternatively, you can connect to a remote llama.cpp server by specifying its \
                URL and API key (set with --api-key, may not be required):",
                )
                .color(Color::Muted),
            )
    }

    fn render_api_key_editor(&self, cx: &Context<Self>) -> impl IntoElement {
        let state = self.state.read(cx);
        let env_var_set = state.api_key_state.is_from_env_var();
        let configured_card_label = if env_var_set {
            format!("API key set in {API_KEY_ENV_VAR_NAME} environment variable.")
        } else {
            "API key configured".to_string()
        };

        let api_key_control = if !state.api_key_state.has_key() {
            self.api_key_editor.clone().into_any_element()
        } else {
            ConfiguredApiCard::new("llama-cpp-reset-key", configured_card_label)
                .disabled(env_var_set)
                .on_click(cx.listener(|this, _, window, cx| this.reset_api_key(window, cx)))
                .when(env_var_set, |this| {
                    this.tooltip_label(format!(
                        "To reset your API key, unset the {API_KEY_ENV_VAR_NAME} environment variable."
                    ))
                })
                .into_any_element()
        };

        v_flex()
            .on_action(cx.listener(Self::save_api_key))
            .child(api_key_control)
            .gap_1p5()
            .mb_2()
            .child(
                Label::new(format!(
                    "You can also set the {API_KEY_ENV_VAR_NAME} environment variable and restart Zed."
                ))
                .size(LabelSize::Small)
                .color(Color::Muted),
            )
    }

    fn render_context_window_editor(&self, cx: &Context<Self>) -> Div {
        let settings = LlamaCppLanguageModelProvider::settings(cx);
        let custom_context_window_set = settings.remote_context_window.is_some();

        if custom_context_window_set {
            h_flex()
                .p_1()
                .justify_between()
                .rounded_md()
                .border_1()
                .border_color(cx.theme().colors().border_variant)
                .bg(cx.theme().colors().background.opacity(0.5))
                .child(
                    h_flex()
                        .gap_1()
                        .child(Icon::new(IconName::Check).color(Color::Success))
                        .child(Label::new(format!(
                            "Context Window: {}",
                            settings.remote_context_window.unwrap_or_default()
                        ))),
                )
                .child(
                    Button::new("reset-context-window", "Reset")
                        .style(ButtonStyle::Outlined)
                        .label_size(LabelSize::Small)
                        .start_icon(Icon::new(IconName::Undo).size(IconSize::Small))
                        .on_click(
                            cx.listener(|this, _, window, cx| {
                                this.reset_context_window(window, cx)
                            }),
                        ),
                )
        } else {
            v_flex()
                .on_action(
                    cx.listener(|this, _: &menu::Confirm, _window, cx| {
                        this.save_context_window(cx)
                    }),
                )
                .child(self.context_window_editor.clone())
                .gap_1p5()
                .child(
                    Label::new("Default: Discovered from the server")
                        .size(LabelSize::Small)
                        .color(Color::Muted),
                )
        }
    }

    fn render_api_url_editor(&self, cx: &Context<Self>) -> Div {
        let api_url = LlamaCppLanguageModelProvider::api_url(cx);
        let custom_api_url_set = api_url != LLAMA_CPP_API_URL;

        if custom_api_url_set {
            h_flex()
                .p_1()
                .justify_between()
                .rounded_md()
                .border_1()
                .border_color(cx.theme().colors().border_variant)
                .bg(cx.theme().colors().background.opacity(0.5))
                .child(
                    h_flex()
                        .gap_1()
                        .child(Icon::new(IconName::Check).color(Color::Success))
                        .child(Label::new(api_url)),
                )
                .child(
                    Button::new("reset-api-url", "Reset API URL")
                        .style(ButtonStyle::Outlined)
                        .label_size(LabelSize::Small)
                        .start_icon(Icon::new(IconName::Undo).size(IconSize::Small))
                        .on_click(
                            cx.listener(|this, _, window, cx| this.reset_api_url(window, cx)),
                        ),
                )
        } else {
            v_flex()
                .on_action(cx.listener(|this, _: &menu::Confirm, _window, cx| {
                    this.save_api_url(cx);
                    cx.notify();
                }))
                .gap_1p5()
                .child(self.api_url_editor.clone())
        }
    }
}

impl Render for ConfigurationView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_authenticated = self.state.read(cx).is_authenticated();

        v_flex()
            .gap_2()
            .child(Headline::new("llama.cpp").size(HeadlineSize::Small))
            .child(Self::render_instructions(cx))
            .child(self.render_api_url_editor(cx))
            .child(self.render_context_window_editor(cx))
            .child(self.render_api_key_editor(cx))
            .child(Divider::horizontal())
            .child(
                h_flex()
                    .pt_2()
                    .w_full()
                    .justify_between()
                    .gap_2()
                    .child(
                        h_flex()
                            .w_full()
                            .gap_2()
                            .map(|this| {
                                if is_authenticated {
                                    this.child(
                                        Button::new("llama-cpp-webui", "Open WebUI")
                                            .style(ButtonStyle::OutlinedGhost)
                                            .size(ButtonSize::Medium)
                                            .end_icon(
                                                Icon::new(IconName::ArrowUpRight)
                                                    .size(IconSize::XSmall)
                                                    .color(Color::Muted),
                                            )
                                            .on_click(move |_, _, cx| {
                                                let url =
                                                    LlamaCppLanguageModelProvider::api_url(cx);
                                                cx.open_url(&url);
                                            })
                                            .into_any_element(),
                                    )
                                    .child(
                                        Button::new("llama-cpp-site", "llama.cpp")
                                            .style(ButtonStyle::OutlinedGhost)
                                            .size(ButtonSize::Medium)
                                            .end_icon(
                                                Icon::new(IconName::ArrowUpRight)
                                                    .size(IconSize::XSmall)
                                                    .color(Color::Muted),
                                            )
                                            .on_click(move |_, _, cx| {
                                                cx.open_url(LLAMA_CPP_DOWNLOAD_URL)
                                            })
                                            .into_any_element(),
                                    )
                                } else {
                                    this.child(
                                        Button::new("download_llama_cpp_button", "Get llama.cpp")
                                            .style(ButtonStyle::OutlinedGhost)
                                            .size(ButtonSize::Medium)
                                            .end_icon(
                                                Icon::new(IconName::ArrowUpRight)
                                                    .size(IconSize::XSmall)
                                                    .color(Color::Muted),
                                            )
                                            .on_click(move |_, _, cx| {
                                                cx.open_url(LLAMA_CPP_DOWNLOAD_URL)
                                            })
                                            .into_any_element(),
                                    )
                                }
                            })
                            .child(
                                Button::new("view-models", "Browse GGUF Models")
                                    .style(ButtonStyle::OutlinedGhost)
                                    .size(ButtonSize::Medium)
                                    .end_icon(
                                        Icon::new(IconName::ArrowUpRight)
                                            .size(IconSize::XSmall)
                                            .color(Color::Muted),
                                    )
                                    .on_click(move |_, _, cx| cx.open_url(LLAMA_CPP_MODELS_URL)),
                            ),
                    )
                    .map(|this| {
                        if is_authenticated {
                            this.child(
                                ButtonLike::new("connected")
                                    .size(ButtonSize::Medium)
                                    .child(
                                        h_flex()
                                            .gap_1()
                                            .child(Icon::new(IconName::Check).color(Color::Success))
                                            .child(Label::new("Connected")),
                                    )
                                    .child(
                                        IconButton::new("refresh-models", IconName::RotateCcw)
                                            .icon_size(IconSize::Small)
                                            .tooltip(Tooltip::text("Refresh Models"))
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                this.state.update(cx, |state, _| {
                                                    state.fetched_models.clear();
                                                });
                                                this.retry_connection(window, cx);
                                            })),
                                    ),
                            )
                        } else {
                            this.child(
                                Button::new("retry_llama_cpp_models", "Connect")
                                    .style(ButtonStyle::Outlined)
                                    .size(ButtonSize::Medium)
                                    .start_icon(
                                        Icon::new(IconName::PlayOutlined).size(IconSize::XSmall),
                                    )
                                    .on_click(cx.listener(move |this, _, window, cx| {
                                        this.retry_connection(window, cx)
                                    })),
                            )
                        }
                    }),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::TestAppContext;
    use http_client::FakeHttpClient;
    use language_model::LanguageModelToolUse;
    use parking_lot::Mutex;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct FakeCredentialsProvider {
        api_key: Vec<u8>,
    }

    impl CredentialsProvider for FakeCredentialsProvider {
        fn read_credentials<'a>(
            &'a self,
            _url: &'a str,
            _cx: &'a AsyncApp,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<Option<(String, Vec<u8>)>>> + 'a>,
        > {
            let api_key = self.api_key.clone();
            Box::pin(async move { Ok(Some(("Bearer".to_string(), api_key))) })
        }

        fn write_credentials<'a>(
            &'a self,
            _url: &'a str,
            _username: &'a str,
            _password: &'a [u8],
            _cx: &'a AsyncApp,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + 'a>> {
            Box::pin(async { Ok(()) })
        }

        fn delete_credentials<'a>(
            &'a self,
            _url: &'a str,
            _cx: &'a AsyncApp,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + 'a>> {
            Box::pin(async { Ok(()) })
        }
    }

    fn entry(id: &str, n_ctx: Option<u64>, n_ctx_train: Option<u64>) -> ModelEntry {
        ModelEntry {
            id: id.to_string(),
            meta: Some(llama_cpp::ModelMeta { n_ctx, n_ctx_train }),
            architecture: None,
            status: None,
        }
    }

    #[test]
    fn display_name_strips_path_and_extension() {
        assert_eq!(
            display_name_for("../models/Qwen2.5-Coder-7B-Instruct-Q4_K_M.gguf"),
            "Qwen2.5-Coder-7B-Instruct-Q4_K_M"
        );
        assert_eq!(display_name_for("my-alias"), "my-alias");
    }

    #[test]
    fn telemetry_id_strips_local_model_paths() {
        assert_eq!(
            telemetry_id_for("/Users/alice/models/Qwen2.5-Coder-7B-Instruct-Q4_K_M.gguf"),
            "llama.cpp/Qwen2.5-Coder-7B-Instruct-Q4_K_M"
        );
        assert_eq!(
            telemetry_id_for(r"C:\Users\alice\models\Qwen2.5-Coder-7B-Instruct-Q4_K_M.gguf"),
            "llama.cpp/Qwen2.5-Coder-7B-Instruct-Q4_K_M"
        );
        assert_eq!(telemetry_id_for("my-alias"), "llama.cpp/my-alias");
    }

    #[test]
    fn model_uses_props_then_meta_for_context() {
        let props = Props {
            default_generation_settings: Some(llama_cpp::GenerationSettings { n_ctx: Some(8192) }),
            modalities: Some(llama_cpp::Modalities { vision: true }),
            chat_template_caps: Some(llama_cpp::ChatTemplateCaps {
                supports_tool_calls: true,
                supports_preserve_reasoning: true,
                ..Default::default()
            }),
        };
        // /props wins when present.
        let model = model_from_entry(&entry("m", Some(4096), Some(131072)), Some(&props));
        assert_eq!(model.max_tokens, 8192);
        assert!(model.supports_tools);
        assert!(model.supports_images);
        assert!(model.supports_thinking);

        // Unprobed: falls back to the listing's runtime context, then trained
        // context. Tools are assumed supported until the model loads.
        let model = model_from_entry(&entry("m", Some(4096), Some(131072)), None);
        assert_eq!(model.max_tokens, 4096);
        assert!(model.supports_tools);
        assert!(!model.supports_thinking);

        let model = model_from_entry(&entry("m", None, Some(131072)), None);
        assert_eq!(model.max_tokens, 131072);

        // Nothing reported -> the optimistic unloaded-context assumption.
        let model = model_from_entry(&entry("m", None, None), None);
        assert_eq!(model.max_tokens, ASSUMED_UNLOADED_CONTEXT);
        assert!(model.supports_tools);
    }

    #[test]
    fn router_entry_detects_vision_from_modalities() {
        let router_entry = ModelEntry {
            id: "vlm".to_string(),
            meta: None,
            architecture: Some(llama_cpp::Architecture {
                input_modalities: vec!["text".to_string(), "image".to_string()],
            }),
            status: Some(llama_cpp::ModelStatus {
                value: "unloaded".to_string(),
            }),
        };
        let model = model_from_entry(&router_entry, None);
        assert!(model.supports_images);
        // Unprobed router models optimistically advertise tools until loaded.
        assert!(model.supports_tools);
    }

    #[test]
    fn settings_override_capabilities_and_context() {
        let mut models: HashMap<String, llama_cpp::Model> = HashMap::default();
        models.insert(
            "qwen".to_string(),
            llama_cpp::Model::new("qwen", Some("qwen"), Some(8192), false, false, false),
        );

        let available = vec![AvailableModel {
            name: "qwen".to_string(),
            display_name: Some("Qwen Coder".to_string()),
            max_tokens: 16384,
            supports_tools: Some(true),
            supports_images: None,
            supports_thinking: Some(true),
        }];

        merge_settings_into_models(&mut models, &available, None);

        let model = models.get("qwen").unwrap();
        assert_eq!(model.display_name.as_deref(), Some("Qwen Coder"));
        assert_eq!(model.max_tokens, 16384);
        assert!(model.supports_tools);
        assert!(model.supports_thinking);
        // Unspecified capability keeps the discovered value.
        assert!(!model.supports_images);
    }

    #[test]
    fn capability_cells_update_when_a_model_loads() {
        let cells: CapabilityCells = Arc::new(RwLock::new(HashMap::default()));
        let settings = LlamaCppSettings {
            auto_discover: true,
            ..Default::default()
        };

        // Cold: the optimistic unloaded-context assumption.
        let cold = vec![llama_cpp::Model::new(
            "m",
            Some("m"),
            Some(ASSUMED_UNLOADED_CONTEXT),
            true,
            false,
            false,
        )];
        sync_capability_cells(&cells, &compute_effective_models(&cold, &settings));
        assert_eq!(
            cells.read().unwrap().get("m").unwrap().max_tokens,
            ASSUMED_UNLOADED_CONTEXT
        );

        // The model loads and reports its real context. The shared map must
        // reflect the new value so a model reading it by name (an open
        // conversation) is no longer stuck on the cold-start assumption.
        let loaded = vec![llama_cpp::Model::new(
            "m",
            Some("m"),
            Some(262_144),
            true,
            false,
            true,
        )];
        sync_capability_cells(&cells, &compute_effective_models(&loaded, &settings));
        assert_eq!(cells.read().unwrap().get("m").unwrap().max_tokens, 262_144);
        assert!(cells.read().unwrap().get("m").unwrap().supports_thinking);
    }

    #[test]
    fn request_preserves_assistant_thinking_when_supported() {
        let request = build_llama_cpp_request(
            "test-model",
            false,
            LiveCapabilities {
                max_tokens: 8192,
                supports_tools: false,
                supports_thinking: true,
            },
            LanguageModelRequest {
                max_output_tokens: Some(1024),
                messages: vec![language_model::LanguageModelRequestMessage {
                    role: Role::Assistant,
                    content: vec![
                        MessageContent::Thinking {
                            text: "reasoning".to_string(),
                            signature: None,
                        },
                        MessageContent::Text("answer".to_string()),
                    ],
                    cache: false,
                    reasoning_details: None,
                }],
                ..Default::default()
            },
        )
        .unwrap();

        assert_eq!(serde_json::to_value(&request).unwrap()["max_tokens"], 1024);
        assert_eq!(request.messages.len(), 1);
        match &request.messages[0] {
            llama_cpp::ChatMessage::Assistant {
                content: Some(llama_cpp::MessageContent::Plain(content)),
                reasoning_content: Some(reasoning_content),
                tool_calls,
            } => {
                assert_eq!(content, "answer");
                assert_eq!(reasoning_content, "reasoning");
                assert!(tool_calls.is_empty());
            }
            message => panic!("unexpected message: {message:?}"),
        }
    }

    #[test]
    fn request_skips_assistant_thinking_when_unsupported() {
        let request = build_llama_cpp_request(
            "test-model",
            false,
            LiveCapabilities {
                max_tokens: 8192,
                supports_tools: false,
                supports_thinking: false,
            },
            LanguageModelRequest {
                messages: vec![language_model::LanguageModelRequestMessage {
                    role: Role::Assistant,
                    content: vec![
                        MessageContent::Thinking {
                            text: "reasoning".to_string(),
                            signature: None,
                        },
                        MessageContent::RedactedThinking("encrypted".to_string()),
                        MessageContent::Text("answer".to_string()),
                    ],
                    cache: false,
                    reasoning_details: None,
                }],
                ..Default::default()
            },
        )
        .unwrap();

        assert!(
            serde_json::to_value(&request)
                .unwrap()
                .get("max_tokens")
                .is_none()
        );
        assert_eq!(request.messages.len(), 1);
        match &request.messages[0] {
            llama_cpp::ChatMessage::Assistant {
                content: Some(llama_cpp::MessageContent::Plain(content)),
                reasoning_content,
                tool_calls,
            } => {
                assert_eq!(content, "answer");
                assert!(reasoning_content.is_none());
                assert!(tool_calls.is_empty());
            }
            message => panic!("unexpected message: {message:?}"),
        }
    }

    #[test]
    fn request_preserves_thinking_for_assistant_tool_calls_when_supported() {
        let request = build_llama_cpp_request(
            "test-model",
            false,
            LiveCapabilities {
                max_tokens: 8192,
                supports_tools: true,
                supports_thinking: true,
            },
            LanguageModelRequest {
                messages: vec![language_model::LanguageModelRequestMessage {
                    role: Role::Assistant,
                    content: vec![
                        MessageContent::Thinking {
                            text: "reasoning".to_string(),
                            signature: None,
                        },
                        MessageContent::ToolUse(LanguageModelToolUse {
                            id: "call_1".into(),
                            name: "weather".into(),
                            raw_input: r#"{"city":"Oslo"}"#.to_string(),
                            input: language_model::LanguageModelToolUseInput::Json(
                                serde_json::json!({ "city": "Oslo" }),
                            ),
                            is_input_complete: true,
                            thought_signature: None,
                        }),
                    ],
                    cache: false,
                    reasoning_details: None,
                }],
                ..Default::default()
            },
        )
        .unwrap();

        assert_eq!(request.messages.len(), 1);
        match &request.messages[0] {
            llama_cpp::ChatMessage::Assistant {
                content: None,
                reasoning_content: Some(reasoning_content),
                tool_calls,
            } => {
                assert_eq!(reasoning_content, "reasoning");
                assert_eq!(tool_calls.len(), 1);
            }
            message => panic!("unexpected message: {message:?}"),
        }
    }

    #[gpui::test]
    async fn authenticate_fetches_models_after_loading_api_key(cx: &mut TestAppContext) {
        cx.update(|cx| {
            let settings_store = SettingsStore::test(cx);
            cx.set_global(settings_store);
        });

        let model_request_authorizations = Arc::new(Mutex::new(Vec::new()));
        let model_request_count = Arc::new(AtomicUsize::new(0));
        let http_client = FakeHttpClient::create({
            let model_request_authorizations = model_request_authorizations.clone();
            let model_request_count = model_request_count.clone();
            move |request| {
                let model_request_authorizations = model_request_authorizations.clone();
                let model_request_count = model_request_count.clone();
                async move {
                    let path = request.uri().path();
                    let authorization = request
                        .headers()
                        .get("Authorization")
                        .and_then(|value| value.to_str().ok())
                        .map(str::to_string);

                    if path == "/v1/models" {
                        model_request_authorizations.lock().push(authorization);
                        let request_index = model_request_count.fetch_add(1, Ordering::SeqCst);
                        if request_index == 0 {
                            return Ok(http_client::Response::builder()
                                .status(503)
                                .body(http_client::AsyncBody::from("not ready"))?);
                        }

                        return Ok(http_client::Response::builder().status(200).body(
                            http_client::AsyncBody::from(
                                r#"{"data":[{"id":"test-model","meta":{"n_ctx":4096}}]}"#,
                            ),
                        )?);
                    }

                    if path == "/props" {
                        return Ok(http_client::Response::builder()
                            .status(200)
                            .body(http_client::AsyncBody::from("{}"))?);
                    }

                    Ok(http_client::Response::builder()
                        .status(404)
                        .body(http_client::AsyncBody::default())?)
                }
            }
        });
        let credentials_provider = Arc::new(FakeCredentialsProvider {
            api_key: b"loaded-key".to_vec(),
        });
        let provider = cx
            .update(|cx| LlamaCppLanguageModelProvider::new(http_client, credentials_provider, cx));

        cx.run_until_parked();

        let result = cx.update(|cx| provider.authenticate(cx)).await;
        assert!(
            result.is_ok(),
            "authenticate should discover models after loading credentials"
        );
        assert_eq!(
            &*model_request_authorizations.lock(),
            &[None, Some("Bearer loaded-key".to_string())]
        );
    }
}

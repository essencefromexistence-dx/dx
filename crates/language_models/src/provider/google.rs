use anyhow::Result;
use collections::BTreeMap;
use credentials_provider::CredentialsProvider;
use futures::{AsyncReadExt, FutureExt, StreamExt, future::BoxFuture};
use google_ai::GenerateContentResponse;
pub use google_ai::completion::{GoogleEventMapper, into_google};
use gpui::{App, AppContext, AsyncApp, Context, Entity, SharedString, Task};
use http_client::{
    AsyncBody, CustomHeaders, HttpClient, Method, Request as HttpRequest, RequestBuilderExt,
};
use language_model::{
    ApiKeyConfiguration, AuthenticateError, EnvVar, LanguageModelCompletionError,
    LanguageModelCompletionStream, LanguageModelToolChoiceSupport, unavailable_error,
};
use language_model::{
    GOOGLE_PROVIDER_ID, GOOGLE_PROVIDER_NAME, IconOrSvg, LanguageModel, LanguageModelClient,
    LanguageModelEffortLevel, LanguageModelId, LanguageModelName, LanguageModelProvider,
    LanguageModelProviderId, LanguageModelProviderName, LanguageModelProviderState,
    LanguageModelRequest, ModelRateLimiters, ProviderSettingsView,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
pub use settings::GoogleAvailableModel as AvailableModel;
use settings::{Settings, SettingsStore};
use std::sync::{Arc, LazyLock};
use ui::IconName;

use language_model::ApiKeyState;

const PROVIDER_ID: LanguageModelProviderId = GOOGLE_PROVIDER_ID;
const PROVIDER_NAME: LanguageModelProviderName = GOOGLE_PROVIDER_NAME;

#[derive(Default, Clone, Debug, PartialEq)]
pub struct GoogleSettings {
    pub api_url: String,
    pub available_models: Vec<AvailableModel>,
    pub custom_headers: CustomHeaders,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ModelMode {
    #[default]
    Default,
    Thinking {
        /// The maximum number of tokens to use for reasoning. Must be lower than the model's `max_output_tokens`.
        budget_tokens: Option<u32>,
    },
}

pub struct GoogleLanguageModelProvider {
    http_client: Arc<dyn HttpClient>,
    state: Entity<State>,
    request_limiters: ModelRateLimiters,
}

pub struct State {
    api_key_state: ApiKeyState,
    credentials_provider: Arc<dyn CredentialsProvider>,
    http_client: Arc<dyn HttpClient>,
    fetched_models: Vec<String>,
    fetch_models_task: Option<Task<Result<()>>>,
}

const GEMINI_API_KEY_VAR_NAME: &str = "GEMINI_API_KEY";
const GOOGLE_AI_API_KEY_VAR_NAME: &str = "GOOGLE_AI_API_KEY";

static API_KEY_ENV_VAR: LazyLock<EnvVar> = LazyLock::new(|| {
    // Try GEMINI_API_KEY first as primary, fallback to GOOGLE_AI_API_KEY
    EnvVar::new(GEMINI_API_KEY_VAR_NAME.into()).or(EnvVar::new(GOOGLE_AI_API_KEY_VAR_NAME.into()))
});

impl State {
    fn is_authenticated(&self) -> bool {
        self.api_key_state.has_key()
    }

    fn set_api_key(&mut self, api_key: Option<String>, cx: &mut Context<Self>) -> Task<Result<()>> {
        let credentials_provider = self.credentials_provider.clone();
        let api_url = GoogleLanguageModelProvider::api_url(cx);
        let should_fetch_models = api_key.is_some();
        let task = self.api_key_state.store(
            api_url,
            api_key,
            |this| &mut this.api_key_state,
            credentials_provider,
            cx,
        );
        self.fetched_models.clear();
        cx.spawn(async move |this, cx| {
            let result = task.await;
            if result.is_ok() && should_fetch_models {
                this.update(cx, |this, cx| this.restart_fetch_models_task(cx))
                    .ok();
            }
            result
        })
    }

    fn authenticate(&mut self, cx: &mut Context<Self>) -> Task<Result<(), AuthenticateError>> {
        let credentials_provider = self.credentials_provider.clone();
        let api_url = GoogleLanguageModelProvider::api_url(cx);
        let task = self.api_key_state.load_if_needed(
            api_url,
            |this| &mut this.api_key_state,
            credentials_provider,
            cx,
        );

        cx.spawn(async move |this, cx| {
            let result = task.await;
            if result.is_ok() {
                this.update(cx, |this, cx| this.restart_fetch_models_task(cx))
                    .ok();
            }
            result
        })
    }

    fn fetch_models(&mut self, cx: &mut Context<Self>) -> Task<Result<()>> {
        let http_client = self.http_client.clone();
        let api_url = GoogleLanguageModelProvider::api_url(cx);
        let Some(api_key) = self.api_key_state.key(&api_url) else {
            return Task::ready(Err(anyhow::anyhow!(
                "cannot fetch Google models without an API key"
            )));
        };
        let extra_headers = GoogleLanguageModelProvider::settings(cx)
            .custom_headers
            .clone();

        cx.spawn(async move |this, cx| {
            let models = list_google_models(
                http_client.as_ref(),
                &api_url,
                api_key.as_ref(),
                &extra_headers,
            )
            .await?;

            this.update(cx, |this, cx| {
                this.fetched_models = models;
                cx.notify();
            })
        })
    }

    fn restart_fetch_models_task(&mut self, cx: &mut Context<Self>) {
        let task = self.fetch_models(cx);
        self.fetch_models_task.replace(task);
    }
}

impl GoogleLanguageModelProvider {
    pub fn new(
        http_client: Arc<dyn HttpClient>,
        credentials_provider: Arc<dyn CredentialsProvider>,
        cx: &mut App,
    ) -> Self {
        let state = cx.new(|cx| {
            cx.observe_global::<SettingsStore>({
                let mut last_api_url = Self::api_url(cx);
                move |this: &mut State, cx| {
                    let credentials_provider = this.credentials_provider.clone();
                    let api_url = Self::api_url(cx);
                    let url_changed = api_url != last_api_url;
                    last_api_url = api_url.clone();
                    this.api_key_state.handle_url_change(
                        api_url,
                        |this| &mut this.api_key_state,
                        credentials_provider,
                        cx,
                    );
                    if url_changed {
                        this.fetched_models.clear();
                        this.authenticate(cx).detach();
                    }
                    cx.notify();
                }
            })
            .detach();
            State {
                api_key_state: ApiKeyState::new(Self::api_url(cx), (*API_KEY_ENV_VAR).clone()),
                credentials_provider,
                http_client: http_client.clone(),
                fetched_models: Vec::new(),
                fetch_models_task: None,
            }
        });

        Self {
            http_client,
            state,
            request_limiters: ModelRateLimiters::default(),
        }
    }

    /// Every model this provider offers, keyed by id: the fetched list, with
    /// settings entries added or overriding fetched ones.
    fn google_models(&self, cx: &App) -> BTreeMap<String, google_ai::Model> {
        let mut models = BTreeMap::default();

        // Models reported by Google's `/v1beta/models` endpoint are the
        // primary source. The list will be empty until authentication has
        // succeeded and the first fetch completes. Fetched ids carry no
        // metadata, so they become `Custom` models with the same field
        // defaults a settings-defined entry would get; a settings entry
        // with the same name overrides them below.
        for id in &self.state.read(cx).fetched_models {
            models.insert(
                id.clone(),
                google_ai::Model::Custom {
                    name: id.clone(),
                    display_name: None,
                    max_tokens: 1_048_576,
                    mode: google_ai::GoogleModelMode::Thinking {
                        budget_tokens: None,
                    },
                },
            );
        }

        for model in &GoogleLanguageModelProvider::settings(cx).available_models {
            models.insert(
                model.name.clone(),
                google_ai::Model::Custom {
                    name: model.name.clone(),
                    display_name: model.display_name.clone(),
                    max_tokens: model.max_tokens,
                    mode: model.mode.unwrap_or_default(),
                },
            );
        }

        models
    }

    /// Describes the offered model with `id`.
    fn offered_model(&self, id: &str, cx: &App) -> Option<LanguageModel> {
        self.google_models(cx).get(id).map(language_model)
    }

    /// The current configuration of `model`, if this provider still offers it.
    fn config(
        &self,
        model: &LanguageModel,
        cx: &App,
    ) -> Result<google_ai::Model, LanguageModelCompletionError> {
        self.google_models(cx)
            .remove(model.id.0.as_ref())
            .ok_or_else(|| unavailable_error(model))
    }

    fn stream_google_request(
        &self,
        request: google_ai::GenerateContentRequest,
        cx: &AsyncApp,
    ) -> BoxFuture<
        'static,
        Result<
            futures::stream::BoxStream<'static, Result<GenerateContentResponse>>,
            LanguageModelCompletionError,
        >,
    > {
        let http_client = self.http_client.clone();

        let (api_key, api_url, extra_headers) = self.state.read_with(cx, |state, cx| {
            let api_url = GoogleLanguageModelProvider::api_url(cx);
            let extra_headers = GoogleLanguageModelProvider::settings(cx)
                .custom_headers
                .clone();
            (state.api_key_state.key(&api_url), api_url, extra_headers)
        });

        async move {
            let Some(api_key) = api_key else {
                return Err(LanguageModelCompletionError::NoApiKey {
                    provider: PROVIDER_NAME,
                });
            };
            let request = google_ai::stream_generate_content(
                http_client.as_ref(),
                &api_url,
                &api_key,
                request,
                &extra_headers,
            );
            request.await
        }
        .boxed()
    }

    fn settings(cx: &App) -> &GoogleSettings {
        &crate::AllLanguageModelSettings::get_global(cx).google
    }

    fn api_url(cx: &App) -> SharedString {
        let api_url = &Self::settings(cx).api_url;
        if api_url.is_empty() {
            google_ai::API_URL.into()
        } else {
            SharedString::new(api_url.as_str())
        }
    }
}

impl LanguageModelProviderState for GoogleLanguageModelProvider {
    type ObservableEntity = State;

    fn observable_entity(&self) -> Option<Entity<Self::ObservableEntity>> {
        Some(self.state.clone())
    }
}

impl LanguageModelProvider for GoogleLanguageModelProvider {
    fn id(&self) -> LanguageModelProviderId {
        PROVIDER_ID
    }

    fn name(&self) -> LanguageModelProviderName {
        PROVIDER_NAME
    }

    fn icon(&self) -> IconOrSvg {
        IconOrSvg::Icon(IconName::AiGoogle)
    }

    fn default_model(&self, cx: &App) -> Option<LanguageModel> {
        let fetched = self.state.read(cx).fetched_models.clone();
        // Prefer a Pro model, then any Gemini model. Returning `None` until
        // the fetch completes matches the Anthropic provider's behavior.
        let gemini: Vec<String> = fetched
            .iter()
            .filter(|id| id.contains("gemini-"))
            .cloned()
            .collect();
        let preferred = pick_preferred_model(&gemini, &["pro"])
            .or_else(|| pick_preferred_model(&gemini, &["gemini-"]))?;
        self.offered_model(&preferred, cx)
    }

    fn default_fast_model(&self, cx: &App) -> Option<LanguageModel> {
        let fetched = self.state.read(cx).fetched_models.clone();
        let preferred = pick_preferred_model(&fetched, &["flash", "lite"])?;
        self.offered_model(&preferred, cx)
    }

    fn provided_models(&self, cx: &App) -> Vec<LanguageModel> {
        self.google_models(cx)
            .values()
            .map(language_model)
            .collect()
    }

    fn is_authenticated(&self, cx: &App) -> bool {
        self.state.read(cx).is_authenticated()
    }

    fn authenticate(&self, cx: &mut App) -> Task<Result<(), AuthenticateError>> {
        self.state.update(cx, |state, cx| state.authenticate(cx))
    }

    fn settings_view(&self, cx: &mut App) -> Option<ProviderSettingsView> {
        let state = self.state.read(cx);
        Some(ProviderSettingsView::ApiKey(ApiKeyConfiguration::new(
            state.api_key_state.has_key(),
            state.api_key_state.is_from_env_var(),
            state.api_key_state.env_var_name().clone(),
            "https://aistudio.google.com/app/apikey".into(),
        )))
    }

    fn set_api_key(&self, api_key: Option<String>, cx: &mut App) -> Task<Result<()>> {
        self.state
            .update(cx, |state, cx| state.set_api_key(api_key, cx))
    }
}

impl LanguageModelClient for GoogleLanguageModelProvider {
    fn stream_completion(
        &self,
        model: &LanguageModel,
        request: LanguageModelRequest,
        cx: &AsyncApp,
    ) -> BoxFuture<'static, Result<LanguageModelCompletionStream, LanguageModelCompletionError>>
    {
        let config = match cx.update(|cx| self.config(model, cx)) {
            Ok(config) => config,
            Err(error) => return async move { Err(error) }.boxed(),
        };
        let request_limiter = self.request_limiters.for_model(&model.id);
        let mut request = request;
        if request.max_output_tokens.is_some() {
            request.max_output_tokens =
                request.effective_max_output_tokens(config.max_output_tokens());
        }
        let request = match into_google(request, config.request_id().to_string(), config.mode()) {
            Ok(request) => request,
            Err(error) => return async move { Err(error.into()) }.boxed(),
        };
        let request = self.stream_google_request(request, cx);
        let future = request_limiter.stream(async move {
            let response = request.await?;
            Ok(GoogleEventMapper::new().map_stream(response))
        });
        async move { Ok(future.await?.boxed()) }.boxed()
    }
}

fn language_model(model: &google_ai::Model) -> LanguageModel {
    let default_level = model.default_thinking_level();
    LanguageModel {
        supports_tools: model.supports_tools(),
        supports_images: model.supports_images(),
        supports_thinking: model.supports_thinking(),
        supported_effort_levels: model
            .supported_thinking_levels()
            .iter()
            .map(|level| LanguageModelEffortLevel {
                name: level.name().into(),
                value: level.value().into(),
                is_default: Some(*level) == default_level,
            })
            .collect(),
        tool_choice_support: LanguageModelToolChoiceSupport::ALL,
        max_output_tokens: model.max_output_tokens(),
        ..LanguageModel::new(
            LanguageModelId::from(model.id().to_string()),
            LanguageModelName::from(model.display_name().to_string()),
            PROVIDER_ID,
            PROVIDER_NAME,
            format!("google/{}", model.request_id()),
            model.max_token_count(),
        )
    }
}

/// Pick the model id from `models` containing the earliest matching
/// substring in `preferred_substrings`. Within a single bucket the
/// lexicographically greatest id wins, which roughly corresponds to the
/// highest version since Gemini ids embed version numbers. Mirrors the
/// Anthropic provider's `pick_preferred_model`, using `contains` instead
/// of `starts_with` because the interesting discriminators (`pro`,
/// `flash`, `lite`) sit in the middle of Gemini ids.
fn pick_preferred_model(models: &[String], preferred_substrings: &[&str]) -> Option<String> {
    for needle in preferred_substrings {
        let candidate = models
            .iter()
            .filter(|id| id.contains(needle))
            .max_by(|a, b| a.cmp(b));
        if let Some(model) = candidate {
            return Some(model.clone());
        }
    }
    None
}

#[derive(Debug, Deserialize)]
struct ListModelsResponse {
    #[serde(default)]
    models: Vec<ListModelEntry>,
}

#[derive(Debug, Deserialize)]
struct ListModelEntry {
    #[serde(default)]
    name: String,
}

/// Fetch the model ids available to the current API key from Google's
/// `GET /v1beta/models` endpoint, which answers
/// `{"models": [{"name": "models/<id>", ...}]}`.
///
/// See https://ai.google.dev/api/models#method:-models.list.
async fn list_google_models(
    client: &dyn HttpClient,
    api_url: &str,
    api_key: &str,
    extra_headers: &CustomHeaders,
) -> Result<Vec<String>, LanguageModelCompletionError> {
    // Default base has no version suffix; a custom base that already ends in
    // `/v1beta` must not get it doubled (verified live: the endpoint is
    // `{root}/v1beta/models`).
    let base = api_url.trim_end_matches('/');
    let base = base.strip_suffix("/v1beta").unwrap_or(base);
    let uri = format!("{base}/v1beta/models");

    let request = HttpRequest::builder()
        .method(Method::GET)
        .uri(uri)
        .header("x-goog-api-key", api_key.trim())
        .header("Accept", "application/json")
        .extra_headers(extra_headers)
        .body(AsyncBody::default())
        .map_err(|error| LanguageModelCompletionError::BuildRequestBody {
            provider: GOOGLE_PROVIDER_NAME,
            error,
        })?;

    let host = request.uri().host().unwrap_or(api_url).to_owned();
    let mut response = client
        .send(request)
        .await
        .map_err(|error| LanguageModelCompletionError::HttpSend {
            provider: GOOGLE_PROVIDER_NAME,
            host,
            error,
        })?;

    let mut body = String::new();
    response
        .body_mut()
        .read_to_string(&mut body)
        .await
        .map_err(|error| LanguageModelCompletionError::ApiReadResponseError {
            provider: GOOGLE_PROVIDER_NAME,
            error,
        })?;

    if !response.status().is_success() {
        return Err(LanguageModelCompletionError::Other(anyhow::anyhow!(
            "failed to list Google models: {body}"
        )));
    }

    let parsed: ListModelsResponse = serde_json::from_str(&body).map_err(|error| {
        LanguageModelCompletionError::Other(anyhow::anyhow!(
            "failed to parse Google models response: {error}"
        ))
    })?;

    Ok(parsed
        .models
        .into_iter()
        .filter_map(|entry| {
            entry
                .name
                .strip_prefix("models/")
                .map(str::to_string)
        })
        .collect())
}

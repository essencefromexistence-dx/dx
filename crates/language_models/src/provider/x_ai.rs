use anyhow::Result;
use collections::BTreeMap;
use credentials_provider::CredentialsProvider;
use futures::{AsyncReadExt, FutureExt, StreamExt, future::BoxFuture};
use gpui::{App, AppContext, AsyncApp, Context, Entity, SharedString, Task};
use http_client::{
    AsyncBody, CustomHeaders, HttpClient, Method, Request as HttpRequest, RequestBuilderExt,
};
use language_model::chat_completion::{ChatCompletionEventMapper, ResponseStreamEvent};
use language_model::{
    ApiKeyConfiguration, ApiKeyState, AuthenticateError, EnvVar, IconOrSvg, LanguageModel,
    LanguageModelClient, LanguageModelCompletionError, LanguageModelCompletionStream,
    LanguageModelEffortLevel, LanguageModelId, LanguageModelName, LanguageModelProvider,
    LanguageModelProviderId, LanguageModelProviderName, LanguageModelProviderState,
    LanguageModelRequest, LanguageModelToolChoiceSupport, ModelRateLimiters, ProviderSettingsView,
    RateLimiter, env_var, unavailable_error,
};
pub use settings::XaiAvailableModel as AvailableModel;
use settings::{Settings, SettingsStore};
use std::sync::{Arc, LazyLock};
use serde::Deserialize;
use ui::IconName;
use x_ai::XAI_API_URL;

const PROVIDER_ID: LanguageModelProviderId = LanguageModelProviderId::new("x_ai");
const PROVIDER_NAME: LanguageModelProviderName = LanguageModelProviderName::new("xAI");

const API_KEY_ENV_VAR_NAME: &str = "XAI_API_KEY";
static API_KEY_ENV_VAR: LazyLock<EnvVar> = env_var!(API_KEY_ENV_VAR_NAME);

#[derive(Default, Clone, Debug, PartialEq)]
pub struct XAiSettings {
    pub api_url: String,
    pub available_models: Vec<AvailableModel>,
    pub custom_headers: CustomHeaders,
}

pub struct XAiLanguageModelProvider {
    http_client: Arc<dyn HttpClient>,
    state: Entity<State>,
    request_limiters: ModelRateLimiters,
}

pub struct State {
    api_key_state: ApiKeyState,
    credentials_provider: Arc<dyn CredentialsProvider>,
    http_client: Arc<dyn HttpClient>,
    fetched_models: Vec<String>,
    fetch_models_task: Option<Task<Result<(), LanguageModelCompletionError>>>,
}

impl State {
    fn is_authenticated(&self) -> bool {
        self.api_key_state.has_key()
    }

    fn set_api_key(&mut self, api_key: Option<String>, cx: &mut Context<Self>) -> Task<Result<()>> {
        let credentials_provider = self.credentials_provider.clone();
        let api_url = XAiLanguageModelProvider::api_url(cx);
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
        let api_url = XAiLanguageModelProvider::api_url(cx);
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

    fn fetch_models(
        &mut self,
        cx: &mut Context<Self>,
    ) -> Task<Result<(), LanguageModelCompletionError>> {
        let http_client = self.http_client.clone();
        let api_url = XAiLanguageModelProvider::api_url(cx);
        let Some(api_key) = self.api_key_state.key(&api_url) else {
            return Task::ready(Err(LanguageModelCompletionError::NoApiKey {
                provider: PROVIDER_NAME,
            }));
        };
        let extra_headers = XAiLanguageModelProvider::settings(cx)
            .custom_headers
            .clone();

        cx.spawn(async move |this, cx| {
            let models = list_models(
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
            .map_err(|e| LanguageModelCompletionError::Other(e))?;
            Ok(())
        })
    }

    fn restart_fetch_models_task(&mut self, cx: &mut Context<Self>) {
        let task = self.fetch_models(cx);
        self.fetch_models_task.replace(task);
    }
}

impl XAiLanguageModelProvider {
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

    /// Every model this provider offers, keyed by id: the live-fetched list,
    /// with settings entries added or overriding fetched ones.
    fn x_ai_models(&self, cx: &App) -> BTreeMap<String, x_ai::Model> {
        let mut models = BTreeMap::default();

        // Models reported by the `/models` endpoint are the primary source.
        // The list will be empty until authentication has succeeded and the
        // first fetch completes. The endpoint only returns ids, so fetched
        // models use conservative defaults matching the known grok
        // capabilities (tools, images, parallel tool calls); settings entries
        // below can override any of these fields per model.
        for id in &self.state.read(cx).fetched_models {
            models.insert(
                id.clone(),
                x_ai::Model::Custom {
                    name: id.clone(),
                    display_name: None,
                    max_tokens: 128_000,
                    max_output_tokens: None,
                    max_completion_tokens: None,
                    supports_images: Some(true),
                    supports_tools: Some(true),
                    parallel_tool_calls: Some(true),
                },
            );
        }

        for model in &Self::settings(cx).available_models {
            models.insert(
                model.name.clone(),
                x_ai::Model::Custom {
                    name: model.name.clone(),
                    display_name: model.display_name.clone(),
                    max_tokens: model.max_tokens,
                    max_output_tokens: model.max_output_tokens,
                    max_completion_tokens: model.max_completion_tokens,
                    supports_images: model.supports_images,
                    supports_tools: model.supports_tools,
                    parallel_tool_calls: model.parallel_tool_calls,
                },
            );
        }

        models
    }

    /// Describes the offered model with `id`.
    fn offered_model(&self, id: &str, cx: &App) -> Option<LanguageModel> {
        self.x_ai_models(cx).get(id).map(language_model)
    }

    /// The current configuration of `model`, if this provider still offers it.
    fn config(
        &self,
        model: &LanguageModel,
        cx: &App,
    ) -> Result<x_ai::Model, LanguageModelCompletionError> {
        self.x_ai_models(cx)
            .remove(model.id.0.as_ref())
            .ok_or_else(|| unavailable_error(model))
    }

    fn stream_chat_completion(
        &self,
        request_limiter: &RateLimiter,
        request: open_ai::Request,
        cx: &AsyncApp,
    ) -> BoxFuture<
        'static,
        Result<
            futures::stream::BoxStream<'static, Result<ResponseStreamEvent>>,
            LanguageModelCompletionError,
        >,
    > {
        let http_client = self.http_client.clone();

        let (api_key, api_url, extra_headers) = self.state.read_with(cx, |state, cx| {
            let api_url = XAiLanguageModelProvider::api_url(cx);
            let extra_headers = XAiLanguageModelProvider::settings(cx)
                .custom_headers
                .clone();
            (state.api_key_state.key(&api_url), api_url, extra_headers)
        });

        let future = request_limiter.stream(async move {
            let provider = PROVIDER_NAME;
            let Some(api_key) = api_key else {
                return Err(LanguageModelCompletionError::NoApiKey { provider });
            };
            let request = open_ai::stream_completion(
                http_client.as_ref(),
                provider.0.as_str(),
                &api_url,
                &api_key,
                request,
                &extra_headers,
            );
            let response = request.await?;
            Ok(response)
        });

        async move { Ok(future.await?.boxed()) }.boxed()
    }

    fn settings(cx: &App) -> &XAiSettings {
        &crate::AllLanguageModelSettings::get_global(cx).x_ai
    }

    fn api_url(cx: &App) -> SharedString {
        let api_url = &Self::settings(cx).api_url;
        if api_url.is_empty() {
            XAI_API_URL.into()
        } else {
            SharedString::new(api_url.as_str())
        }
    }
}

impl LanguageModelProviderState for XAiLanguageModelProvider {
    type ObservableEntity = State;

    fn observable_entity(&self) -> Option<Entity<Self::ObservableEntity>> {
        Some(self.state.clone())
    }
}

impl LanguageModelProvider for XAiLanguageModelProvider {
    fn id(&self) -> LanguageModelProviderId {
        PROVIDER_ID
    }

    fn name(&self) -> LanguageModelProviderName {
        PROVIDER_NAME
    }

    fn icon(&self) -> IconOrSvg {
        IconOrSvg::Icon(IconName::AiXAi)
    }

    fn default_model(&self, cx: &App) -> Option<LanguageModel> {
        let fetched = self.state.read(cx).fetched_models.clone();
        // Live-first: prefer the newest grok generation, then any fetched
        // model. Returning `None` until the fetch completes matches the
        // Anthropic provider's behavior.
        let preferred = pick_preferred_model(&fetched, &["grok-4", "grok-3", "grok-"])
            .or_else(|| fetched.iter().max().cloned())?;
        self.offered_model(&preferred, cx)
    }

    fn default_fast_model(&self, cx: &App) -> Option<LanguageModel> {
        let fetched = self.state.read(cx).fetched_models.clone();
        // xAI ships fast variants as suffixed ids (e.g. `grok-3-mini`,
        // `grok-4-fast`), so match on substring rather than prefix, then fall
        // back to any fetched model. `None` until the fetch completes.
        let preferred = pick_preferred_model_containing(&fetched, &["mini", "fast"])
            .or_else(|| fetched.iter().max().cloned())?;
        self.offered_model(&preferred, cx)
    }

    fn provided_models(&self, cx: &App) -> Vec<LanguageModel> {
        self.x_ai_models(cx).values().map(language_model).collect()
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
            "https://console.x.ai/team/default/api-keys".into(),
        )))
    }

    fn set_api_key(&self, api_key: Option<String>, cx: &mut App) -> Task<Result<()>> {
        self.state
            .update(cx, |state, cx| state.set_api_key(api_key, cx))
    }
}

impl LanguageModelClient for XAiLanguageModelProvider {
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
        let reasoning_effort = reasoning_effort_for_request(&request, &config);
        let request = match crate::provider::open_ai::into_open_ai(
            request,
            config.id(),
            config.supports_parallel_tool_calls(),
            config.supports_prompt_cache_key(),
            config.max_output_tokens(),
            crate::provider::open_ai::ChatCompletionMaxTokensParameter::MaxCompletionTokens,
            reasoning_effort,
            false,
        ) {
            Ok(request) => request,
            Err(error) => return async move { Err(error.into()) }.boxed(),
        };
        let completions = self.stream_chat_completion(&request_limiter, request, cx);
        let executor = cx.background_executor().clone();
        async move {
            let mapper = ChatCompletionEventMapper::new();
            Ok(language_model::stream_in_background(
                mapper.map_stream(completions.await?).boxed(),
                executor,
            ))
        }
        .boxed()
    }
}

fn x_ai_reasoning_efforts(model: &x_ai::Model) -> &'static [open_ai::ReasoningEffort] {
    match model {
        x_ai::Model::Grok43 => &[
            open_ai::ReasoningEffort::None,
            open_ai::ReasoningEffort::Low,
            open_ai::ReasoningEffort::Medium,
            open_ai::ReasoningEffort::High,
        ],
        x_ai::Model::Grok45 => &[
            open_ai::ReasoningEffort::Low,
            open_ai::ReasoningEffort::Medium,
            open_ai::ReasoningEffort::High,
        ],
        x_ai::Model::Grok46 | x_ai::Model::Grok47 => &[
            open_ai::ReasoningEffort::Low,
            open_ai::ReasoningEffort::Medium,
            open_ai::ReasoningEffort::High,
            open_ai::ReasoningEffort::XHigh,
        ],
        _ => &[],
    }
}

fn default_thinking_reasoning_effort(model: &x_ai::Model) -> Option<open_ai::ReasoningEffort> {
    match model {
        x_ai::Model::Grok43 => Some(open_ai::ReasoningEffort::Low),
        x_ai::Model::Grok45 | x_ai::Model::Grok46 | x_ai::Model::Grok47 => {
            Some(open_ai::ReasoningEffort::High)
        }
        _ => None,
    }
}

fn reasoning_effort_for_request(
    request: &LanguageModelRequest,
    model: &x_ai::Model,
) -> Option<open_ai::ReasoningEffort> {
    let supported_efforts = x_ai_reasoning_efforts(model);
    if supported_efforts.is_empty() {
        return None;
    }

    if request.thinking_allowed {
        request
            .thinking_effort
            .as_deref()
            .and_then(|effort| effort.parse::<open_ai::ReasoningEffort>().ok())
            .filter(|effort| supported_efforts.contains(effort))
            .filter(|effort| *effort != open_ai::ReasoningEffort::None)
            .or_else(|| default_thinking_reasoning_effort(model))
    } else if supported_efforts.contains(&open_ai::ReasoningEffort::None) {
        Some(open_ai::ReasoningEffort::None)
    } else {
        None
    }
}

fn supported_thinking_effort_levels(model: &x_ai::Model) -> Vec<LanguageModelEffortLevel> {
    let default_effort = default_thinking_reasoning_effort(model);
    x_ai_reasoning_efforts(model)
        .iter()
        .copied()
        .filter_map(|effort| {
            let (name, value) = match effort {
                open_ai::ReasoningEffort::None => return None,
                open_ai::ReasoningEffort::Minimal => ("Minimal", "minimal"),
                open_ai::ReasoningEffort::Low => ("Low", "low"),
                open_ai::ReasoningEffort::Medium => ("Medium", "medium"),
                open_ai::ReasoningEffort::High => ("High", "high"),
                open_ai::ReasoningEffort::XHigh => ("Extra High", "xhigh"),
                open_ai::ReasoningEffort::Max => return None, // Not supported by any xAI models
            };

            Some(LanguageModelEffortLevel {
                name: name.into(),
                value: value.into(),
                is_default: Some(effort) == default_effort,
                supports_disabling_thinking: None,
            })
        })
        .collect()
}

fn language_model(model: &x_ai::Model) -> LanguageModel {
    LanguageModel {
        supports_tools: model.supports_tool(),
        supports_images: model.supports_images(),
        supports_streaming_tools: true,
        tool_choice_support: LanguageModelToolChoiceSupport::ALL,
        supports_thinking: model.supports_reasoning_effort(),
        supported_effort_levels: supported_thinking_effort_levels(model).into(),
        max_output_tokens: model.max_output_tokens(),
        supports_split_token_display: true,
        ..LanguageModel::new(
            LanguageModelId::from(model.id().to_string()),
            LanguageModelName::from(model.display_name().to_string()),
            PROVIDER_ID,
            PROVIDER_NAME,
            format!("x_ai/{}", model.id()),
            model.max_token_count(),
        )
    }
}

/// Pick the model id from `models` that starts with the earliest matching
/// prefix in `preferred_prefixes`. Within a single prefix bucket the
/// lexicographically greatest id wins, which roughly corresponds to the
/// highest version since xAI ids embed version suffixes.
fn pick_preferred_model(models: &[String], preferred_prefixes: &[&str]) -> Option<String> {
    for prefix in preferred_prefixes {
        let candidate = models
            .iter()
            .filter(|m| m.starts_with(prefix))
            .max();
        if let Some(model) = candidate {
            return Some(model.clone());
        }
    }
    None
}

/// Variant of [`pick_preferred_model`] matching on substring instead of
/// prefix, for fast variants xAI ships as suffixed ids (e.g. `grok-3-mini`,
/// `grok-4-fast`).
fn pick_preferred_model_containing(
    models: &[String],
    preferred_substrings: &[&str],
) -> Option<String> {
    for substring in preferred_substrings {
        let candidate = models.iter().filter(|m| m.contains(substring)).max();
        if let Some(model) = candidate {
            return Some(model.clone());
        }
    }
    None
}

#[derive(Deserialize)]
struct ListModelsResponse {
    data: Vec<ListedModel>,
}

#[derive(Deserialize)]
struct ListedModel {
    id: String,
}

/// Fetch the model ids available to the current API key from xAI's
/// OpenAI-compatible `/models` endpoint.
///
/// See <https://docs.x.ai/docs/models>.
async fn list_models(
    client: &dyn HttpClient,
    api_url: &str,
    api_key: &str,
    extra_headers: &CustomHeaders,
) -> Result<Vec<String>, LanguageModelCompletionError> {
    let uri = format!("{api_url}/models");

    let request = HttpRequest::builder()
        .method(Method::GET)
        .uri(uri)
        .header("Authorization", format!("Bearer {}", api_key.trim()))
        .header("Accept", "application/json")
        .extra_headers(extra_headers)
        .body(AsyncBody::default())
        .map_err(|error| LanguageModelCompletionError::BuildRequestBody {
            provider: PROVIDER_NAME,
            error,
        })?;

    let host = request.uri().host().unwrap_or(api_url).to_owned();
    let mut response = client
        .send(request)
        .await
        .map_err(|error| LanguageModelCompletionError::HttpSend {
            provider: PROVIDER_NAME,
            host,
            error,
        })?;

    let mut body = String::new();
    response
        .body_mut()
        .read_to_string(&mut body)
        .await
        .map_err(|error| LanguageModelCompletionError::ApiReadResponseError {
            provider: PROVIDER_NAME,
            error,
        })?;

    if !response.status().is_success() {
        return Err(LanguageModelCompletionError::from_http_status(
            PROVIDER_NAME,
            response.status(),
            extract_error_message(&body),
            None,
        ));
    }

    let parsed: ListModelsResponse =
        serde_json::from_str(&body).map_err(|error| {
            LanguageModelCompletionError::DeserializeResponse {
                provider: PROVIDER_NAME,
                error,
            }
        })?;

    Ok(parsed.data.into_iter().map(|model| model.id).collect())
}

fn extract_error_message(body: &str) -> String {
    let json = match serde_json::from_str::<serde_json::Value>(body) {
        Ok(json) => json,
        Err(_) => return body.to_string(),
    };

    json.get("error")
        .and_then(|value| {
            value
                .get("message")
                .and_then(serde_json::Value::as_str)
                .or_else(|| value.as_str())
        })
        .or_else(|| json.get("message").and_then(serde_json::Value::as_str))
        .map(ToString::to_string)
        .unwrap_or_else(|| body.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pick_preferred_model_prefers_newest_grok_generation() {
        let models = vec![
            "grok-3".to_string(),
            "grok-4.3".to_string(),
            "grok-4.7".to_string(),
        ];

        assert_eq!(
            pick_preferred_model(&models, &["grok-4", "grok-3", "grok-"]),
            Some("grok-4.7".to_string())
        );
    }

    #[test]
    fn pick_preferred_model_falls_back_to_older_generations() {
        let models = vec!["grok-3-mini".to_string(), "grok-3".to_string()];

        assert_eq!(
            pick_preferred_model(&models, &["grok-4", "grok-3", "grok-"]),
            Some("grok-3-mini".to_string())
        );
    }

    #[test]
    fn pick_preferred_model_returns_none_without_models() {
        assert_eq!(
            pick_preferred_model(&[], &["grok-4", "grok-3", "grok-"]),
            None
        );
    }

    #[test]
    fn fast_pick_prefers_mini_then_fast_variants() {
        let models = vec![
            "grok-4.7".to_string(),
            "grok-4-fast".to_string(),
            "grok-3-mini".to_string(),
        ];

        assert_eq!(
            pick_preferred_model_containing(&models, &["mini", "fast"]),
            Some("grok-3-mini".to_string())
        );
    }

    #[test]
    fn fast_pick_falls_back_to_fast_when_no_mini() {
        let models = vec!["grok-4.7".to_string(), "grok-4-fast".to_string()];

        assert_eq!(
            pick_preferred_model_containing(&models, &["mini", "fast"]),
            Some("grok-4-fast".to_string())
        );
    }

    #[test]
    fn list_models_response_parses_ids() {
        let body = r#"{"data":[{"id":"grok-4.7"},{"id":"grok-3-mini"}]}"#;
        let parsed: ListModelsResponse =
            serde_json::from_str(body).expect("test fixture should parse");
        let ids: Vec<String> = parsed.data.into_iter().map(|model| model.id).collect();

        assert_eq!(ids, vec!["grok-4.7".to_string(), "grok-3-mini".to_string()]);
    }

    #[test]
    fn grok_43_supports_selectable_thinking_effort_levels() {
        let effort_levels = supported_thinking_effort_levels(&x_ai::Model::Grok43);
        let values = effort_levels
            .iter()
            .map(|level| level.value.as_ref())
            .collect::<Vec<_>>();

        assert_eq!(values, ["low", "medium", "high"]);
        assert_eq!(
            effort_levels
                .iter()
                .find(|level| level.is_default)
                .map(|level| level.value.as_ref()),
            Some("low")
        );
    }

    #[test]
    fn grok_43_request_uses_selected_reasoning_effort() {
        let request = LanguageModelRequest {
            thinking_allowed: true,
            thinking_effort: Some("high".to_string()),
            ..Default::default()
        };

        assert_eq!(
            reasoning_effort_for_request(&request, &x_ai::Model::Grok43),
            Some(open_ai::ReasoningEffort::High)
        );
    }

    #[test]
    fn grok_43_request_uses_none_when_thinking_is_disabled() {
        let request = LanguageModelRequest {
            thinking_allowed: false,
            ..Default::default()
        };

        assert_eq!(
            reasoning_effort_for_request(&request, &x_ai::Model::Grok43),
            Some(open_ai::ReasoningEffort::None)
        );
    }
}

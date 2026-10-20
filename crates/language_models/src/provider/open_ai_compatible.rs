use anyhow::Result;
use collections::BTreeMap;
use credentials_provider::CredentialsProvider;
use futures::{FutureExt, StreamExt, future::BoxFuture};
use gpui::{App, AppContext, AsyncApp, Entity, Task};
use http_client::{CustomHeaders, HttpClient};
use language_model::chat_completion::ChatCompletionEventMapper;
use language_model::{
    AuthenticateError, IconOrSvg, LanguageModel, LanguageModelClient, LanguageModelCompletionError,
    LanguageModelCompletionStream, LanguageModelEffortLevel, LanguageModelId, LanguageModelName,
    LanguageModelProvider, LanguageModelProviderId, LanguageModelProviderName,
    LanguageModelProviderState, LanguageModelRequest, LanguageModelToolChoiceSupport,
    ModelRateLimiters, ProviderSettingsView, RateLimiter, SubPageProviderSettings,
    unavailable_error,
};
use open_ai::{
    ResponseStreamEvent,
    responses::{Request as ResponseRequest, StreamEvent as ResponsesStreamEvent, stream_response},
    stream_completion,
};
use settings::Settings;
use std::sync::Arc;
use ui::IconName;

use crate::provider::api_compatible::{
    ApiCompatibleProviderConfigurationView, ApiCompatibleProviderSettings,
    ApiCompatibleProviderState, CompatibleModelsFetch, CompatibleModelsFetchAuth,
};
use crate::provider::open_ai::{OpenAiResponseEventMapper, into_open_ai, into_open_ai_response};
pub use settings::OpenAiCompatibleAvailableModel as AvailableModel;
pub use settings::OpenAiCompatibleModelCapabilities as ModelCapabilities;

const API_KEY_PLACEHOLDER: &str = "000000000000000000000000000000000000000000000000000";

#[derive(Default, Clone, Debug, PartialEq)]
pub struct OpenAiCompatibleSettings {
    pub api_url: String,
    pub available_models: Vec<AvailableModel>,
    pub custom_headers: CustomHeaders,
}

impl ApiCompatibleProviderSettings for OpenAiCompatibleSettings {
    fn api_url(&self) -> &str {
        &self.api_url
    }
}

pub type State = ApiCompatibleProviderState<OpenAiCompatibleSettings>;

pub struct OpenAiCompatibleLanguageModelProvider {
    id: LanguageModelProviderId,
    name: LanguageModelProviderName,
    http_client: Arc<dyn HttpClient>,
    state: Entity<State>,
    request_limiters: ModelRateLimiters,
}

impl OpenAiCompatibleLanguageModelProvider {
    pub fn new(
        id: Arc<str>,
        http_client: Arc<dyn HttpClient>,
        credentials_provider: Arc<dyn CredentialsProvider>,
        cx: &mut App,
    ) -> Self {
        let state = State::new(
            id.clone(),
            credentials_provider,
            |id, cx| {
                crate::AllLanguageModelSettings::get_global(cx)
                    .openai_compatible
                    .get(id)
            },
            http_client.clone(),
            // Live `/models` (OpenAI-compatible standard, verified live):
            // the picker reflects fetched ids; settings stay as fallback
            // and per-model override.
            Some(CompatibleModelsFetch {
                list_suffix: "/models",
                auth: CompatibleModelsFetchAuth::Bearer,
            }),
            cx,
        );

        Self {
            id: id.clone().into(),
            name: id.into(),
            http_client,
            state,
            request_limiters: ModelRateLimiters::default(),
        }
    }

    /// Every model this provider offers, in settings order.
    fn available_models<'a>(&self, cx: &'a App) -> &'a [AvailableModel] {
        &self.state.read(cx).settings.available_models
    }

    /// Live `/models` ids as settings-shaped entries (conservative defaults:
    /// same capabilities as a manually added model).
    fn fetched_models(&self, cx: &App) -> Vec<AvailableModel> {
        let mut ids = self.state.read(cx).fetched_models.clone();
        ids.sort();
        ids.dedup();
        ids.into_iter()
            .map(|name| AvailableModel {
                name,
                display_name: None,
                max_tokens: 128_000,
                max_output_tokens: None,
                max_completion_tokens: None,
                reasoning_effort: None,
                capabilities: ModelCapabilities::default(),
            })
            .collect()
    }

    /// Fetched models first, settings entries overriding/extending by name.
    /// The picker therefore reflects the live gateway inventory; the
    /// hardcoded settings list is fallback and per-model override only.
    fn combined_models(&self, cx: &App) -> BTreeMap<String, AvailableModel> {
        let mut models = BTreeMap::default();
        for model in self.fetched_models(cx) {
            models.insert(model.name.clone(), model);
        }
        for model in self.available_models(cx) {
            models.insert(model.name.clone(), model.clone());
        }
        models
    }

    fn language_model(&self, model: &AvailableModel) -> LanguageModel {
        LanguageModel {
            supports_tools: model.capabilities.tools,
            supports_images: model.capabilities.images,
            tool_choice_support: LanguageModelToolChoiceSupport {
                auto: model.capabilities.tools,
                any: model.capabilities.tools,
                none: true,
            },
            supports_streaming_tools: true,
            supports_thinking: default_thinking_reasoning_effort(model).is_some(),
            supported_effort_levels: supported_thinking_effort_levels(model).into(),
            supports_split_token_display: true,
            max_output_tokens: model.max_output_tokens,
            ..LanguageModel::new(
                LanguageModelId::from(model.name.clone()),
                LanguageModelName::from(
                    model
                        .display_name
                        .clone()
                        .unwrap_or_else(|| model.name.clone()),
                ),
                self.id.clone(),
                self.name.clone(),
                format!("openai/{}", model.name),
                model.max_tokens,
            )
        }
    }

    /// The current configuration of `model`, if this provider still offers it.
    fn config(
        &self,
        model: &LanguageModel,
        cx: &App,
    ) -> Result<AvailableModel, LanguageModelCompletionError> {
        self.combined_models(cx)
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

        let (api_key, api_url, extra_headers) = self.state.read_with(cx, |state, _cx| {
            let api_url = &state.settings.api_url;
            (
                state.api_key_state.key(api_url),
                state.settings.api_url.clone(),
                state.settings.custom_headers.clone(),
            )
        });

        let provider = self.name.clone();
        let future = request_limiter.stream(async move {
            let Some(api_key) = api_key else {
                return Err(LanguageModelCompletionError::NoApiKey { provider });
            };
            let request = stream_completion(
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

    fn stream_response(
        &self,
        request_limiter: &RateLimiter,
        request: ResponseRequest,
        cx: &AsyncApp,
    ) -> BoxFuture<'static, Result<futures::stream::BoxStream<'static, Result<ResponsesStreamEvent>>>>
    {
        let http_client = self.http_client.clone();

        let (api_key, api_url, extra_headers) = self.state.read_with(cx, |state, _cx| {
            let api_url = &state.settings.api_url;
            (
                state.api_key_state.key(api_url),
                state.settings.api_url.clone(),
                state.settings.custom_headers.clone(),
            )
        });

        let provider = self.name.clone();
        let future = request_limiter.stream(async move {
            let Some(api_key) = api_key else {
                return Err(LanguageModelCompletionError::NoApiKey { provider });
            };
            let request = stream_response(
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
}

impl LanguageModelProviderState for OpenAiCompatibleLanguageModelProvider {
    type ObservableEntity = State;

    fn observable_entity(&self) -> Option<Entity<Self::ObservableEntity>> {
        Some(self.state.clone())
    }
}

impl LanguageModelProvider for OpenAiCompatibleLanguageModelProvider {
    fn id(&self) -> LanguageModelProviderId {
        self.id.clone()
    }

    fn name(&self) -> LanguageModelProviderName {
        self.name.clone()
    }

    fn icon(&self) -> IconOrSvg {
        IconOrSvg::Icon(IconName::AiOpenAiCompat)
    }

    fn default_model(&self, cx: &App) -> Option<LanguageModel> {
        // Live-first: the first fetched gateway model, else the first
        // settings entry (offline fallback).
        let combined = self.combined_models(cx);
        let mut fetched: Vec<&String> = self
            .state
            .read(cx)
            .fetched_models
            .iter()
            .filter(|name| combined.contains_key(*name))
            .collect();
        fetched.sort();
        fetched
            .first()
            .map(|name| name.as_str())
            .or_else(|| combined.keys().next().map(String::as_str))
            .and_then(|name| combined.get(name))
            .map(|model| self.language_model(model))
    }

    fn default_fast_model(&self, _cx: &App) -> Option<LanguageModel> {
        None
    }

    fn provided_models(&self, cx: &App) -> Vec<LanguageModel> {
        self.combined_models(cx)
            .values()
            .map(|model| self.language_model(model))
            .collect()
    }

    fn is_authenticated(&self, cx: &App) -> bool {
        self.state.read(cx).is_authenticated()
    }

    fn authenticate(&self, cx: &mut App) -> Task<Result<(), AuthenticateError>> {
        self.state.update(cx, |state, cx| state.authenticate(cx))
    }

    fn settings_view(&self, _cx: &mut App) -> Option<ProviderSettingsView> {
        let state = self.state.clone();
        Some(ProviderSettingsView::SubPage(SubPageProviderSettings::new(
            move |window, cx| {
                cx.new(|cx| {
                    ApiCompatibleProviderConfigurationView::new(
                        state.clone(),
                        "OpenAI",
                        API_KEY_PLACEHOLDER,
                        window,
                        cx,
                    )
                })
                .into()
            },
        )))
    }

    fn set_api_key(&self, api_key: Option<String>, cx: &mut App) -> Task<Result<()>> {
        self.state
            .update(cx, |state, cx| state.set_api_key(api_key, cx))
    }
}

impl LanguageModelClient for OpenAiCompatibleLanguageModelProvider {
    fn stream_completion(
        &self,
        model: &LanguageModel,
        mut request: LanguageModelRequest,
        cx: &AsyncApp,
    ) -> BoxFuture<'static, Result<LanguageModelCompletionStream, LanguageModelCompletionError>>
    {
        let config = match cx.update(|cx| self.config(model, cx)) {
            Ok(config) => config,
            Err(error) => return async move { Err(error) }.boxed(),
        };
        let request_limiter = self.request_limiters.for_model(&model.id);
        // `speed` can leak in from a parent thread's model; this provider never
        // supports fast mode, and arbitrary compatible endpoints reject `service_tier`.
        request.speed = None;

        if config.capabilities.chat_completions {
            let reasoning_effort = chat_completion_reasoning_effort(&request, &config);
            let request = match into_open_ai(
                request,
                &config.name,
                config.capabilities.parallel_tool_calls,
                config.capabilities.prompt_cache_key,
                config.max_output_tokens,
                chat_completion_max_tokens_parameter(&config),
                reasoning_effort,
                config.capabilities.interleaved_reasoning,
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
        } else {
            disable_response_thinking_for_none_effort(&mut request, &config);
            let request = match into_open_ai_response(
                request,
                &config.name,
                config.capabilities.parallel_tool_calls,
                config.capabilities.prompt_cache_key,
                config.max_output_tokens,
                default_thinking_reasoning_effort(&config),
                supports_none_reasoning_effort(&config),
                &self.id,
            ) {
                Ok(request) => request,
                Err(error) => return async move { Err(error.into()) }.boxed(),
            };
            let completions = self.stream_response(&request_limiter, request, cx);
            let compaction_state_owner = self.id.clone();
            let executor = cx.background_executor().clone();
            async move {
                let mapper = OpenAiResponseEventMapper::new(compaction_state_owner);
                Ok(language_model::stream_in_background(
                    mapper.map_stream(completions.await?).boxed(),
                    executor,
                ))
            }
            .boxed()
        }
    }
}

fn default_thinking_reasoning_effort(model: &AvailableModel) -> Option<open_ai::ReasoningEffort> {
    model
        .reasoning_effort
        .filter(|effort| *effort != open_ai::ReasoningEffort::None)
}

fn supported_thinking_effort_levels(model: &AvailableModel) -> Vec<LanguageModelEffortLevel> {
    let Some(default_effort) = default_thinking_reasoning_effort(model) else {
        return Vec::new();
    };

    open_ai::ReasoningEffort::OPENAI_COMPATIBLE_SELECTABLE
        .into_iter()
        .map(|effort| LanguageModelEffortLevel {
            name: effort.label().into(),
            value: effort.value().into(),
            is_default: effort == default_effort,
            supports_disabling_thinking: None,
        })
        .collect()
}

fn selected_thinking_reasoning_effort(
    request: &LanguageModelRequest,
) -> Option<open_ai::ReasoningEffort> {
    request
        .thinking_effort
        .as_deref()
        .and_then(|effort| effort.parse::<open_ai::ReasoningEffort>().ok())
        .filter(|effort| *effort != open_ai::ReasoningEffort::None)
}

fn chat_completion_max_tokens_parameter(
    model: &AvailableModel,
) -> crate::provider::open_ai::ChatCompletionMaxTokensParameter {
    if model.capabilities.max_tokens_parameter {
        crate::provider::open_ai::ChatCompletionMaxTokensParameter::MaxTokens
    } else {
        crate::provider::open_ai::ChatCompletionMaxTokensParameter::MaxCompletionTokens
    }
}

fn supports_none_reasoning_effort(model: &AvailableModel) -> bool {
    model.reasoning_effort.is_some()
}

fn chat_completion_reasoning_effort(
    request: &LanguageModelRequest,
    model: &AvailableModel,
) -> Option<open_ai::ReasoningEffort> {
    if model.reasoning_effort == Some(open_ai::ReasoningEffort::None) {
        return Some(open_ai::ReasoningEffort::None);
    }

    if request.thinking_allowed {
        selected_thinking_reasoning_effort(request)
            .or_else(|| default_thinking_reasoning_effort(model))
    } else if supports_none_reasoning_effort(model) {
        Some(open_ai::ReasoningEffort::None)
    } else {
        None
    }
}

fn disable_response_thinking_for_none_effort(
    request: &mut LanguageModelRequest,
    model: &AvailableModel,
) {
    if model.reasoning_effort == Some(open_ai::ReasoningEffort::None) {
        request.thinking_allowed = false;
        request.thinking_effort = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_json::json;

    fn available_model(reasoning_effort: Option<open_ai::ReasoningEffort>) -> AvailableModel {
        AvailableModel {
            name: "custom-model".to_string(),
            display_name: None,
            max_tokens: 128_000,
            max_output_tokens: None,
            max_completion_tokens: None,
            reasoning_effort,
            capabilities: ModelCapabilities {
                chat_completions: false,
                ..Default::default()
            },
        }
    }

    #[test]
    fn configured_reasoning_effort_supports_thinking() {
        assert_eq!(
            default_thinking_reasoning_effort(&available_model(Some(
                open_ai::ReasoningEffort::High
            ))),
            Some(open_ai::ReasoningEffort::High)
        );
    }

    #[test]
    fn missing_or_none_reasoning_effort_does_not_support_thinking() {
        assert_eq!(
            default_thinking_reasoning_effort(&available_model(None)),
            None
        );
        assert_eq!(
            default_thinking_reasoning_effort(&available_model(Some(
                open_ai::ReasoningEffort::None
            ))),
            None
        );
    }

    #[test]
    fn supported_thinking_effort_levels_use_configured_effort_as_default() {
        let effort_levels = supported_thinking_effort_levels(&available_model(Some(
            open_ai::ReasoningEffort::High,
        )));
        let values = effort_levels
            .iter()
            .map(|level| level.value.as_ref())
            .collect::<Vec<_>>();

        assert_eq!(values, ["minimal", "low", "medium", "high", "xhigh", "max"]);
        assert_eq!(
            effort_levels
                .iter()
                .find(|level| level.is_default)
                .map(|level| level.value.as_ref()),
            Some("high")
        );
    }

    #[test]
    fn supported_thinking_effort_levels_hide_missing_or_none_effort() {
        assert!(supported_thinking_effort_levels(&available_model(None)).is_empty());
        assert!(
            supported_thinking_effort_levels(&available_model(Some(
                open_ai::ReasoningEffort::None
            )))
            .is_empty()
        );
    }

    #[test]
    fn chat_completion_reasoning_effort_honors_request_and_configured_effort() {
        let model = available_model(Some(open_ai::ReasoningEffort::Medium));
        let mut request = LanguageModelRequest {
            thinking_allowed: true,
            ..Default::default()
        };

        assert_eq!(
            chat_completion_reasoning_effort(&request, &model),
            Some(open_ai::ReasoningEffort::Medium)
        );

        request.thinking_effort = Some("high".to_string());
        assert_eq!(
            chat_completion_reasoning_effort(&request, &model),
            Some(open_ai::ReasoningEffort::High)
        );

        request.thinking_effort = Some("not-supported".to_string());
        assert_eq!(
            chat_completion_reasoning_effort(&request, &model),
            Some(open_ai::ReasoningEffort::Medium)
        );

        request.thinking_allowed = false;
        assert_eq!(
            chat_completion_reasoning_effort(&request, &model),
            Some(open_ai::ReasoningEffort::None)
        );
    }

    #[test]
    fn chat_completion_reasoning_effort_omits_missing_effort() {
        let model = available_model(None);
        let request = LanguageModelRequest {
            thinking_allowed: false,
            ..Default::default()
        };

        assert_eq!(chat_completion_reasoning_effort(&request, &model), None);
    }

    #[test]
    fn chat_completion_reasoning_effort_preserves_explicit_none() {
        let model = available_model(Some(open_ai::ReasoningEffort::None));
        let request = LanguageModelRequest {
            thinking_allowed: true,
            thinking_effort: Some("high".to_string()),
            ..Default::default()
        };

        assert_eq!(
            chat_completion_reasoning_effort(&request, &model),
            Some(open_ai::ReasoningEffort::None)
        );
    }

    #[test]
    fn chat_completion_max_tokens_parameter_defaults_to_max_completion_tokens() {
        let model = available_model(Some(open_ai::ReasoningEffort::Medium));

        assert_eq!(
            chat_completion_max_tokens_parameter(&model),
            crate::provider::open_ai::ChatCompletionMaxTokensParameter::MaxCompletionTokens
        );
    }

    #[test]
    fn chat_completion_max_tokens_parameter_uses_max_tokens_when_configured() {
        let mut model = available_model(Some(open_ai::ReasoningEffort::Medium));
        model.capabilities.max_tokens_parameter = true;

        assert_eq!(
            chat_completion_max_tokens_parameter(&model),
            crate::provider::open_ai::ChatCompletionMaxTokensParameter::MaxTokens
        );
    }

    #[test]
    fn response_request_includes_reasoning_when_effort_is_configured() {
        let model = available_model(Some(open_ai::ReasoningEffort::High));
        let request = LanguageModelRequest {
            thinking_allowed: true,
            ..Default::default()
        };

        let request = into_open_ai_response(
            request,
            &model.name,
            model.capabilities.parallel_tool_calls,
            model.capabilities.prompt_cache_key,
            model.max_output_tokens,
            default_thinking_reasoning_effort(&model),
            supports_none_reasoning_effort(&model),
            &LanguageModelProviderId::new("test-compatible-provider"),
        )
        .unwrap();
        let serialized = serde_json::to_value(request).unwrap();

        assert_eq!(
            serialized["reasoning"],
            json!({ "effort": "high", "summary": "auto" })
        );
        assert_eq!(
            serialized["include"],
            json!(["reasoning.encrypted_content"])
        );
    }

    #[test]
    fn response_request_omits_reasoning_when_effort_is_missing() {
        let model = available_model(None);
        let request = LanguageModelRequest {
            thinking_allowed: true,
            ..Default::default()
        };

        let request = into_open_ai_response(
            request,
            &model.name,
            model.capabilities.parallel_tool_calls,
            model.capabilities.prompt_cache_key,
            model.max_output_tokens,
            default_thinking_reasoning_effort(&model),
            supports_none_reasoning_effort(&model),
            &LanguageModelProviderId::new("test-compatible-provider"),
        )
        .unwrap();
        let serialized = serde_json::to_value(request).unwrap();

        assert_eq!(serialized.get("reasoning"), None);
        assert_eq!(serialized.get("include"), None);
    }

    #[test]
    fn chat_completion_request_includes_selected_reasoning_effort() {
        let mut model = available_model(Some(open_ai::ReasoningEffort::Medium));
        model.capabilities.chat_completions = true;
        let request = LanguageModelRequest {
            thinking_allowed: true,
            thinking_effort: Some("high".to_string()),
            ..Default::default()
        };
        let reasoning_effort = chat_completion_reasoning_effort(&request, &model);

        let request = into_open_ai(
            request,
            &model.name,
            model.capabilities.parallel_tool_calls,
            model.capabilities.prompt_cache_key,
            model.max_output_tokens,
            chat_completion_max_tokens_parameter(&model),
            reasoning_effort,
            model.capabilities.interleaved_reasoning,
        )
        .unwrap();
        let serialized = serde_json::to_value(request).unwrap();

        assert_eq!(serialized["reasoning_effort"], json!("high"));
    }

    #[test]
    fn configured_reasoning_effort_supports_none_reasoning_effort() {
        assert!(supports_none_reasoning_effort(&available_model(Some(
            open_ai::ReasoningEffort::Medium
        ))));
        assert!(supports_none_reasoning_effort(&available_model(Some(
            open_ai::ReasoningEffort::None
        ))));
        assert!(!supports_none_reasoning_effort(&available_model(None)));
    }

    #[test]
    fn response_thinking_effort_preserves_explicit_none() {
        let model = available_model(Some(open_ai::ReasoningEffort::None));
        let mut request = LanguageModelRequest {
            thinking_allowed: true,
            thinking_effort: Some("high".to_string()),
            ..Default::default()
        };

        disable_response_thinking_for_none_effort(&mut request, &model);
        assert!(!request.thinking_allowed);
        assert_eq!(request.thinking_effort, None);
    }

    static TEST_SETTINGS: std::sync::LazyLock<OpenAiCompatibleSettings> =
        std::sync::LazyLock::new(|| OpenAiCompatibleSettings {
            api_url: "https://gateway.example/v1".to_string(),
            available_models: vec![
                AvailableModel {
                    name: "settings-only".to_string(),
                    display_name: Some("Settings Only".to_string()),
                    max_tokens: 64_000,
                    max_output_tokens: None,
                    max_completion_tokens: None,
                    reasoning_effort: None,
                    capabilities: ModelCapabilities::default(),
                },
                AvailableModel {
                    name: "overlap".to_string(),
                    display_name: Some("From Settings".to_string()),
                    max_tokens: 32_000,
                    max_output_tokens: None,
                    max_completion_tokens: None,
                    reasoning_effort: None,
                    capabilities: ModelCapabilities::default(),
                },
            ],
            custom_headers: CustomHeaders::default(),
        });

    fn resolve_test_settings<'a>(
        _: &'a str,
        _: &'a App,
    ) -> Option<&'a OpenAiCompatibleSettings> {
        Some(&TEST_SETTINGS)
    }

    struct FakeCredentialsProvider;

    impl CredentialsProvider for FakeCredentialsProvider {
        fn read_credentials<'a>(
            &'a self,
            _url: &'a str,
            _cx: &'a gpui::AsyncApp,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<Option<(String, Vec<u8>)>>> + 'a>,
        > {
            Box::pin(async { Ok(None) })
        }

        fn write_credentials<'a>(
            &'a self,
            _url: &'a str,
            _username: &'a str,
            _password: &'a [u8],
            _cx: &'a gpui::AsyncApp,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + 'a>> {
            Box::pin(async { Ok(()) })
        }

        fn delete_credentials<'a>(
            &'a self,
            _url: &'a str,
            _cx: &'a gpui::AsyncApp,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + 'a>> {
            Box::pin(async { Ok(()) })
        }
    }

    #[gpui::test]
    async fn compatible_models_fetch_parses_openai_list_shape(
        _cx: &mut gpui::TestAppContext,
    ) {
        use http_client::{FakeHttpClient, Response};

        let client = FakeHttpClient::create(|request| async move {
            assert_eq!(
                request
                    .headers()
                    .get("authorization")
                    .and_then(|value| value.to_str().ok()),
                Some("Bearer test-key")
            );
            Ok(Response::builder()
                .status(200)
                .body(r#"{"data": [{"id": "b-live"}, {"id": "a-live"}]}"#.into())
                .expect("failed to build fake list models response"))
        });
        let ids = crate::provider::api_compatible::fetch_compatible_models(
            client.as_ref(),
            "https://gateway.example/v1",
            "test-key",
            crate::provider::api_compatible::CompatibleModelsFetch {
                list_suffix: "/models",
                auth: crate::provider::api_compatible::CompatibleModelsFetchAuth::Bearer,
            },
        )
        .await
        .expect("fetch should parse the canned list response");
        assert_eq!(ids, vec!["b-live".to_string(), "a-live".to_string()]);
    }

    #[gpui::test]
    async fn fetched_models_merge_with_settings_and_lead_defaults(
        cx: &mut gpui::TestAppContext,
    ) {
        use http_client::{FakeHttpClient, Response};

        let http_client: Arc<dyn HttpClient> = FakeHttpClient::create(|_| async move {
            Ok(Response::builder()
                .status(200)
                .body(r#"{"data": [{"id": "overlap"}, {"id": "live-model"}]}"#.into())
                .expect("failed to build fake list models response"))
        });
        let id: Arc<str> = "test-gateway".into();
        let (provider, state) = cx.update(|cx| {
            let state = State::new(
                id.clone(),
                Arc::new(FakeCredentialsProvider),
                resolve_test_settings,
                http_client.clone(),
                Some(crate::provider::api_compatible::CompatibleModelsFetch {
                    list_suffix: "/models",
                    auth: crate::provider::api_compatible::CompatibleModelsFetchAuth::Bearer,
                }),
                cx,
            );
            let provider = OpenAiCompatibleLanguageModelProvider {
                id: id.clone().into(),
                name: id.into(),
                http_client,
                state: state.clone(),
                request_limiters: ModelRateLimiters::default(),
            };
            (provider, state)
        });

        // Seed the live inventory directly (the HTTP fetch path is covered
        // by compatible_models_fetch_parses_openai_list_shape above; here
        // the merge/default/config behavior is under test).
        state.update(cx, |state, _| {
            state.fetched_models = vec!["overlap".to_string(), "live-model".to_string()];
        });

        let (provided_ids, default_id, overlap_name) = cx.read(|cx| {
            let provided = provider.provided_models(cx);
            let ids = provided
                .iter()
                .map(|model| model.id().0.to_string())
                .collect::<Vec<_>>();
            let default_id = provider
                .default_model(cx)
                .map(|model| model.id().0.to_string());
            let overlap_name = provided
                .iter()
                .find(|model| model.id().0.as_ref() == "overlap")
                .map(|model| model.name().0.to_string());
            (ids, default_id, overlap_name)
        });

        // Live ids join the hardcoded settings list; settings win on collision.
        assert_eq!(provided_ids, vec!["live-model", "overlap", "settings-only"]);
        assert_eq!(overlap_name.as_deref(), Some("From Settings"));
        // Default leads with the live inventory, not the settings list.
        assert_eq!(default_id.as_deref(), Some("live-model"));
        // A live-only model resolves for completion.
        cx.read(|cx| {
            assert!(provider
                .config(
                    &provider
                        .provided_models(cx)
                        .into_iter()
                        .find(|model| model.id().0.as_ref() == "live-model")
                        .unwrap(),
                    cx
                )
                .is_ok());
        });
    }
}

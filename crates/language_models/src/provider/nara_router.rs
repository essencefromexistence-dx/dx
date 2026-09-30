use std::sync::{Arc, LazyLock};

use anyhow::Result;
use credentials_provider::CredentialsProvider;
use futures::{FutureExt, StreamExt, future::BoxFuture};
use gpui::{App, AppContext, AsyncApp, Context, Entity, SharedString, Task};
use http_client::{CustomHeaders, HttpClient};
use language_model::{
    ApiKeyConfiguration, ApiKeyState, AuthenticateError, EnvVar, IconOrSvg, LanguageModel,
    LanguageModelCompletionError, LanguageModelCompletionStream, LanguageModelId,
    LanguageModelName, LanguageModelProvider, LanguageModelProviderId, LanguageModelProviderName,
    LanguageModelProviderState, LanguageModelRequest, LanguageModelToolChoiceSupport,
    ModelRateLimiters, NARA_ROUTER_PROVIDER_ID, NARA_ROUTER_PROVIDER_NAME, ProviderSettingsView,
    RateLimiter, env_var, stream_in_background, unavailable_error,
};
use language_model::chat_completion::ChatCompletionEventMapper;
use open_ai::completion::{
    ChatCompletionMaxTokensParameter, OpenAiResponseEventMapper, into_open_ai,
    into_open_ai_response,
};
use open_ai::{
    ResponseStreamEvent,
    responses::{Request as ResponseRequest, StreamEvent as ResponsesStreamEvent, stream_response},
    stream_completion,
};
use settings::{NaraRouterAvailableModel as AvailableModel, Settings, SettingsStore};
use ui::IconName;

pub const NARA_ROUTER_API_URL: &str = "https://router.bynara.id/v1";

const API_KEY_ENV_VAR_NAME: &str = "NARA_ROUTER_API_KEY";
static API_KEY_ENV_VAR: LazyLock<EnvVar> = env_var!(API_KEY_ENV_VAR_NAME);

#[derive(Default, Clone, Debug, PartialEq)]
pub struct NaraRouterSettings {
    pub api_url: String,
    pub available_models: Vec<AvailableModel>,
    pub custom_headers: CustomHeaders,
}

pub struct NaraRouterLanguageModelProvider {
    http_client: Arc<dyn HttpClient>,
    state: Entity<State>,
    request_limiters: ModelRateLimiters,
}

pub struct State {
    api_key_state: ApiKeyState,
    credentials_provider: Arc<dyn CredentialsProvider>,
}

impl State {
    fn is_authenticated(&self) -> bool {
        self.api_key_state.has_key()
    }

    fn set_api_key(&mut self, api_key: Option<String>, cx: &mut Context<Self>) -> Task<Result<()>> {
        let credentials_provider = self.credentials_provider.clone();
        let api_url = NaraRouterLanguageModelProvider::api_url(cx);
        self.api_key_state.store(
            api_url,
            api_key,
            |this| &mut this.api_key_state,
            credentials_provider,
            cx,
        )
    }

    fn authenticate(&mut self, cx: &mut Context<Self>) -> Task<Result<(), AuthenticateError>> {
        let credentials_provider = self.credentials_provider.clone();
        let api_url = NaraRouterLanguageModelProvider::api_url(cx);
        self.api_key_state.load_if_needed(
            api_url,
            |this| &mut this.api_key_state,
            credentials_provider,
            cx,
        )
    }
}

impl NaraRouterLanguageModelProvider {
    pub fn new(
        http_client: Arc<dyn HttpClient>,
        credentials_provider: Arc<dyn CredentialsProvider>,
        cx: &mut App,
    ) -> Self {
        let state = cx.new(|cx| {
            cx.observe_global::<SettingsStore>(|this: &mut State, cx| {
                let credentials_provider = this.credentials_provider.clone();
                let api_url = Self::api_url(cx);
                this.api_key_state.handle_url_change(
                    api_url,
                    |this| &mut this.api_key_state,
                    credentials_provider,
                    cx,
                );
                cx.notify();
            })
            .detach();
            State {
                api_key_state: ApiKeyState::new(Self::api_url(cx), (*API_KEY_ENV_VAR).clone()),
                credentials_provider,
            }
        });

        Self {
            http_client,
            state,
            request_limiters: ModelRateLimiters::default(),
        }
    }

    fn settings(cx: &App) -> &NaraRouterSettings {
        &crate::AllLanguageModelSettings::get_global(cx).nara_router
    }

    fn api_url(cx: &App) -> SharedString {
        let api_url = &Self::settings(cx).api_url;
        if api_url.is_empty() {
            NARA_ROUTER_API_URL.into()
        } else {
            SharedString::new(api_url.as_str())
        }
    }

    /// The current configuration of `model`, if this provider still offers it.
    fn config(
        &self,
        model: &LanguageModel,
        cx: &App,
    ) -> Result<AvailableModel, LanguageModelCompletionError> {
        Self::settings(cx)
            .available_models
            .iter()
            .find(|model_config| model_config.name == model.id.0.as_ref())
            .cloned()
            .ok_or_else(|| unavailable_error(model))
    }

    fn stream_chat_completion(
        &self,
        request_limiter: &RateLimiter,
        request: open_ai::Request,
        cx: &AsyncApp,
    ) -> BoxFuture<'static, Result<futures::stream::BoxStream<'static, Result<ResponseStreamEvent>>>>
    {
        let http_client = self.http_client.clone();

        let (api_key, api_url, extra_headers) = self.state.read_with(cx, |state, cx| {
            let api_url = NaraRouterLanguageModelProvider::api_url(cx);
            let extra_headers = NaraRouterLanguageModelProvider::settings(cx)
                .custom_headers
                .clone();
            (state.api_key_state.key(&api_url), api_url, extra_headers)
        });

        let future = request_limiter.stream(async move {
            let provider_name = NARA_ROUTER_PROVIDER_NAME;
            let Some(api_key) = api_key else {
                return Err(LanguageModelCompletionError::NoApiKey {
                    provider: provider_name,
                });
            };
            let request = stream_completion(
                http_client.as_ref(),
                provider_name.0.as_ref(),
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

        let (api_key, api_url, extra_headers) = self.state.read_with(cx, |state, cx| {
            let api_url = NaraRouterLanguageModelProvider::api_url(cx);
            let extra_headers = NaraRouterLanguageModelProvider::settings(cx)
                .custom_headers
                .clone();
            (state.api_key_state.key(&api_url), api_url, extra_headers)
        });

        let future = request_limiter.stream(async move {
            let provider_name = NARA_ROUTER_PROVIDER_NAME;
            let Some(api_key) = api_key else {
                return Err(LanguageModelCompletionError::NoApiKey {
                    provider: provider_name,
                });
            };
            let request = stream_response(
                http_client.as_ref(),
                provider_name.0.as_ref(),
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

impl LanguageModelProviderState for NaraRouterLanguageModelProvider {
    type ObservableEntity = State;

    fn observable_entity(&self) -> Option<Entity<Self::ObservableEntity>> {
        Some(self.state.clone())
    }
}

impl LanguageModelProvider for NaraRouterLanguageModelProvider {
    fn id(&self) -> LanguageModelProviderId {
        NARA_ROUTER_PROVIDER_ID
    }

    fn name(&self) -> LanguageModelProviderName {
        NARA_ROUTER_PROVIDER_NAME
    }

    fn icon(&self) -> IconOrSvg {
        IconOrSvg::Icon(IconName::AiOpenAiCompat)
    }

    fn default_model(&self, cx: &App) -> Option<LanguageModel> {
        Self::settings(cx)
            .available_models
            .first()
            .map(language_model)
    }

    fn default_fast_model(&self, _cx: &App) -> Option<LanguageModel> {
        None
    }

    fn provided_models(&self, cx: &App) -> Vec<LanguageModel> {
        Self::settings(cx)
            .available_models
            .iter()
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
            "https://router.naraya.ai".into(),
        )))
    }

    fn set_api_key(&self, api_key: Option<String>, cx: &mut App) -> Task<Result<()>> {
        self.state
            .update(cx, |state, cx| state.set_api_key(api_key, cx))
    }

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
        if config.capabilities.chat_completions {
            let request = match into_open_ai(
                request,
                &config.name,
                config.capabilities.parallel_tool_calls,
                config.capabilities.prompt_cache_key,
                config.max_output_tokens,
                ChatCompletionMaxTokensParameter::MaxCompletionTokens,
                config.reasoning_effort,
                config.capabilities.interleaved_reasoning,
            ) {
                Ok(request) => request,
                Err(error) => return async move { Err(error.into()) }.boxed(),
            };
            let completions = self.stream_chat_completion(&request_limiter, request, cx);
            let executor = cx.background_executor().clone();
            async move {
                let mapper = ChatCompletionEventMapper::new();
                Ok(stream_in_background(
                    mapper.map_stream(completions.await?).boxed(),
                    executor,
                ))
            }
            .boxed()
        } else {
            let request = match into_open_ai_response(
                request,
                &config.name,
                config.capabilities.parallel_tool_calls,
                config.capabilities.prompt_cache_key,
                config.max_output_tokens,
                config
                    .reasoning_effort
                    .filter(|effort| *effort != open_ai::ReasoningEffort::None),
                config.reasoning_effort == Some(open_ai::ReasoningEffort::None),
                &NARA_ROUTER_PROVIDER_ID,
            ) {
                Ok(request) => request,
                Err(error) => return async move { Err(error.into()) }.boxed(),
            };
            let completions = self.stream_response(&request_limiter, request, cx);
            let executor = cx.background_executor().clone();
            async move {
                let mapper = OpenAiResponseEventMapper::new(NARA_ROUTER_PROVIDER_ID);
                Ok(stream_in_background(
                    mapper.map_stream(completions.await?).boxed(),
                    executor,
                ))
            }
            .boxed()
        }
    }
}

fn language_model(model: &AvailableModel) -> LanguageModel {
    let supports_tools = model.capabilities.tools;
    LanguageModel {
        supports_tools,
        supports_images: model.capabilities.images,
        tool_choice_support: LanguageModelToolChoiceSupport {
            auto: supports_tools,
            any: supports_tools,
            none: true,
        },
        supports_streaming_tools: true,
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
            NARA_ROUTER_PROVIDER_ID,
            NARA_ROUTER_PROVIDER_NAME,
            format!("nara_router/{}", model.name),
            model.max_tokens,
        )
    }
}

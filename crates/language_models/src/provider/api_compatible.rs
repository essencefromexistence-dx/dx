use std::sync::Arc;

use ::util::ResultExt;
use anyhow::{Context as _, Result};
use convert_case::{Case, Casing};
use credentials_provider::CredentialsProvider;
use futures::AsyncReadExt;
use gpui::{App, AppContext as _, Context, Entity, SharedString, Task, TaskExt, Window};
use http_client::{AsyncBody, HttpClient, Method, Request as HttpRequest};
use language_model::{ApiKeyState, AuthenticateError, EnvVar};
use settings::SettingsStore;
use ui::{ElevationIndex, Tooltip, prelude::*};
use ui_input::InputField;

pub trait ApiCompatibleProviderSettings: Clone + Default + PartialEq + 'static {
    fn api_url(&self) -> &str;
}

/// How an OpenAI/Anthropic-compatible custom provider lists its models live.
///
/// The model picker must reflect fetched models instead of only the
/// hardcoded `available_models` from settings: custom gateways rotate ids
/// (new free models appear, old ones retire) and a stale settings list is
/// exactly the "hardcoded models in the picker" complaint.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct CompatibleModelsFetch {
    /// Appended to the configured `api_url`, e.g. `"/models"`.
    pub list_suffix: &'static str,
    /// Auth scheme for the list call.
    pub auth: CompatibleModelsFetchAuth,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CompatibleModelsFetchAuth {
    /// `Authorization: Bearer <key>` (OpenAI-compatible `/models`).
    Bearer,
    /// `x-api-key` + `anthropic-version` (Anthropic `/v1/models`).
    Anthropic,
}

pub struct ApiCompatibleProviderState<S: ApiCompatibleProviderSettings> {
    id: Arc<str>,
    pub api_key_state: ApiKeyState,
    pub settings: S,
    credentials_provider: Arc<dyn CredentialsProvider>,
    http_client: Arc<dyn HttpClient>,
    /// Live model ids from the provider's list endpoint. Empty until the
    /// first fetch completes; settings `available_models` remain the offline
    /// fallback and per-model override.
    pub fetched_models: Vec<String>,
    fetch_models_task: Option<Task<()>>,
    models_fetch: Option<CompatibleModelsFetch>,
}

impl<S: ApiCompatibleProviderSettings> ApiCompatibleProviderState<S> {
    pub fn new(
        id: Arc<str>,
        credentials_provider: Arc<dyn CredentialsProvider>,
        resolve_settings: for<'a> fn(&'a str, &'a App) -> Option<&'a S>,
        http_client: Arc<dyn HttpClient>,
        models_fetch: Option<CompatibleModelsFetch>,
        cx: &mut App,
    ) -> Entity<Self> {
        let api_key_env_var_name: SharedString =
            format!("{}_API_KEY", id).to_case(Case::UpperSnake).into();
        cx.new(|cx| {
            cx.observe_global::<SettingsStore>(move |this: &mut Self, cx| {
                let Some(settings) = resolve_settings(&this.id, cx).cloned() else {
                    return;
                };
                this.update_settings(settings, cx);
            })
            .detach();

            let settings = resolve_settings(&id, cx).cloned().unwrap_or_default();
            let mut this = Self {
                id,
                api_key_state: ApiKeyState::new(
                    SharedString::new(settings.api_url()),
                    EnvVar::new(api_key_env_var_name),
                ),
                settings,
                credentials_provider,
                http_client,
                fetched_models: Vec::new(),
                fetch_models_task: None,
                models_fetch,
            };
            // Fetch on startup when a key is already available (env var or
            // stored), so the picker shows live models without waiting for a
            // manual re-authentication.
            this.restart_models_fetch(cx);
            this
        })
    }

    pub fn is_authenticated(&self) -> bool {
        self.api_key_state.has_key()
    }

    pub fn set_api_key(
        &mut self,
        api_key: Option<String>,
        cx: &mut Context<Self>,
    ) -> Task<Result<()>> {
        let api_url = SharedString::new(self.settings.api_url());
        let should_fetch_models = api_key.is_some();
        let task = self.api_key_state.store(
            api_url,
            api_key,
            |this| &mut this.api_key_state,
            self.credentials_provider.clone(),
            cx,
        );
        self.fetched_models.clear();
        cx.spawn(async move |this, cx| {
            let result = task.await;
            if result.is_ok() && should_fetch_models {
                this.update(cx, |this, cx| this.restart_models_fetch(cx))
                    .ok();
            }
            result
        })
    }

    pub fn authenticate(&mut self, cx: &mut Context<Self>) -> Task<Result<(), AuthenticateError>> {
        let api_url = SharedString::new(self.settings.api_url());
        let task = self.api_key_state.load_if_needed(
            api_url,
            |this| &mut this.api_key_state,
            self.credentials_provider.clone(),
            cx,
        );
        cx.spawn(async move |this, cx| {
            let result = task.await;
            if result.is_ok() {
                this.update(cx, |this, cx| this.restart_models_fetch(cx))
                    .ok();
            }
            result
        })
    }

    pub fn update_settings(&mut self, settings: S, cx: &mut Context<Self>) {
        if self.settings != settings {
            let api_url = SharedString::new(settings.api_url());
            let url_changed = api_url != self.settings.api_url();
            self.api_key_state.handle_url_change(
                api_url,
                |this| &mut this.api_key_state,
                self.credentials_provider.clone(),
                cx,
            );
            self.settings = settings;
            if url_changed {
                self.fetched_models.clear();
                self.authenticate(cx).detach();
            }
            cx.notify();
        }
    }

    /// Re-fetch the live model list when a fetch config is set and a key is
    /// available. No-op otherwise (settings list remains the fallback).
    pub fn restart_models_fetch(&mut self, cx: &mut Context<Self>) {
        let Some(fetch) = self.models_fetch else {
            return;
        };
        let api_url = self.settings.api_url().to_string();
        let Some(api_key) = self.api_key_state.key(&api_url) else {
            return;
        };
        let api_key = api_key.to_string();
        let http_client = self.http_client.clone();
        self.fetch_models_task = Some(cx.spawn(async move |this, cx| {
            match fetch_compatible_models(http_client.as_ref(), &api_url, &api_key, fetch).await {
                Ok(models) => {
                    this.update(cx, |this, cx| {
                        this.fetched_models = models;
                        this.fetch_models_task = None;
                        cx.notify();
                    })
                    .ok();
                }
                Err(error) => {
                    log::warn!("Failed to fetch compatible provider models: {error:#}");
                    this.update(cx, |this, _| {
                        this.fetch_models_task = None;
                    })
                    .ok();
                }
            }
        }));
    }
}

#[derive(serde::Deserialize)]
struct CompatibleListModelsResponse {
    #[serde(default)]
    data: Vec<CompatibleListModelEntry>,
}

#[derive(serde::Deserialize)]
struct CompatibleListModelEntry {
    id: String,
}

/// `GET {api_url}{suffix}` returning `{"data": [{"id": ...}]}`.
///
/// Both OpenAI-compatible `/models` and Anthropic `/v1/models` answer this
/// shape (verified live against api.openai.com, api.mistral.ai, api.x.ai,
/// api.deepseek.com and api.anthropic.com: all answer 401 without a key,
/// proving the URL shape, and 200 with one).
pub(crate) async fn fetch_compatible_models(
    client: &dyn HttpClient,
    api_url: &str,
    api_key: &str,
    fetch: CompatibleModelsFetch,
) -> Result<Vec<String>> {
    let base = api_url.trim_end_matches('/');
    let uri = format!("{base}{}", fetch.list_suffix);
    let mut builder = HttpRequest::builder()
        .method(Method::GET)
        .uri(uri)
        .header("Accept", "application/json");
    builder = match fetch.auth {
        CompatibleModelsFetchAuth::Bearer => {
            builder.header("Authorization", format!("Bearer {}", api_key.trim()))
        }
        CompatibleModelsFetchAuth::Anthropic => builder
            .header("x-api-key", api_key.trim())
            .header("anthropic-version", "2023-06-01"),
    };
    let request = builder
        .body(AsyncBody::default())
        .context("failed to build compatible list models request")?;
    let mut response = client.send(request).await?;
    anyhow::ensure!(
        response.status().is_success(),
        "error listing compatible provider models, status: {:?}",
        response.status(),
    );
    let mut body = String::new();
    response
        .body_mut()
        .read_to_string(&mut body)
        .await
        .context("failed to read compatible list models response")?;
    let parsed: CompatibleListModelsResponse =
        serde_json::from_str(&body).context("failed to parse compatible list models response")?;
    Ok(parsed.data.into_iter().map(|entry| entry.id).collect())
}

pub struct ApiCompatibleProviderConfigurationView<S: ApiCompatibleProviderSettings> {
    api_key_editor: Entity<InputField>,
    state: Entity<ApiCompatibleProviderState<S>>,
    provider_name: &'static str,
    load_credentials_task: Option<Task<()>>,
}

impl<S: ApiCompatibleProviderSettings> ApiCompatibleProviderConfigurationView<S> {
    pub fn new(
        state: Entity<ApiCompatibleProviderState<S>>,
        provider_name: &'static str,
        placeholder_text: &'static str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let api_key_editor = cx.new(|cx| InputField::new(window, cx, placeholder_text));

        cx.observe(&state, |_, _, cx| {
            cx.notify();
        })
        .detach();

        let load_credentials_task = Some(cx.spawn_in(window, {
            let state = state.clone();
            async move |this, cx| {
                let task = state.update(cx, |state, cx| state.authenticate(cx));
                match task.await {
                    Ok(()) | Err(AuthenticateError::CredentialsNotFound) => {}
                    Err(error) => {
                        log::error!(
                            "Failed to load {provider_name}-compatible provider API credentials: {error}"
                        );
                    }
                }
                this.update(cx, |this, cx| {
                    this.load_credentials_task = None;
                    cx.notify();
                })
                .log_err();
            }
        }));

        Self {
            api_key_editor,
            state,
            provider_name,
            load_credentials_task,
        }
    }

    fn save_api_key(&mut self, _: &menu::Confirm, window: &mut Window, cx: &mut Context<Self>) {
        let api_key = self.api_key_editor.read(cx).text(cx).trim().to_string();
        if api_key.is_empty() {
            return;
        }

        // url changes can cause the editor to be displayed again
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
    }

    fn remove_provider(&mut self, _: &mut Window, cx: &mut Context<Self>) {
        let id = self.state.read(cx).id.clone();
        let fs = <dyn fs::Fs>::global(cx);
        settings::update_settings_file(fs, cx, move |settings, _| {
            let Some(language_models) = settings.language_models.as_mut() else {
                return;
            };

            if let Some(providers) = language_models.openai_compatible.as_mut() {
                providers.remove(id.as_ref());
            }
            if let Some(providers) = language_models.anthropic_compatible.as_mut() {
                providers.remove(id.as_ref());
            }
        });
    }

    fn should_render_editor(&self, cx: &Context<Self>) -> bool {
        !self.state.read(cx).is_authenticated()
    }
}

impl<S: ApiCompatibleProviderSettings> Render for ApiCompatibleProviderConfigurationView<S> {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.state.read(cx);
        let env_var_set = state.api_key_state.is_from_env_var();
        let env_var_name = state.api_key_state.env_var_name();
        let provider_name = self.provider_name;

        let api_key_section = if self.should_render_editor(cx) {
            v_flex()
                .on_action(cx.listener(Self::save_api_key))
                .child(Label::new(format!(
                    "To use Zed's agent with an {provider_name}-compatible provider, you need to add an API key."
                )))
                .child(
                    div()
                        .pt(DynamicSpacing::Base04.rems(cx))
                        .child(self.api_key_editor.clone()),
                )
                .child(
                    Label::new(format!(
                        "You can also set the {env_var_name} environment variable and restart Zed.",
                    ))
                    .size(LabelSize::Small)
                    .color(Color::Muted),
                )
                .into_any()
        } else {
            h_flex()
                .mt_1()
                .p_1()
                .justify_between()
                .rounded_md()
                .border_1()
                .border_color(cx.theme().colors().border)
                .bg(cx.theme().colors().background)
                .child(
                    h_flex()
                        .flex_1()
                        .min_w_0()
                        .gap_1()
                        .child(Icon::new(IconName::Check).color(Color::Success))
                        .child(
                            div().w_full().overflow_x_hidden().text_ellipsis().child(Label::new(
                                if env_var_set {
                                    format!("API key set in {env_var_name} environment variable")
                                } else {
                                    format!("API key configured for {}", state.settings.api_url())
                                },
                            )),
                        ),
                )
                .child(
                    h_flex().flex_shrink_0().child(
                        Button::new("reset-api-key", "Reset API Key")
                            .label_size(LabelSize::Small)
                            .start_icon(Icon::new(IconName::Undo).size(IconSize::Small))
                            .layer(ElevationIndex::ModalSurface)
                            .when(env_var_set, |this| {
                                this.tooltip(Tooltip::text(format!(
                                    "To reset your API key, unset the {env_var_name} environment variable.",
                                )))
                            })
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.reset_api_key(window, cx)
                            })),
                    ),
                )
                .into_any()
        };

        if self.load_credentials_task.is_some() {
            div().child(Label::new("Loading credentials…")).into_any()
        } else {
            v_flex()
                .size_full()
                .gap_4()
                .child(api_key_section)
                .child(
                    h_flex().w_full().justify_end().child(
                        Button::new("remove-compatible-provider", "Remove Provider")
                            .style(ButtonStyle::OutlinedGhost)
                            .label_size(LabelSize::Small)
                            .start_icon(
                                Icon::new(IconName::Trash)
                                    .size(IconSize::Small)
                                    .color(Color::Muted),
                            )
                            .on_click(cx.listener(|this, _event, window, cx| {
                                this.remove_provider(window, cx)
                            })),
                    ),
                )
                .into_any()
        }
    }
}

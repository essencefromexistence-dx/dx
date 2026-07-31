use std::sync::Arc;

use anyhow::{Result, bail};
use collections::IndexMap;
use convert_case::{Case, Casing as _};
use fs::Fs;
use gpui::{App, SharedString};
use settings::{
    AgentProfileContent, ContextServerPresetContent, LanguageModelSelection, Settings as _,
    SettingsContent, SettingsStore, update_settings_file,
};
use util::ResultExt as _;

use crate::{AgentProfileId, AgentSettings};

pub mod builtin_profiles {
    use super::AgentProfileId;

    pub const WRITE: &str = "write";
    pub const ASK: &str = "ask";
    pub const MINIMAL: &str = "minimal";
    pub const MEDIA: &str = "media";
    pub const SEARCH: &str = "search";
    pub const STUDY: &str = "study";
    pub const LEGACY_MINIMAL: &str = MINIMAL;
    pub const DX_PROFILE_ORDER: [&str; 4] = [WRITE, SEARCH, MEDIA, ASK];

    pub fn is_builtin(profile_id: &AgentProfileId) -> bool {
        matches!(
            profile_id.as_str(),
            WRITE | ASK | MINIMAL | MEDIA | SEARCH | STUDY
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DxAiProfileKind {
    Ask,
    Agents,
    Search,
    Study,
    Media,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DxAiProfileBackendState {
    Wired,
    EvidenceBacked,
    ReceiptBacked,
    ProviderPending,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DxAiProfileMetadata {
    pub id: &'static str,
    pub kind: DxAiProfileKind,
    pub display_name: &'static str,
    pub summary: &'static str,
    pub backend_state: DxAiProfileBackendState,
    pub runtime_proof_backend_lane_id: Option<&'static str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentProfile {
    id: AgentProfileId,
}

pub type AvailableProfiles = IndexMap<AgentProfileId, SharedString>;

impl AgentProfile {
    pub fn new(id: AgentProfileId) -> Self {
        Self { id }
    }

    pub fn id(&self) -> &AgentProfileId {
        &self.id
    }

    pub fn display_name(profile_id: &AgentProfileId, name: &SharedString) -> SharedString {
        Self::dx_builtin_metadata_for_id(profile_id.as_str())
            .map(|metadata| metadata.display_name.into())
            .unwrap_or_else(|| name.clone())
    }

    pub fn dx_builtin_metadata(profile_id: &AgentProfileId) -> Option<DxAiProfileMetadata> {
        Self::dx_builtin_metadata_for_id(profile_id.as_str())
    }

    pub fn dx_builtin_metadata_for_id(profile_id: &str) -> Option<DxAiProfileMetadata> {
        match profile_id {
            builtin_profiles::ASK | builtin_profiles::LEGACY_MINIMAL => Some(DxAiProfileMetadata {
                id: builtin_profiles::ASK,
                kind: DxAiProfileKind::Ask,
                display_name: "Ask",
                summary: "Answers questions with lightweight local and web sources.",
                backend_state: DxAiProfileBackendState::Wired,
                runtime_proof_backend_lane_id: None,
            }),
            builtin_profiles::WRITE => Some(DxAiProfileMetadata {
                id: builtin_profiles::WRITE,
                kind: DxAiProfileKind::Agents,
                display_name: "Agents",
                summary: "Runs builder and worker flows for code, tools, goals, plans, and multitask work.",
                backend_state: DxAiProfileBackendState::Wired,
                runtime_proof_backend_lane_id: None,
            }),
            builtin_profiles::SEARCH => Some(DxAiProfileMetadata {
                id: builtin_profiles::SEARCH,
                kind: DxAiProfileKind::Search,
                display_name: "Search",
                summary: "Uses DX MetaSearch, source-pack evidence, and Web Preview inspection where available.",
                backend_state: DxAiProfileBackendState::EvidenceBacked,
                runtime_proof_backend_lane_id: Some("dx-metasearch-live-proof"),
            }),
            builtin_profiles::STUDY => Some(DxAiProfileMetadata {
                id: builtin_profiles::STUDY,
                kind: DxAiProfileKind::Study,
                display_name: "Study",
                summary: "Organizes attached sources, receipts, and study rails without inventing results.",
                backend_state: DxAiProfileBackendState::ReceiptBacked,
                runtime_proof_backend_lane_id: Some("study-source-workspace-execution"),
            }),
            builtin_profiles::MEDIA => Some(DxAiProfileMetadata {
                id: builtin_profiles::MEDIA,
                kind: DxAiProfileKind::Media,
                display_name: "Media",
                summary: "Plans and gates image, video, audio, music, 3D, and document providers while execution setup is pending.",
                backend_state: DxAiProfileBackendState::ProviderPending,
                runtime_proof_backend_lane_id: Some("media-provider-readiness-proof"),
            }),
            _ => None,
        }
    }

    pub fn builtin_sort_index(profile_id: &AgentProfileId) -> Option<usize> {
        builtin_profiles::DX_PROFILE_ORDER
            .iter()
            .position(|id| *id == profile_id.as_str())
    }

    /// Saves a new profile to the settings.
    pub fn create(
        name: String,
        base_profile_id: Option<AgentProfileId>,
        fs: Arc<dyn Fs>,
        cx: &App,
    ) -> AgentProfileId {
        let id = AgentProfileId(name.to_case(Case::Kebab).into());

        let base_profile =
            base_profile_id.and_then(|id| AgentSettings::get_global(cx).profiles.get(&id).cloned());

        // Copy toggles from the base profile so the new profile starts with familiar defaults.
        let tools = base_profile
            .as_ref()
            .map(|profile| profile.tools.clone())
            .unwrap_or_default();
        let enable_all_context_servers = base_profile
            .as_ref()
            .map(|profile| profile.enable_all_context_servers)
            .unwrap_or_default();
        let context_servers = base_profile
            .as_ref()
            .map(|profile| profile.context_servers.clone())
            .unwrap_or_default();
        // Preserve the base profile's model preference when cloning into a new profile.
        let default_model = base_profile
            .as_ref()
            .and_then(|profile| profile.default_model.clone());

        let profile_settings = AgentProfileSettings {
            name: name.into(),
            tools,
            enable_all_context_servers,
            context_servers,
            default_model,
        };

        update_settings_file(fs, cx, {
            let id = id.clone();
            move |settings, _cx| {
                profile_settings.save_to_settings(id, settings).log_err();
            }
        });

        id
    }

    /// Returns a map of AgentProfileIds to their names
    pub fn available_profiles(cx: &App) -> AvailableProfiles {
        let mut profiles = AvailableProfiles::default();
        for (id, profile) in AgentSettings::get_global(cx).profiles.iter() {
            profiles.insert(id.clone(), profile.name.clone());
        }
        profiles
    }
}

/// A profile for the Zed Agent that controls its behavior.
#[derive(Debug, Clone)]
pub struct AgentProfileSettings {
    /// The name of the profile.
    pub name: SharedString,
    pub tools: IndexMap<Arc<str>, bool>,
    pub enable_all_context_servers: bool,
    pub context_servers: IndexMap<Arc<str>, ContextServerPreset>,
    /// Default language model to apply when this profile becomes active.
    pub default_model: Option<LanguageModelSelection>,
}

impl AgentProfileSettings {
    pub fn is_tool_enabled(&self, tool_name: &str) -> bool {
        self.tools.get(tool_name) == Some(&true)
    }

    /// Whether the built-in profile with the given id still matches the shipped
    /// default — i.e. the user has neither customized the built-in profile nor
    /// shadowed it with a custom profile of the same id. Custom profile ids are
    /// never considered unmodified defaults.
    pub fn is_unmodified_default(profile_id: &AgentProfileId, cx: &App) -> bool {
        if !builtin_profiles::is_builtin(profile_id) {
            return false;
        }
        let store = cx.global::<SettingsStore>();
        let profile_in = |content: &SettingsContent| {
            content
                .agent
                .as_ref()
                .and_then(|agent| agent.profiles.as_ref())
                .and_then(|profiles| profiles.get(profile_id.as_str()))
                .cloned()
        };
        match (
            profile_in(store.merged_settings()),
            profile_in(store.raw_default_settings()),
        ) {
            (Some(merged), Some(default)) => merged == default,
            _ => false,
        }
    }

    pub fn is_context_server_tool_enabled(&self, server_id: &str, tool_name: &str) -> bool {
        self.context_servers
            .get(server_id)
            .and_then(|preset| preset.tools.get(tool_name).copied())
            .unwrap_or(self.enable_all_context_servers)
    }

    pub fn save_to_settings(
        &self,
        profile_id: AgentProfileId,
        content: &mut SettingsContent,
    ) -> Result<()> {
        let profiles = content
            .agent
            .get_or_insert_default()
            .profiles
            .get_or_insert_default();
        if profiles.contains_key(&profile_id.0) {
            bail!("profile with ID '{profile_id}' already exists");
        }

        profiles.insert(
            profile_id.0,
            AgentProfileContent {
                name: self.name.clone().into(),
                tools: self.tools.clone(),
                enable_all_context_servers: Some(self.enable_all_context_servers),
                context_servers: self
                    .context_servers
                    .clone()
                    .into_iter()
                    .map(|(server_id, preset)| {
                        (
                            server_id,
                            ContextServerPresetContent {
                                tools: preset.tools,
                            },
                        )
                    })
                    .collect(),
                default_model: self.default_model.clone(),
            },
        );

        Ok(())
    }
}

impl From<AgentProfileContent> for AgentProfileSettings {
    fn from(content: AgentProfileContent) -> Self {
        let AgentProfileContent {
            name,
            tools,
            enable_all_context_servers,
            context_servers,
            default_model,
        } = content;

        Self {
            name: name.into(),
            tools,
            enable_all_context_servers: enable_all_context_servers.unwrap_or_default(),
            context_servers: context_servers
                .into_iter()
                .map(|(server_id, preset)| (server_id, preset.into()))
                .collect(),
            default_model,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ContextServerPreset {
    pub tools: IndexMap<Arc<str>, bool>,
}

impl From<settings::ContextServerPresetContent> for ContextServerPreset {
    fn from(content: settings::ContextServerPresetContent) -> Self {
        Self {
            tools: content.tools,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile(
        enable_all_context_servers: bool,
        context_servers: IndexMap<Arc<str>, ContextServerPreset>,
    ) -> AgentProfileSettings {
        AgentProfileSettings {
            name: "test".into(),
            tools: IndexMap::default(),
            enable_all_context_servers,
            context_servers,
            default_model: None,
        }
    }

    fn preset(tools: &[(&str, bool)]) -> ContextServerPreset {
        ContextServerPreset {
            tools: tools
                .iter()
                .map(|(name, enabled)| (Arc::from(*name), *enabled))
                .collect(),
        }
    }

    #[test]
    fn explicit_false_disables_tool_when_enable_all_is_true() {
        let mut servers = IndexMap::default();
        servers.insert(Arc::from("server"), preset(&[("disabled_tool", false)]));
        let profile = profile(true, servers);

        assert!(!profile.is_context_server_tool_enabled("server", "disabled_tool"));
        assert!(profile.is_context_server_tool_enabled("server", "other_tool"));
        assert!(profile.is_context_server_tool_enabled("other_server", "any_tool"));
    }

    #[test]
    fn explicit_true_enables_tool_when_enable_all_is_false() {
        let mut servers = IndexMap::default();
        servers.insert(Arc::from("server"), preset(&[("enabled_tool", true)]));
        let profile = profile(false, servers);

        assert!(profile.is_context_server_tool_enabled("server", "enabled_tool"));
        assert!(!profile.is_context_server_tool_enabled("server", "other_tool"));
        assert!(!profile.is_context_server_tool_enabled("other_server", "any_tool"));
    }

    #[gpui::test]
    fn unmodified_default_detection(cx: &mut gpui::App) {
        use gpui::UpdateGlobal as _;

        let store = SettingsStore::test(cx);
        cx.set_global(store);
        project::DisableAiSettings::register(cx);
        AgentSettings::register(cx);

        let write = AgentProfileId(builtin_profiles::WRITE.into());
        let minimal = AgentProfileId(builtin_profiles::MINIMAL.into());
        let custom = AgentProfileId("custom".into());

        // Fresh defaults: the shipped built-in profiles are unmodified.
        assert!(AgentProfileSettings::is_unmodified_default(&write, cx));
        assert!(AgentProfileSettings::is_unmodified_default(&minimal, cx));
        // Custom (non-built-in) ids are never considered unmodified defaults.
        assert!(!AgentProfileSettings::is_unmodified_default(&custom, cx));

        // The user customizes the `write` profile; `minimal` stays untouched.
        SettingsStore::update_global(cx, |store, cx| {
            store
                .set_user_settings(
                    r#"{ "agent": { "profiles": { "write": { "name": "Write", "tools": { "fetch": false } } } } }"#,
                    cx,
                )
                .unwrap();
        });

        assert!(!AgentProfileSettings::is_unmodified_default(&write, cx));
        assert!(AgentProfileSettings::is_unmodified_default(&minimal, cx));
    }
}

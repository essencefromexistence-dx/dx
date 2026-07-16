    fn render_hidden_feature_buttons(&self, cx: &mut Context<Self>) -> Vec<AnyElement> {
        let active_right_panel = self.active_right_panel_name(cx);
        vec![
            self.render_title_right_panel_button(
                "titlebar-icon-picker",
                dx_icon(DxUiIcon::Icons),
                "Icons",
                icon_picker::ToggleFocus.boxed_clone(),
                active_right_panel == Some("Icon Picker"),
            ),
            self.render_title_right_panel_button(
                "titlebar-font-panel",
                dx_icon(DxUiIcon::Fonts),
                "Fonts",
                font_panel::ToggleFocus.boxed_clone(),
                active_right_panel == Some("Font Panel"),
            ),
            // self.render_title_right_panel_button(
            //     "titlebar-dx-forge-panel",
            //     dx_icon(DxUiIcon::Forge),
            //     "Forge",
            //     zed_actions::dx_forge::TogglePanel.boxed_clone(),
            //     active_right_panel == Some("Forge"),
            // ), // commented out per request

            self.render_title_right_panel_button(
                "titlebar-media-panel",
                dx_icon(DxUiIcon::Media),
                "Media",
                media_panel::ToggleFocus.boxed_clone(),
                active_right_panel == Some("Media"),
            ),
            self.render_title_right_panel_button(
                "titlebar-shadcn-ui-panel",
                dx_icon(DxUiIcon::Ui),
                "UI",
                shadcn_ui_panel::ToggleFocus.boxed_clone(),
                active_right_panel == Some("UI"),
            ),
            // TODO(dx-style-panel): Re-enable the Style panel button when its Web Preview
            // workflow graduates from parked implementation to production UI.
            self.render_title_right_panel_button(
                "titlebar-dx-check-panel",
                dx_icon(DxUiIcon::Check),
                "Check",
                zed_actions::dx_check_panel::ToggleFocus.boxed_clone(),
                active_right_panel == Some("Check"),
            ),
            self.render_title_right_panel_button(
                "titlebar-dx-skill-panel",
                dx_icon(DxUiIcon::Automations),
                "Skill",
                zed_actions::dx_skill_panel::ToggleFocus.boxed_clone(),
                active_right_panel == Some("Skill"),
            ),
            self.render_hidden_feature_menu(cx),
        ]
    }

    fn active_right_panel_name(&self, cx: &App) -> Option<&'static str> {
        let workspace = self.workspace.upgrade()?;
        let workspace = workspace.read(cx);
        let dock = workspace.dock_at_position(DockPosition::Right).read(cx);
        dock.visible_panel().map(|panel| panel.persistent_name())
    }

    fn render_title_right_panel_button(
        &self,
        id: &'static str,
        icon: IconName,
        tooltip: &'static str,
        action: Box<dyn Action>,
        selected: bool,
    ) -> AnyElement {
        let workspace = self.workspace.clone();
        IconButton::new(id, icon)
            .icon_size(IconSize::Small)
            .style(ButtonStyle::Subtle)
            .selected_style(ButtonStyle::Tinted(TintColor::Accent))
            .toggle_state(selected)
            .tooltip(Tooltip::text(tooltip))
            .on_click(move |_, window, cx| {
                if selected && let Some(workspace) = workspace.upgrade() {
                    let right_dock = workspace
                        .read(cx)
                        .dock_at_position(DockPosition::Right)
                        .clone();
                    let panel_id = right_dock
                        .read(cx)
                        .active_panel()
                        .map(|panel| panel.panel_id());
                    if let Some(panel_id) = panel_id {
                        workspace.update(cx, |workspace, cx| {
                            workspace.close_side_panel_by_id(panel_id, window, cx);
                        });
                    }
                } else if let Some(workspace) = workspace.upgrade() {
                    let focus_handle = workspace.read(cx).focus_handle(cx);
                    focus_handle.dispatch_action(action.as_ref(), window, cx);
                } else {
                    window.dispatch_action(action.boxed_clone(), cx);
                }
            })
            .into_any_element()
    }

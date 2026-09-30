use gpui::{Anchor, Entity};
use ui::{
    ContextMenu, ContextMenuEntry, DocumentationSide, IconButton, IconName, IconPosition, IconSize,
    Label, PopoverMenu, Tooltip, prelude::*,
};

use crate::{
    TableView,
    settings::{FilterSortOrder, TableViewSettings, VerticalAlignment},
};

///// Settings related /////

/// Adds a toggleable entry that applies `set` to the pane's settings when clicked. `description`,
/// when given, shows as a documentation aside explaining what the entry does.
fn toggle_entry(
    menu: ContextMenu,
    label: &'static str,
    description: Option<&'static str>,
    selected: bool,
    view_entity: &Entity<TableView>,
    set: impl Fn(&mut TableViewSettings) + 'static,
) -> ContextMenu {
    let view_entity = view_entity.clone();
    let entry = ContextMenuEntry::new(label)
        .toggleable(IconPosition::Start, selected)
        .handler(move |_, cx| {
            view_entity.update(cx, |this, cx| {
                set(&mut this.settings);
                cx.notify();
            });
        });
    let entry = if let Some(description) = description {
        entry.documentation_aside(DocumentationSide::Right, move |_| {
            Label::new(description).into_any_element()
        })
    } else {
        entry
    };
    menu.item(entry)
}

pub(crate) fn settings_popover_menu(view_entity: Entity<TableView>) -> PopoverMenu<ContextMenu> {
    PopoverMenu::new("table-settings-menu")
        .trigger_with_tooltip(
            IconButton::new("table-settings-trigger", IconName::Filter)
                .icon_size(IconSize::Small)
                .size(ButtonSize::Compact),
            Tooltip::text("Table Settings"),
        )
        .anchor(Anchor::TopRight)
        .menu(move |window, cx| {
            let view_entity = view_entity.clone();
            Some(ContextMenu::build_persistent(
                window,
                cx,
                move |menu, _window, cx| {
                    let settings = view_entity.read(cx).settings.clone();

                    let menu = toggle_entry(
                        menu.header("Text Alignment"),
                        "Top",
                        Some("Choose vertical text alignment within cells"),
                        matches!(settings.vertical_alignment, VerticalAlignment::Top),
                        &view_entity,
                        |settings| settings.vertical_alignment = VerticalAlignment::Top,
                    );
                    let menu = toggle_entry(
                        menu,
                        "Center",
                        None,
                        matches!(settings.vertical_alignment, VerticalAlignment::Center),
                        &view_entity,
                        |settings| settings.vertical_alignment = VerticalAlignment::Center,
                    );

                    let menu = menu.separator().header("Filter Sort");
                    let menu = toggle_entry(
                        menu,
                        "A-Z, then Count",
                        Some("Choose how filter values are sorted in the filter menu"),
                        settings.filter_sort_order == FilterSortOrder::AlphaThenCount,
                        &view_entity,
                        |settings| settings.filter_sort_order = FilterSortOrder::AlphaThenCount,
                    );
                    let menu = toggle_entry(
                        menu,
                        "Count, then A-Z",
                        None,
                        settings.filter_sort_order == FilterSortOrder::CountThenAlpha,
                        &view_entity,
                        |settings| settings.filter_sort_order = FilterSortOrder::CountThenAlpha,
                    );

                    let menu = toggle_entry(
                        menu.separator(),
                        "Display multiline rows",
                        Some(
                            "When enabled, row height grows to show all content. \
                             When disabled, only the first line is visible — hover a cell to see the rest.",
                        ),
                        settings.multiline_cells_enabled,
                        &view_entity,
                        |settings| settings.multiline_cells_enabled = !settings.multiline_cells_enabled,
                    );

                    // dx fork: upstream appends the dev-only entries here. Our fork keeps them on
                    // its own `dev_only_popover_menu` trigger instead (see below) so the local UI
                    // survives — our settings surfaces are branded with `IconName::DxCog`
                    // (`crates/ui/src/dx_icons.rs`: `DxUiIcon::Settings => IconName::DxCog`).
                    // Upstream's call is preserved verbatim, only commented out:
                    //
                    //     #[cfg(feature = "dev-tools")]
                    //     let menu = append_dev_only_entries(menu, &view_entity, &settings);

                    menu
                },
            ))
        })
}

#[cfg(feature = "dev-tools")]
fn append_dev_only_entries(
    menu: ContextMenu,
    view_entity: &Entity<TableView>,
    settings: &TableViewSettings,
) -> ContextMenu {
    use crate::settings::RowRenderMechanism;

    let menu = menu.separator().header("Dev-only: Rendering Mode");
    let menu = toggle_entry(
        menu,
        "Variable Height",
        Some(
            "Dev-only section used for debugging purposes.\n\
             Will be removed on public release of the tabular data preview feature",
        ),
        settings.rendering_with == RowRenderMechanism::VariableList,
        view_entity,
        |settings| settings.rendering_with = RowRenderMechanism::VariableList,
    );
    let menu = toggle_entry(
        menu,
        "Uniform Height",
        None,
        settings.rendering_with == RowRenderMechanism::UniformList,
        view_entity,
        |settings| settings.rendering_with = RowRenderMechanism::UniformList,
    );

    let menu = toggle_entry(
        menu.separator(),
        "Show perf metrics",
        None,
        settings.show_perf_metrics_overlay,
        view_entity,
        |settings| settings.show_perf_metrics_overlay = !settings.show_perf_metrics_overlay,
    );
    toggle_entry(
        menu,
        "Show cell positions",
        None,
        settings.show_debug_info,
        view_entity,
        |settings| settings.show_debug_info = !settings.show_debug_info,
    )
}

/// dx fork: our dev-only popover, preserved from `crates/csv_preview/src/renderer/settings.rs`.
///
/// Upstream folded these entries into [`settings_popover_menu`] and deleted our separate
/// trigger. We keep the local surface: its trigger is branded with `IconName::DxCog`
/// (`crates/ui/src/dx_icons.rs`: `DxUiIcon::Settings => IconName::DxCog`, asserted by
/// `script/dx-icon-system-source.test.ts` and `script/dx-forge-panel-source.test.ts`).
///
/// The entries themselves come from upstream's [`append_dev_only_entries`], so there is exactly
/// one definition of them and nothing upstream is duplicated or lost.
#[cfg(feature = "dev-tools")]
pub(crate) fn dev_only_popover_menu(
    view_entity: Entity<TabularDataPreviewPane>,
) -> PopoverMenu<ContextMenu> {
    PopoverMenu::new("debug-options-menu")
        .trigger_with_tooltip(
            IconButton::new("debug-options-trigger", IconName::DxCog).icon_size(IconSize::Small),
            Tooltip::text(
                "Dev-only section used for debugging purposes.\nWill be removed on public release of the tabular data preview feature",
            ),
        )
        .anchor(Anchor::TopRight)
        .menu(move |window, cx| {
            let view_entity = view_entity.clone();
            Some(ContextMenu::build_persistent(
                window,
                cx,
                move |menu, _window, cx| {
                    let settings = view_entity.read(cx).settings.clone();
                    append_dev_only_entries(menu, &view_entity, &settings)
                },
            ))
        })
}

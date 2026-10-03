#!/usr/bin/env python3
"""Port ours' two per-model behaviours onto upstream's data path in opencode.rs.

1. `icon_for_model` — ours' only local addition to the (now removed) `LanguageModel`
   trait was a per-model icon override: OpenCode's GLM 5 / 5.1 show the Zhipu mark.
   Upstream models the icon per *provider* only, so the port uses the defaulted
   `LanguageModelProvider::icon_for_model` hook (which we added to the trait).
2. Free-tier naming — ours' Free tier is keyless and its models are already named for
   the user, so it does not get upstream's `"{subscription}: {model}"` prefix.
"""
P = r"G:/Dx/code/crates/language_models/src/provider/opencode.rs"

raw = open(P, "r", encoding="utf-8", newline="").read()
had_crlf = "\r\n" in raw
text = raw.replace("\r\n", "\n")

ICON_OLD = """    fn icon(&self) -> IconOrSvg {
        IconOrSvg::Icon(IconName::Sparkle)
    }
"""
ICON_NEW = """    fn icon(&self) -> IconOrSvg {
        IconOrSvg::Icon(IconName::Sparkle)
    }

    /// Ours: a per-model icon override.
    ///
    /// GLM 5 / 5.1 carry the Zhipu mark instead of the provider's own. Upstream's
    /// [`LanguageModel::id`] is subscription-prefixed (`zen/glm-5`), so the check
    /// looks at the model segment.
    fn icon_for_model(&self, model: &LanguageModel) -> IconOrSvg {
        let id = model.id.0.as_ref();
        let model_segment = id.rsplit('/').next().unwrap_or(id);
        if matches!(model_segment, "glm-5" | "glm-5.1") {
            IconOrSvg::Icon(IconName::AiZhipu)
        } else {
            self.icon()
        }
    }
"""

NAME_OLD = """            LanguageModelName::from(format!(
                "{}: {}",
                subscription.display_name(),
                model.display_name()
            )),
"""
NAME_NEW = """            // Ours: the keyless Free tier's models are already named for the
            // user, so they do not get the subscription prefix.
            if subscription == OpenCodeSubscription::Free {
                LanguageModelName::from(model.display_name().to_string())
            } else {
                LanguageModelName::from(format!(
                    "{}: {}",
                    subscription.display_name(),
                    model.display_name()
                ))
            },
"""

for label, old, new in [("icon_for_model", ICON_OLD, ICON_NEW), ("free-tier name", NAME_OLD, NAME_NEW)]:
    n = text.count(old)
    assert n == 1, f"{label}: expected 1 match, found {n}"
    text = text.replace(old, new, 1)
    print(f"{label}: applied")

if had_crlf:
    text = text.replace("\n", "\r\n")
open(P, "w", encoding="utf-8", newline="").write(text)
print(f"wrote {P} (crlf={had_crlf})")

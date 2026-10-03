#!/usr/bin/env python3
"""Resolve the 3 remaining conflict blocks in language_models/src/provider/opencode.rs.

Semantic merge, not a side-pick:
  * block 1 -> UNION. Upstream moved the request path onto the provider
    (base_api_url / current_api_key / custom_headers / stream_*). We keep all of it and
    re-attach OURS' Free-tier behaviour: the keyless `public` bearer fallback in
    `current_api_key`, plus ours' `create_tier_model` bridge rewritten to produce
    upstream's `LanguageModel` data instead of ours' (now removed) model type.
  * block 2 -> UNION. Ours' Free-tier preference chain + union catalog (discovered +
    static OpenCodeTier + user-configured) survive, returning upstream's `LanguageModel`.
    Keys use upstream's `model_id(...)` so upstream's `config()` lookup still resolves.
  * block 3 -> THEIRS. Ours' `impl OpenCodeLanguageModel` / `impl LanguageModel for` are
    vestigial: upstream's `LanguageModel` is plain data and its accessor set is identical,
    and the merge had already dropped ours' struct definition, leaving them uncompilable.
    Upstream's `model_id` / `language_model` producers take their place.
"""
P = r"G:/Dx/code/crates/language_models/src/provider/opencode.rs"

raw = open(P, "r", encoding="utf-8", newline="").read()
had_crlf = "\r\n" in raw
text = raw.replace("\r\n", "\n")
lines = text.split("\n")


def clean(s):
    return s[:-1] if s.endswith("\r") else s


blocks = []
i = 0
while i < len(lines):
    if clean(lines[i]).startswith("<<<<<<<"):
        s = i; m = None; e = None; j = i + 1
        while j < len(lines):
            c = clean(lines[j])
            if c == "=======" and m is None:
                m = j
            elif c.startswith(">>>>>>>"):
                e = j; break
            j += 1
        blocks.append((s, m, e)); i = e + 1
    else:
        i += 1
print("blocks:", [(s + 1, m + 1, e + 1) for s, m, e in blocks])
assert len(blocks) == 3

(s1, m1, e1), (s2, m2, e2), (s3, m3, e3) = blocks
blk1_theirs = lines[m1 + 1:e1]
blk2_ours = lines[s2 + 1:m2]
blk3_theirs = lines[m3 + 1:e3]

# ---------------- block 1: upstream provider path + ours' Free-tier key ----------
blk1 = "\n".join(blk1_theirs)

OLD_CK = """    fn current_api_key(&self, cx: &AsyncApp) -> Option<Arc<str>> {
        self.state.read_with(cx, |state, cx| {
            let api_url = OpenCodeLanguageModelProvider::api_url(cx);
            state.api_key_state.key(&api_url)
        })
    }"""
NEW_CK = """    /// The bearer token for `config`'s subscription.
    ///
    /// Ours: the keyless public Free tier authenticates with the literal
    /// `public` bearer whenever the model is one of the built-in free models,
    /// so an absent stored key is not an error there.
    fn current_api_key(
        &self,
        config: &DiscoveredModel,
        subscription: OpenCodeSubscription,
        cx: &AsyncApp,
    ) -> Option<Arc<str>> {
        let api_key = self.state.read_with(cx, |state, cx| {
            let api_url = OpenCodeLanguageModelProvider::api_url(cx);
            state.api_key_state.key(&api_url)
        });

        api_key.or_else(|| {
            if subscription == OpenCodeSubscription::Free
                && is_builtin_public_free_model(&config.model)
            {
                Some(Arc::from(PUBLIC_API_KEY))
            } else {
                None
            }
        })
    }"""
assert blk1.count(OLD_CK) == 1, f"current_api_key def: {blk1.count(OLD_CK)}"
blk1 = blk1.replace(OLD_CK, NEW_CK, 1)

n_calls = blk1.count("self.current_api_key(cx)")
assert n_calls == 4, f"current_api_key call sites: {n_calls}"
blk1 = blk1.replace("self.current_api_key(cx)", "self.current_api_key(config, subscription, cx)")

TIER_FN = """
    /// Ours' static [`OpenCodeTier`] materialised onto upstream's data path.
    ///
    /// This is the bridge that lets ours' Free-tier preference chain
    /// (`default_model` / `default_fast_model`) and static catalog coexist with
    /// upstream's discovered-model path: the tier stays ours' carrier, the value
    /// the provider and the request path speak is upstream's [`LanguageModel`].
    fn create_tier_model(
        &self,
        tier: OpenCodeTier,
        subscription: OpenCodeSubscription,
    ) -> LanguageModel {
        let model = tier.to_model(subscription);
        let supports_images = tier.supports_images();
        let supports_thinking = model_supports_thinking(&model);
        language_model(
            &DiscoveredModel {
                model,
                supports_images,
                supports_thinking,
            },
            subscription,
        )
    }
"""
# `pub fn settings` sits in the shared region just after this block (the block's last
# line is stream_google's body, whose closing brace is shared), so the helper is
# inserted file-wide after the block swap, not into blk1.
SETTINGS_ANCHOR = "    pub fn settings(cx: &App) -> &OpenCodeSettings {"
blk1 = blk1.split("\n")

# ---------------- block 2: ours' logic returning upstream's data -----------------
blk2 = "\n".join(blk2_ours)
SUBS2 = [
    ("    fn default_model(&self, cx: &App) -> Option<Arc<dyn LanguageModel>> {",
     "    fn default_model(&self, cx: &App) -> Option<LanguageModel> {"),
    ("    fn default_fast_model(&self, cx: &App) -> Option<Arc<dyn LanguageModel>> {",
     "    fn default_fast_model(&self, cx: &App) -> Option<LanguageModel> {"),
    ("    fn provided_models(&self, cx: &App) -> Vec<Arc<dyn LanguageModel>> {",
     "    fn provided_models(&self, cx: &App) -> Vec<LanguageModel> {"),
    ("                        let key = opencode_model_registry_key(&model.model);",
     "                        let key = model_id(&model.model, subscription);"),
    ("                    let key = opencode_model_registry_key(&model);",
     "                    let key = model_id(&model, subscription);"),
    ("                let key = opencode_model_registry_key(&custom_model);",
     "                let key = model_id(&custom_model, subscription);"),
    ("""            .map(|(model, subscription, _)| {
                self.create_language_model_with_capabilities(
                    model.model,
                    subscription,
                    model.supports_images,
                    model.supports_thinking,
                )
            })""",
     "            .map(|(model, subscription, _)| language_model(&model, subscription))"),
]
for old, new in SUBS2:
    assert blk2.count(old) == 1, f"block2 sub not unique: {old[:60]!r} -> {blk2.count(old)}"
    blk2 = blk2.replace(old, new, 1)
blk2 = blk2.split("\n")

blk3 = list(blk3_theirs)

# ---------------- apply bottom-up ----------------------------------------------
for (s, m, e), body in sorted(
    zip(blocks, [blk1, blk2, blk3]), key=lambda x: -x[0][0]
):
    print(f"block {s+1}-{e+1}: -> {len(body)} lines")
    lines[s:e + 1] = body

out = "\n".join(lines)
assert out.count(SETTINGS_ANCHOR) == 1, f"settings anchor: {out.count(SETTINGS_ANCHOR)}"
out = out.replace(SETTINGS_ANCHOR, TIER_FN + "\n" + SETTINGS_ANCHOR, 1)
print("inserted create_tier_model before `pub fn settings`")

if had_crlf:
    out = out.replace("\n", "\r\n")
open(P, "w", encoding="utf-8", newline="").write(out)
print(f"wrote {P} (crlf={had_crlf})")

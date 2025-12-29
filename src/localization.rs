use egui::Context;
use egui_l20n::{ContextExt as _, Localization};

/// Extension methods for [`Context`]
pub(crate) trait ContextExt {
    fn set_localizations(&self);
}

impl ContextExt for Context {
    fn set_localizations(&self) {
        self.set_localization(
            locales::EN,
            Localization::new(locales::EN).with_sources(sources::EN),
        );
        self.set_localization(
            locales::RU,
            Localization::new(locales::RU).with_sources(sources::RU),
        );
        self.set_language_identifier(locales::EN)
    }
}

mod locales {
    use egui_l20n::{LanguageIdentifier, langid};

    pub(super) const EN: LanguageIdentifier = langid!("en");
    pub(super) const RU: LanguageIdentifier = langid!("ru");
}

mod sources {
    use crate::asset;

    pub(super) const EN: &[&str] = &[asset!("/ftl/en/main.ftl"), asset!("/ftl/en/main.ext.ftl")];

    pub(super) const RU: &[&str] = &[asset!("/ftl/ru/main.ftl"), asset!("/ftl/ru/main.ext.ftl")];
}

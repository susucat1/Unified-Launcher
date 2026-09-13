use std::collections::HashMap;
use fluent_templates::fluent_bundle::FluentValue;
use fluent_templates::Loader;
use unic_langid::{langid, LanguageIdentifier};

fluent_templates::static_loader! {
    pub static LOCALES = {
        locales: "./assets/locales",
        fallback_language: "en-US"
    };
}

pub const US_ENGLISH: LanguageIdentifier = langid!("en-US");

#[inline(always)]
pub fn get_lang() -> &'static LanguageIdentifier {
    &US_ENGLISH
}

#[macro_export]
macro_rules! tr {
    ($id:expr) => {
        $crate::i18n::LOCALES.lookup($crate::i18n::get_lang(), $id)
    };

    ($id:expr, { $($key:literal = $value:expr),* $(,)? }) => {
        {
            let mut count = 0;
            $(
                let _ = $key;
                count += 1;
            )*

            let mut args = std::collections::HashMap::with_capacity(count);
            $(
                args.insert(
                    std::borrow::Cow::Borrowed($key), 
                    fluent_templates::fluent_bundle::FluentValue::from($value)
                );
            )*

            $crate::i18n::LOCALES.lookup_complete($crate::i18n::get_lang(), $id, Some(&args))
        }
    };
}

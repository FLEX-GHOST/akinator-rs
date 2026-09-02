use std::fmt;
use std::str::FromStr;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::error::Error;

/// Languages supported by the Akinator service.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Language {
    #[default]
    English,
    Arabic,
    French,
    Spanish,
    German,
    Italian,
    Japanese,
    Chinese,
    Russian,
    Portuguese,
    Turkish,
    Korean,
    Dutch,
    Polish,
    Indonesian,
    Hebrew,
}

impl Language {
    /// Returns the subdomain language code used by Akinator servers.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::English => "en",
            Self::Arabic => "ar",
            Self::French => "fr",
            Self::Spanish => "es",
            Self::German => "de",
            Self::Italian => "it",
            Self::Japanese => "jp",
            Self::Chinese => "cn",
            Self::Russian => "ru",
            Self::Portuguese => "pt",
            Self::Turkish => "tr",
            Self::Korean => "kr",
            Self::Dutch => "nl",
            Self::Polish => "pl",
            Self::Indonesian => "id",
            Self::Hebrew => "il",
        }
    }

    /// Resolves a language from its ISO/Akinator code.
    #[must_use]
    pub fn from_code(code: &str) -> Option<Self> {
        match code.trim().to_ascii_lowercase().as_str() {
            "en" | "english" => Some(Self::English),
            "ar" | "arabic" | "العربية" => Some(Self::Arabic),
            "fr" | "french" | "français" => Some(Self::French),
            "es" | "spanish" | "español" => Some(Self::Spanish),
            "de" | "german" | "deutsch" => Some(Self::German),
            "it" | "italian" | "italiano" => Some(Self::Italian),
            "jp" | "ja" | "japanese" | "日本語" => Some(Self::Japanese),
            "cn" | "zh" | "chinese" | "中文" => Some(Self::Chinese),
            "ru" | "russian" | "русский" => Some(Self::Russian),
            "pt" | "portuguese" | "português" => Some(Self::Portuguese),
            "tr" | "turkish" | "türkçe" => Some(Self::Turkish),
            "kr" | "ko" | "korean" | "한국어" => Some(Self::Korean),
            "nl" | "dutch" | "nederlands" => Some(Self::Dutch),
            "pl" | "polish" | "polski" => Some(Self::Polish),
            "id" | "indonesian" | "bahasa indonesia" => Some(Self::Indonesian),
            "il" | "he" | "hebrew" | "עבריت" | "עברית" => Some(Self::Hebrew),
            _ => None,
        }
    }

    /// English display name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::English => "English",
            Self::Arabic => "Arabic",
            Self::French => "French",
            Self::Spanish => "Spanish",
            Self::German => "German",
            Self::Italian => "Italian",
            Self::Japanese => "Japanese",
            Self::Chinese => "Chinese",
            Self::Russian => "Russian",
            Self::Portuguese => "Portuguese",
            Self::Turkish => "Turkish",
            Self::Korean => "Korean",
            Self::Dutch => "Dutch",
            Self::Polish => "Polish",
            Self::Indonesian => "Indonesian",
            Self::Hebrew => "Hebrew",
        }
    }

    /// Native language name.
    #[must_use]
    pub const fn native_name(&self) -> &'static str {
        match self {
            Self::English => "English",
            Self::Arabic => "العربية",
            Self::French => "Français",
            Self::Spanish => "Español",
            Self::German => "Deutsch",
            Self::Italian => "Italiano",
            Self::Japanese => "日本語",
            Self::Chinese => "中文",
            Self::Russian => "Русский",
            Self::Portuguese => "Português",
            Self::Turkish => "Türkçe",
            Self::Korean => "한국어",
            Self::Dutch => "Nederlands",
            Self::Polish => "Polski",
            Self::Indonesian => "Bahasa Indonesia",
            Self::Hebrew => "עברית",
        }
    }

    /// Returns localized answer labels in the order: Yes, No, Don't know, Probably, Probably not.
    #[must_use]
    pub const fn answer_labels(&self) -> [&'static str; 5] {
        match self {
            Self::English => ["Yes", "No", "Don't know", "Probably", "Probably not"],
            Self::Arabic => ["نعم", "لا", "لا أعرف", "غالباً نعم", "غالباً لا"],
            Self::French => ["Oui", "Non", "Je ne sais pas", "Probablement oui", "Probablement non"],
            Self::Spanish => ["Sí", "No", "No sé", "Probablemente sí", "Probablemente no"],
            Self::German => ["Ja", "Nein", "Ich weiß nicht", "Wahrscheinlich ja", "Wahrscheinlich nein"],
            Self::Italian => ["Sì", "No", "Non so", "Probabilmente sì", "Probabilmente no"],
            Self::Japanese => ["はい", "いいえ", "分からない", "たぶんそう", "たぶん違う"],
            Self::Chinese => ["是", "不是", "不知道", "可能是", "可能不是"],
            Self::Russian => ["Да", "Нет", "Я не знаю", "Возможно да", "Возможно нет"],
            Self::Portuguese => ["Sim", "Não", "Não sei", "Provavelmente sim", "Provavelmente não"],
            Self::Turkish => ["Evet", "Hayır", "Bilmiyorum", "Muhtemelen", "Muhtemelen değil"],
            Self::Korean => ["예", "아니오", "모르겠습니다", "그럴 겁니다", "아닐 겁니다"],
            Self::Dutch => ["Ja", "Nee", "Ik weet het niet", "Waarschijnlijk wel", "Waarschijnlijk niet"],
            Self::Polish => ["Tak", "Nie", "Nie wiem", "Chyba tak", "Chyba nie"],
            Self::Indonesian => ["Ya", "Tidak", "Saya tidak tahu", "Mungkin", "Mungkin tidak"],
            Self::Hebrew => ["כן", "לא", "אני לא יודע", "כנראה שכן", "כנראה שלא"],
        }
    }

    /// Returns list of themes supported for this language.
    #[must_use]
    pub const fn available_themes(&self) -> &'static [Theme] {
        match self {
            Self::English | Self::French | Self::Spanish | Self::German | Self::Italian | Self::Japanese => &[
                Theme::Characters,
                Theme::Objects,
                Theme::Animals,
            ],
            Self::Arabic | Self::Russian | Self::Portuguese | Self::Turkish | Self::Chinese | Self::Korean | Self::Dutch | Self::Polish | Self::Indonesian | Self::Hebrew => &[
                Theme::Characters,
            ],
        }
    }

    /// Checks whether a given theme is supported by this language.
    #[must_use]
    pub fn is_theme_supported(&self, theme: Theme) -> bool {
        self.available_themes().contains(&theme)
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.code())
    }
}

impl FromStr for Language {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_code(s).ok_or_else(|| Error::InvalidLanguage(s.to_string()))
    }
}

impl TryFrom<&str> for Language {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl Serialize for Language {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.code())
    }
}

impl<'de> Deserialize<'de> for Language {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_code(&s).ok_or_else(|| serde::de::Error::custom(format!("unknown language: {s}")))
    }
}

/// The gameplay category/theme in Akinator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(usize)]
pub enum Theme {
    #[default]
    Characters = 1,
    Objects = 2,
    Animals = 14,
}

impl Theme {
    /// Returns the internal numerical ID expected by the Akinator API.
    #[must_use]
    pub const fn id(&self) -> usize {
        *self as usize
    }

    /// Parses a theme from its numeric ID.
    #[must_use]
    pub const fn from_id(id: usize) -> Option<Self> {
        match id {
            1 => Some(Self::Characters),
            2 => Some(Self::Objects),
            14 => Some(Self::Animals),
            _ => None,
        }
    }

    /// English display name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Characters => "Characters",
            Self::Objects => "Objects",
            Self::Animals => "Animals",
        }
    }
}

impl fmt::Display for Theme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

impl FromStr for Theme {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "1" | "c" | "character" | "characters" | "person" | "شخصيات" | "شخصية" => Ok(Self::Characters),
            "2" | "o" | "object" | "objects" | "item" | "كائنات" | "أشياء" => Ok(Self::Objects),
            "14" | "a" | "animal" | "animals" | "حيوانات" | "حيوان" => Ok(Self::Animals),
            _ => Err(Error::InvalidTheme(s.to_string())),
        }
    }
}

impl TryFrom<usize> for Theme {
    type Error = Error;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Self::from_id(value).ok_or_else(|| Error::InvalidTheme(value.to_string()))
    }
}

impl TryFrom<&str> for Theme {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl Serialize for Theme {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(self.id() as u64)
    }
}

impl<'de> Deserialize<'de> for Theme {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let val = usize::deserialize(deserializer)?;
        Self::from_id(val).ok_or_else(|| serde::de::Error::custom(format!("unknown theme ID: {val}")))
    }
}

/// Answer choice provided in response to an Akinator question.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u8)]
pub enum Answer {
    #[default]
    Yes = 0,
    No = 1,
    DontKnow = 2,
    Probably = 3,
    ProbablyNot = 4,
}

impl Answer {
    /// Alias for `Answer::DontKnow` ("I don't know").
    pub const IDK: Self = Self::DontKnow;

    /// Returns the numerical answer code sent to Akinator servers.
    #[must_use]
    pub const fn id(&self) -> u8 {
        *self as u8
    }

    /// Converts a numeric answer ID (0..=4) to an [`Answer`].
    #[must_use]
    pub const fn from_id(id: u8) -> Option<Self> {
        match id {
            0 => Some(Self::Yes),
            1 => Some(Self::No),
            2 => Some(Self::DontKnow),
            3 => Some(Self::Probably),
            4 => Some(Self::ProbablyNot),
            _ => None,
        }
    }

    /// Localized string representation of this answer for a given language.
    #[must_use]
    pub const fn label(&self, lang: Language) -> &'static str {
        lang.answer_labels()[*self as usize]
    }
}

impl fmt::Display for Answer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label(Language::English))
    }
}

impl FromStr for Answer {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "0" | "y" | "yes" | "yeah" | "true" | "نعم" | "اي" | "اى" | "صحيح" | "oui" | "ja" | "si" | "sì" | "sim" | "tak" | "да" | "evet" | "예" | "はい" | "是" => {
                Ok(Self::Yes)
            }
            "1" | "n" | "no" | "nah" | "false" | "لا" | "كلا" | "non" | "nein" | "não" | "nie" | "нет" | "hayır" | "아니오" | "いいえ" | "不是" => {
                Ok(Self::No)
            }
            "2" | "i" | "idk" | "dont know" | "don't know" | "unknown" | "لا اعلم" | "لا أعرف" | "لا ادري" | "لا أدري" | "je ne sais pas" | "weiß nicht" | "non so" | "não sei" | "nie wiem" | "не знаю" | "bilmiyorum" | "모르겠습니다" | "分からない" | "不知道" => {
                Ok(Self::DontKnow)
            }
            "3" | "p" | "py" | "probably" | "probably yes" | "likely" | "ربما" | "غالبا" | "غالباً" | "غالبا نعم" | "غالباً نعم" | "ربما نعم" | "probablement oui" | "wahrscheinlich ja" | "probabilmente sì" | "provavelmente sim" | "chyba tak" | "возможно да" | "muhtemelen" | "그럴 겁니다" | "たぶんそう" | "可能是" => {
                Ok(Self::Probably)
            }
            "4" | "pn" | "probably not" | "unlikely" | "ربما لا" | "غالبا لا" | "غالباً لا" | "probablement non" | "wahrscheinlich nein" | "probabilmente no" | "provavelmente não" | "chyba nie" | "возможно нет" | "muhtemelen değil" | "아닐 겁니다" | "たぶん違う" | "可能不是" => {
                Ok(Self::ProbablyNot)
            }
            _ => Err(Error::InvalidAnswer(s.to_string())),
        }
    }
}

impl TryFrom<u8> for Answer {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::from_id(value).ok_or_else(|| Error::InvalidAnswer(value.to_string()))
    }
}

impl TryFrom<&str> for Answer {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl Serialize for Answer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u8(self.id())
    }
}

impl<'de> Deserialize<'de> for Answer {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let val = u8::deserialize(deserializer)?;
        Self::from_id(val).ok_or_else(|| serde::de::Error::custom(format!("unknown answer ID: {val}")))
    }
}

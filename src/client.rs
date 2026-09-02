use std::sync::Arc;
use std::time::Duration;

use regex::Regex;
use wreq::cookie::Jar;
use wreq::header::{HeaderMap, HeaderValue, ACCEPT, ACCEPT_LANGUAGE, REFERER, USER_AGENT};
use wreq::{Client, Proxy};

use crate::enums::{Answer, Language, Theme};
use crate::error::{Error, Result};
use crate::models::{Guess, RawStepResponse, SessionData, StepResult};

/// Builder for configuring and initializing an [`Akinator`] instance.
#[derive(Clone)]
pub struct AkinatorBuilder {
    language: Language,
    theme: Theme,
    child_mode: bool,
    proxy: Option<String>,
    timeout: Duration,
    user_agent: Option<String>,
    client: Option<Client>,
}

impl std::fmt::Debug for AkinatorBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AkinatorBuilder")
            .field("language", &self.language)
            .field("theme", &self.theme)
            .field("child_mode", &self.child_mode)
            .field("proxy", &self.proxy)
            .field("timeout", &self.timeout)
            .field("user_agent", &self.user_agent)
            .field("has_custom_client", &self.client.is_some())
            .finish()
    }
}

impl Default for AkinatorBuilder {
    fn default() -> Self {
        Self {
            language: Language::English,
            theme: Theme::Characters,
            child_mode: false,
            proxy: None,
            timeout: Duration::from_secs(30),
            user_agent: None,
            client: None,
        }
    }
}

impl AkinatorBuilder {
    /// Creates a new builder with default configuration.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the language for the Akinator game.
    #[must_use]
    pub fn language(mut self, language: Language) -> Self {
        self.language = language;
        self
    }

    /// Sets the theme for the Akinator game (Characters, Objects, Animals).
    #[must_use]
    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Enables or disables child mode (NSFW content filtering).
    #[must_use]
    pub fn child_mode(mut self, enabled: bool) -> Self {
        self.child_mode = enabled;
        self
    }

    /// Sets an HTTP/HTTPS/SOCKS proxy.
    #[must_use]
    pub fn proxy(mut self, proxy_url: impl Into<String>) -> Self {
        self.proxy = Some(proxy_url.into());
        self
    }

    /// Sets the request timeout duration.
    #[must_use]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Sets a custom User-Agent header string.
    #[must_use]
    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = Some(user_agent.into());
        self
    }

    /// Supplies a pre-configured [`wreq::Client`].
    #[must_use]
    pub fn custom_client(mut self, client: Client) -> Self {
        self.client = Some(client);
        self
    }

    /// Builds and returns the configured [`Akinator`] instance.
    ///
    /// # Errors
    /// Returns [`Error::ThemeNotSupportedForLanguage`] if the chosen theme is not available
    /// for the selected language, or [`Error::Http`] if HTTP client creation fails.
    pub fn build(self) -> Result<Akinator> {
        if !self.language.is_theme_supported(self.theme) {
            return Err(Error::ThemeNotSupportedForLanguage {
                theme: self.theme,
                language: self.language,
            });
        }

        let client = if let Some(c) = self.client {
            c
        } else {
            let cookie_jar = Arc::new(Jar::default());
            let mut builder = Client::builder()
                .cookie_provider(cookie_jar)
                .timeout(self.timeout);

            if let Some(ref proxy_str) = self.proxy {
                builder = builder.proxy(Proxy::all(proxy_str)?);
            } else if let Ok(proxy_env) = std::env::var("AKINATOR_PROXY")
                .or_else(|_| std::env::var("HTTPS_PROXY"))
                .or_else(|_| std::env::var("ALL_PROXY"))
                && !proxy_env.trim().is_empty()
            {
                builder = builder.proxy(Proxy::all(proxy_env.trim())?);
            }

            builder.build()?
        };

        let user_agent = self.user_agent.unwrap_or_else(|| {
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36".to_string()
        });

        Ok(Akinator {
            language: self.language,
            theme: self.theme,
            child_mode: self.child_mode,
            client,
            user_agent,
            url: format!("https://{}.akinator.com", self.language.code()),
            session: String::new(),
            signature: String::new(),
            identifiant: String::new(),
            question: String::new(),
            step: 0,
            progression: 0.0,
            step_last_proposition: String::new(),
            akitude: "defi.png".to_string(),
            started: false,
            won: false,
            ko: false,
            current_guess: None,
        })
    }
}

/// The core Akinator client and game session manager.
#[derive(Clone)]
pub struct Akinator {
    language: Language,
    theme: Theme,
    child_mode: bool,
    client: Client,
    user_agent: String,
    url: String,
    session: String,
    signature: String,
    identifiant: String,
    question: String,
    step: usize,
    progression: f32,
    step_last_proposition: String,
    akitude: String,
    started: bool,
    won: bool,
    ko: bool,
    current_guess: Option<Guess>,
}

impl std::fmt::Debug for Akinator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Akinator")
            .field("language", &self.language)
            .field("theme", &self.theme)
            .field("child_mode", &self.child_mode)
            .field("user_agent", &self.user_agent)
            .field("url", &self.url)
            .field("session", &self.session)
            .field("signature", &self.signature)
            .field("identifiant", &self.identifiant)
            .field("question", &self.question)
            .field("step", &self.step)
            .field("progression", &self.progression)
            .field("step_last_proposition", &self.step_last_proposition)
            .field("akitude", &self.akitude)
            .field("started", &self.started)
            .field("won", &self.won)
            .field("ko", &self.ko)
            .field("current_guess", &self.current_guess)
            .finish()
    }
}

impl Default for Akinator {
    fn default() -> Self {
        Self::builder().build().unwrap_or_else(|_| Self {
            language: Language::English,
            theme: Theme::Characters,
            child_mode: false,
            client: Client::new(),
            user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36".to_string(),
            url: "https://en.akinator.com".to_string(),
            session: String::new(),
            signature: String::new(),
            identifiant: String::new(),
            question: String::new(),
            step: 0,
            progression: 0.0,
            step_last_proposition: String::new(),
            akitude: "defi.png".to_string(),
            started: false,
            won: false,
            ko: false,
            current_guess: None,
        })
    }
}

impl Akinator {
    /// Creates a new [`Akinator`] instance with default English language and Characters theme.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a new builder for customizing the Akinator instance.
    #[must_use]
    pub fn builder() -> AkinatorBuilder {
        AkinatorBuilder::new()
    }

    /// The configured language.
    #[must_use]
    pub const fn language(&self) -> Language {
        self.language
    }

    /// The configured theme.
    #[must_use]
    pub const fn theme(&self) -> Theme {
        self.theme
    }

    /// Whether child mode is enabled.
    #[must_use]
    pub const fn child_mode(&self) -> bool {
        self.child_mode
    }

    /// The current question text.
    #[must_use]
    pub fn current_question(&self) -> &str {
        &self.question
    }

    /// The current question number (starts at 0).
    #[must_use]
    pub const fn step(&self) -> usize {
        self.step
    }

    /// Akinator's certainty progression percentage (0.0 to 100.0).
    #[must_use]
    pub const fn progression(&self) -> f32 {
        self.progression
    }

    /// Current visual mood/akitude filename.
    #[must_use]
    pub fn akitude(&self) -> &str {
        &self.akitude
    }

    /// The proposed character guess if Akinator has made a prediction.
    #[must_use]
    pub fn current_guess(&self) -> Option<&Guess> {
        self.current_guess.as_ref()
    }

    /// Returns whether the game has started.
    #[must_use]
    pub const fn is_started(&self) -> bool {
        self.started
    }

    /// Returns whether Akinator has produced a winning guess.
    #[must_use]
    pub const fn is_won(&self) -> bool {
        self.won
    }

    /// Returns whether Akinator has given up without finding a match.
    #[must_use]
    pub const fn is_given_up(&self) -> bool {
        self.ko
    }

    /// Returns the localized answer choices for the current language.
    #[must_use]
    pub const fn answers(&self) -> [&'static str; 5] {
        self.language.answer_labels()
    }

    /// Starts a new game session and returns the initial question.
    ///
    /// # Errors
    /// Returns [`Error`] if network communication fails, Cloudflare challenges the request,
    /// or session metadata cannot be extracted from the initial game page.
    pub async fn start(&mut self) -> Result<StepResult> {
        self.url = format!("https://{}.akinator.com", self.language.code());

        let headers = self.build_headers(false)?;

        let _ = self
            .client
            .get(format!("{}/", self.url))
            .headers(headers.clone())
            .send()
            .await?;

        let form_params = [
            ("sid", self.theme.id().to_string()),
            ("cm", self.child_mode.to_string()),
        ];

        let res = self
            .client
            .post(format!("{}/game", self.url))
            .headers(headers)
            .form(&form_params)
            .send()
            .await?;

        let body = res.text().await?;
        self.check_cloudflare(&body, "/game")?;

        let session_re = Regex::new(r#"name="session" id="session" value="([^"]+)""#)?;
        let signature_re = Regex::new(r#"name="signature" id="signature" value="([^"]+)""#)?;
        let question_re = Regex::new(r#"<div class="bubble-body"><p class="question-text" id="question-label">(.*?)</p></div>"#)?;
        let identifiant_re = Regex::new(r#"localStorage\.setItem\('identifiant', '([^']+)'\);"#)?;
        let akitude_re = Regex::new(r#"akitude[^"]*"[^"]*([^/]+\.png)""#)?;

        let session = session_re
            .captures(&body)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().to_string())
            .ok_or_else(|| Error::ExtractionFailed("session ID".to_string()))?;

        let signature = signature_re
            .captures(&body)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().to_string())
            .ok_or_else(|| Error::ExtractionFailed("signature".to_string()))?;

        let question = question_re
            .captures(&body)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().trim().to_string())
            .unwrap_or_else(|| "Question 1".to_string());

        let identifiant = identifiant_re
            .captures(&body)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().to_string())
            .unwrap_or_default();

        let akitude = akitude_re
            .captures(&body)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().to_string())
            .unwrap_or_else(|| "defi.png".to_string());

        self.session = session;
        self.signature = signature;
        self.identifiant = identifiant;
        self.question = question.clone();
        self.akitude = akitude.clone();
        self.step = 0;
        self.progression = 0.0;
        self.step_last_proposition = String::new();
        self.started = true;
        self.won = false;
        self.ko = false;
        self.current_guess = None;

        Ok(StepResult::Question {
            question,
            step: 0,
            progression: 0.0,
            akitude,
        })
    }

    /// Submits an answer to the current question and returns the next step result.
    ///
    /// # Errors
    /// Returns [`Error::SessionNotStarted`] if the game is not active,
    /// [`Error::SessionAlreadyWon`] if a guess is already pending, or network/API errors.
    pub async fn answer(&mut self, answer: Answer) -> Result<StepResult> {
        if !self.started {
            return Err(Error::SessionNotStarted);
        }
        if self.won {
            return Err(Error::SessionAlreadyWon);
        }
        if self.ko {
            return Err(Error::SessionGivenUp);
        }

        let form_params = [
            ("step", self.step.to_string()),
            ("progression", self.progression.to_string()),
            ("sid", self.theme.id().to_string()),
            ("cm", self.child_mode.to_string()),
            ("answer", answer.id().to_string()),
            ("step_last_proposition", self.step_last_proposition.clone()),
            ("session", self.session.clone()),
            ("signature", self.signature.clone()),
            ("identifiant", self.identifiant.clone()),
        ];

        let headers = self.build_headers(true)?;
        let res = self
            .client
            .post(format!("{}/answer", self.url))
            .headers(headers)
            .form(&form_params)
            .send()
            .await?;

        let body = res.text().await?;
        self.check_cloudflare(&body, "/answer")?;

        let raw: RawStepResponse = serde_json::from_str(&body)?;
        self.apply_step_response(raw)
    }

    /// Undoes the last answer and returns the previous question.
    ///
    /// # Errors
    /// Returns [`Error::CantGoBack`] if already on the first question (step 0),
    /// or network/API errors.
    pub async fn back(&mut self) -> Result<StepResult> {
        if !self.started {
            return Err(Error::SessionNotStarted);
        }
        if self.step == 0 {
            return Err(Error::CantGoBack);
        }

        let form_params = [
            ("step", self.step.to_string()),
            ("progression", self.progression.to_string()),
            ("sid", self.theme.id().to_string()),
            ("cm", self.child_mode.to_string()),
            ("session", self.session.clone()),
            ("signature", self.signature.clone()),
            ("identifiant", self.identifiant.clone()),
        ];

        let headers = self.build_headers(true)?;
        let res = self
            .client
            .post(format!("{}/cancel_answer", self.url))
            .headers(headers)
            .form(&form_params)
            .send()
            .await?;

        let body = res.text().await?;
        self.check_cloudflare(&body, "/cancel_answer")?;

        let raw: RawStepResponse = serde_json::from_str(&body)?;
        self.apply_step_response(raw)
    }

    /// Continues the game after rejecting Akinator's guess proposal.
    ///
    /// # Errors
    /// Returns [`Error::SessionNotStarted`] if the game is inactive,
    /// or network/API errors.
    pub async fn continue_game(&mut self) -> Result<StepResult> {
        if !self.started {
            return Err(Error::SessionNotStarted);
        }

        let form_params = [
            ("step", self.step.to_string()),
            ("progression", self.progression.to_string()),
            ("sid", self.theme.id().to_string()),
            ("cm", self.child_mode.to_string()),
            ("step_last_proposition", self.step_last_proposition.clone()),
            ("session", self.session.clone()),
            ("signature", self.signature.clone()),
            ("identifiant", self.identifiant.clone()),
        ];

        let headers = self.build_headers(true)?;
        let res = self
            .client
            .post(format!("{}/exclude", self.url))
            .headers(headers)
            .form(&form_params)
            .send()
            .await?;

        let body = res.text().await?;
        self.check_cloudflare(&body, "/exclude")?;

        self.won = false;
        self.current_guess = None;

        let raw: RawStepResponse = serde_json::from_str(&body)?;
        self.apply_step_response(raw)
    }

    /// Submits confirmation that Akinator correctly guessed the character.
    ///
    /// # Errors
    /// Returns [`Error::SessionNotStarted`] if no game is active,
    /// or network/API errors.
    pub async fn submit_win(&mut self) -> Result<()> {
        if !self.started {
            return Err(Error::SessionNotStarted);
        }

        let form_params = [
            ("step", self.step.to_string()),
            ("progression", self.progression.to_string()),
            ("sid", self.theme.id().to_string()),
            ("cm", self.child_mode.to_string()),
            ("step_last_proposition", self.step_last_proposition.clone()),
            ("session", self.session.clone()),
            ("signature", self.signature.clone()),
            ("identifiant", self.identifiant.clone()),
        ];

        let headers = self.build_headers(true)?;
        let res = self
            .client
            .post(format!("{}/choice", self.url))
            .headers(headers)
            .form(&form_params)
            .send()
            .await?;

        let body = res.text().await?;
        self.check_cloudflare(&body, "/choice")?;

        self.won = true;
        Ok(())
    }

    /// Fetches the raw image bytes for the current guess proposal, if a photo URL is available.
    ///
    /// # Errors
    /// Returns [`Error`] if downloading fails.
    pub async fn fetch_guess_photo(&self) -> Result<Option<Vec<u8>>> {
        let Some(guess) = &self.current_guess else {
            return Ok(None);
        };
        let Some(url) = guess.photo_url() else {
            return Ok(None);
        };
        let bytes = self.fetch_image_bytes(&url).await?;
        Ok(Some(bytes))
    }

    /// Fetches the current mood/akitude image bytes.
    ///
    /// # Errors
    /// Returns [`Error`] if downloading fails.
    pub async fn fetch_akitude_photo(&self) -> Result<Vec<u8>> {
        let url = format!("{}/assets/img/akitudes_670x1096/{}", self.url, self.akitude);
        self.fetch_image_bytes(&url).await
    }

    /// Fetches raw image bytes from an arbitrary URL using the configured client.
    ///
    /// # Errors
    /// Returns [`Error`] if downloading fails.
    pub async fn fetch_image_bytes(&self, url: &str) -> Result<Vec<u8>> {
        let res = self
            .client
            .get(url)
            .header(
                USER_AGENT,
                HeaderValue::from_str(&self.user_agent)
                    .map_err(|e| Error::AkinatorError(e.to_string()))?,
            )
            .header(
                ACCEPT,
                HeaderValue::from_static(
                    "image/avif,image/webp,image/apng,image/svg+xml,image/*,*/*;q=0.8",
                ),
            )
            .header(
                REFERER,
                HeaderValue::from_str(&format!("{}/", self.url))
                    .map_err(|e| Error::AkinatorError(e.to_string()))?,
            )
            .send()
            .await?;

        let bytes = res.bytes().await?;
        Ok(bytes.to_vec())
    }

    /// Downloads and saves the guess photo to a specified local file path, returning the path.
    /// Follows the RAM Ownership policy by writing directly to disk.
    ///
    /// # Errors
    /// Returns [`Error`] if downloading or file write fails.
    pub async fn fetch_guess_photo_to_file(
        &self,
        destination_path: impl AsRef<std::path::Path>,
    ) -> Result<Option<std::path::PathBuf>> {
        let Some(guess) = &self.current_guess else {
            return Ok(None);
        };
        let Some(url) = guess.photo_url() else {
            return Ok(None);
        };
        let dest = destination_path.as_ref().to_path_buf();
        let bytes = self.fetch_image_bytes(&url).await?;
        tokio::fs::write(&dest, bytes)
            .await
            .map_err(|e| Error::AkinatorError(e.to_string()))?;
        Ok(Some(dest))
    }

    /// Exports the current game state into a serializable [`SessionData`].
    #[must_use]
    pub fn export_session(&self) -> SessionData {
        SessionData {
            language: self.language,
            theme: self.theme,
            child_mode: self.child_mode,
            url: self.url.clone(),
            session: self.session.clone(),
            signature: self.signature.clone(),
            identifiant: self.identifiant.clone(),
            question: self.question.clone(),
            step: self.step,
            progression: self.progression,
            step_last_proposition: self.step_last_proposition.clone(),
            akitude: self.akitude.clone(),
            started: self.started,
            won: self.won,
            ko: self.ko,
            current_guess: self.current_guess.clone(),
        }
    }

    /// Reconstructs an [`Akinator`] instance from previously exported [`SessionData`].
    pub fn from_session(data: SessionData) -> Result<Self> {
        let cookie_jar = Arc::new(Jar::default());
        let client = Client::builder()
            .cookie_provider(cookie_jar)
            .timeout(Duration::from_secs(30))
            .build()?;

        let user_agent = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36".to_string();

        Ok(Self {
            language: data.language,
            theme: data.theme,
            child_mode: data.child_mode,
            client,
            user_agent,
            url: data.url,
            session: data.session,
            signature: data.signature,
            identifiant: data.identifiant,
            question: data.question,
            step: data.step,
            progression: data.progression,
            step_last_proposition: data.step_last_proposition,
            akitude: data.akitude,
            started: data.started,
            won: data.won,
            ko: data.ko,
            current_guess: data.current_guess,
        })
    }

    fn build_headers(&self, is_xhr: bool) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_str(&self.user_agent)
                .map_err(|e| Error::AkinatorError(e.to_string()))?,
        );
        headers.insert(
            ACCEPT_LANGUAGE,
            HeaderValue::from_static("ar,en-US;q=0.9,en;q=0.8"),
        );

        if is_xhr {
            headers.insert(
                ACCEPT,
                HeaderValue::from_static("application/json, text/javascript, */*; q=0.01"),
            );
            headers.insert(
                REFERER,
                HeaderValue::from_str(&format!("{}/game", self.url))
                    .map_err(|e| Error::AkinatorError(e.to_string()))?,
            );
            headers.insert(
                "origin",
                HeaderValue::from_str(&self.url)
                    .map_err(|e| Error::AkinatorError(e.to_string()))?,
            );
            headers.insert(
                "x-requested-with",
                HeaderValue::from_static("XMLHttpRequest"),
            );
        } else {
            headers.insert(
                ACCEPT,
                HeaderValue::from_static(
                    "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8",
                ),
            );
            headers.insert(
                REFERER,
                HeaderValue::from_str(&format!("{}/", self.url))
                    .map_err(|e| Error::AkinatorError(e.to_string()))?,
            );
            headers.insert(
                "origin",
                HeaderValue::from_str(&self.url)
                    .map_err(|e| Error::AkinatorError(e.to_string()))?,
            );
        }

        Ok(headers)
    }

    fn check_cloudflare(&self, body: &str, endpoint: &str) -> Result<()> {
        let trimmed = body.trim_start();
        if trimmed.starts_with('<')
            && (body.contains("<title>Just a moment...</title>")
                || body.contains("Attention Required! | Cloudflare")
                || body.contains("Sorry, you have been blocked")
                || body.contains("cf-browser-verification"))
        {
            return Err(Error::CloudflareBlocked {
                endpoint: endpoint.to_string(),
            });
        }
        Ok(())
    }

    fn apply_step_response(&mut self, data: RawStepResponse) -> Result<StepResult> {
        if let Some(id_prop) = data.id_proposition {
            let id_str = Self::json_value_to_string(&id_prop);
            if !id_str.is_empty() && id_str != "0" && id_str != "null" {
                self.won = true;
                let guess = Guess {
                    id: id_str,
                    name: data.name_proposition.unwrap_or_default(),
                    description: data.description_proposition.unwrap_or_default(),
                    photo: data.photo.filter(|p| !p.is_empty()),
                    pseudo: data.pseudo.filter(|p| !p.is_empty()),
                    flag_photo: data
                        .flag_photo
                        .and_then(|v| v.as_u64())
                        .map(|v| v as u32)
                        .unwrap_or(0),
                    base_proposition_id: data
                        .id_base_proposition
                        .map(|v| Self::json_value_to_string(&v)),
                };
                self.current_guess = Some(guess.clone());
                return Ok(StepResult::Guess(guess));
            }
        }

        if let Some(completion) = data.completion
            && completion.eq_ignore_ascii_case("KO")
        {
            self.ko = true;
            return Ok(StepResult::GiveUp);
        }

        if let Some(step_num) = data.step {
            self.step = Self::json_value_to_string(&step_num)
                .parse::<usize>()
                .unwrap_or(self.step + 1);
        }

        if let Some(prog) = data.progression {
            self.progression = Self::json_value_to_string(&prog)
                .parse::<f32>()
                .unwrap_or(self.progression);
        }

        if let Some(q) = data.question {
            self.question = q;
        }

        if let Some(akitude) = data.akitude {
            self.akitude = akitude;
        }

        if let Some(prop) = data.step_last_proposition {
            self.step_last_proposition = Self::json_value_to_string(&prop);
        }

        Ok(StepResult::Question {
            question: self.question.clone(),
            step: self.step,
            progression: self.progression,
            akitude: self.akitude.clone(),
        })
    }

    fn json_value_to_string(value: &serde_json::Value) -> String {
        match value {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Number(n) => n.to_string(),
            _ => String::new(),
        }
    }
}

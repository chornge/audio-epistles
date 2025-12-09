//! Web automation module for Spotify for Podcasters.
//!
//! This module uses Selenium WebDriver (via chromedriver and fantoccini) to
//! automate the process of uploading podcast episodes to Spotify for Podcasters.
//! It handles authentication, form filling, and saving episodes as drafts.

use anyhow::{Context, Result};
use dotenvy::dotenv;
use fantoccini::key::Key;
use fantoccini::{Client, Locator};
use rand::{rng, Rng};
use std::env;
use std::process::{Child, Command, Stdio};
use tokio::time::{sleep, Duration};
use tracing::{debug, info};

/// Guard struct that ensures chromedriver process is properly cleaned up
/// even if the upload fails partway through
struct ChromeDriverGuard {
    process: Child,
}

impl ChromeDriverGuard {
    /// Start chromedriver process
    fn new() -> std::io::Result<Self> {
        let process = Command::new("chromedriver")
            .arg("--port=64175")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;

        Ok(ChromeDriverGuard { process })
    }
}

impl Drop for ChromeDriverGuard {
    fn drop(&mut self) {
        // Attempt to kill the process
        let _ = self.process.kill();
        // CRITICAL: wait() to reap the process and avoid zombies
        let _ = self.process.wait();
    }
}

/// Add randomized human-like delay
async fn human_delay(min_ms: u64, max_ms: u64) {
    let mut rng = rng();
    let delay_ms = rng.random_range(min_ms..=max_ms);
    sleep(Duration::from_millis(delay_ms)).await;
}

/// Uploads a podcast episode to Spotify for Podcasters.
///
/// This function automates the complete workflow of uploading an audio file
/// to Spotify for Podcasters:
/// 1. Starts chromedriver
/// 2. Navigates to Spotify for Podcasters and logs in
/// 3. Delegates to `draft_episode` to upload the audio and fill in episode details
/// 4. Saves the episode as a draft
/// 5. Cleans up by closing the browser and killing chromedriver
///
/// The function uses randomized delays between actions to simulate human behavior
/// and avoid bot detection. Authentication credentials are read from environment
/// variables `SPOTIFY_EMAIL` and `SPOTIFY_PASSWORD`.
///
/// # Arguments
///
/// * `title` - The episode title to use when creating the draft
///
/// # Returns
///
/// Returns `Ok(())` on success.
///
/// # Errors
///
/// Returns an error if:
/// - chromedriver fails to start
/// - Required environment variables are not set
/// - WebDriver connection fails
/// - Any web element cannot be found or interacted with
/// - Login fails or authentication is rejected
/// - Episode upload or save fails
///
/// # Example
///
/// ```no_run
/// # tokio_test::block_on(async {
/// // Ensure SPOTIFY_EMAIL, SPOTIFY_PASSWORD, and AUDIO_FILE are set
/// audio_epistles::webdriver::upload("Sunday Service | Jan 1, 2024").await.unwrap();
/// # })
/// ```
#[allow(deprecated)]
#[allow(unused_variables)]
pub async fn upload(title: &str) -> Result<()> {
    dotenv().ok();

    // Start chromedriver with proper cleanup guard
    let _webdriver_guard =
        ChromeDriverGuard::new().context("Failed to start chromedriver")?;

    // Wait for chromedriver to be ready with a reasonable startup time
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    let email = env::var("SPOTIFY_EMAIL").context("SPOTIFY_EMAIL must be set")?;
    let password = env::var("SPOTIFY_PASSWORD").context("SPOTIFY_PASSWORD must be set")?;

    let client = Client::new("http://localhost:64175")
        .await
        .context("Failed to connect to WebDriver")?;

    client.goto("https://podcasters.spotify.com/").await?;
    human_delay(6000, 9000).await;

    // Click 'Log in'
    client
        .find(Locator::Css("a[href='/pod/login']"))
        .await?
        .click()
        .await?;
    human_delay(3500, 4500).await;

    // Click 'Continue with Spotify'
    client
        .find(Locator::XPath(
            "//span[text()='Continue with Spotify']/ancestor::button",
        ))
        .await?
        .click()
        .await?;
    human_delay(3500, 4500).await;

    // Input email
    client
        .find(Locator::Css("input#login-username"))
        .await?
        .send_keys(&email)
        .await?;
    human_delay(1000, 2000).await;

    // Click 'Continue'
    client
        .find(Locator::Css("button#login-button"))
        .await?
        .click()
        .await?;
    human_delay(6800, 7800).await;

    // Click 'Log in with a password'
    if let Ok(password_btn) = client
        .find(Locator::Css(
            "button[data-encore-id='buttonTertiary'].Button-sc-1dqy6lx-0",
        ))
        .await
    {
        password_btn.click().await?;
        human_delay(1200, 2000).await;
    }
    human_delay(6200, 8200).await;

    // Input password
    client
        .wait()
        .for_element(Locator::Css("input[data-testid='login-password']"))
        .await?
        .send_keys(&password)
        .await?;
    human_delay(7700, 8800).await;

    // Click 'Log In'
    let login_btn = client
    .wait()
    .for_element(Locator::XPath(
        "//button[@id='login-button' or @data-testid='login-button' or .//span[translate(normalize-space(text()), 'abcdefghijklmnopqrstuvwxyz', 'ABCDEFGHIJKLMNOPQRSTUVWXYZ')='LOG IN']]",
    ))
    .await?;

    login_btn.click().await?;
    human_delay(4000, 6000).await;

    info!("Spotify login successful");

    draft_episode(title, &client).await?;

    client.close().await?;
    // No need to manually kill webdriver - ChromeDriverGuard's Drop will handle it

    Ok(())
}

/// Creates a draft episode on Spotify for Podcasters.
///
/// This function handles the episode creation workflow after authentication:
/// 1. Navigates to the episode upload wizard
/// 2. Uploads the audio file specified in the `AUDIO_FILE` environment variable
/// 3. Fills in the episode title
/// 4. Sets a default description
/// 5. Saves the episode as a draft
///
/// The audio file path must be an absolute path and is read from the `AUDIO_FILE`
/// environment variable. The function uses character-by-character input for the
/// description to work around Spotify's Slate.js rich text editor.
///
/// # Arguments
///
/// * `title` - The episode title to set
/// * `client` - A reference to the authenticated WebDriver client
///
/// # Returns
///
/// Returns `Ok(())` on success.
///
/// # Errors
///
/// Returns an error if:
/// - The `AUDIO_FILE` environment variable is not set
/// - Navigation to the episode wizard fails
/// - Audio file upload fails or times out
/// - Any form field cannot be found or filled
/// - Saving the draft fails
///
/// # Example
///
/// ```no_run
/// # use fantoccini::Client;
/// # tokio_test::block_on(async {
/// let client = Client::new("http://localhost:64175").await.unwrap();
/// // ... authenticate first ...
/// audio_epistles::webdriver::draft_episode("My Episode", &client).await.unwrap();
/// # })
/// ```
#[allow(deprecated)]
pub async fn draft_episode(title: &str, client: &Client) -> Result<()> {
    dotenv().ok();

    // Go to episode upload wizard (logged in already)
    human_delay(9000, 10000).await;
    client
        .goto("https://podcasters.spotify.com/pod/dashboard/episode/wizard")
        .await?;
    human_delay(7600, 9300).await;

    // Upload audio
    if let Ok(select_btn) = client
        .find(Locator::XPath(
            "//span[text()='Select a file']/ancestor::button",
        ))
        .await
    {
        select_btn.click().await?;
    }
    human_delay(2200, 3000).await;

    let audio_file_path = env::var("AUDIO_FILE").context("AUDIO_FILE must be set")?;
    client
        .find(Locator::Css("input[type='file']"))
        .await?
        .send_keys(&audio_file_path)
        .await?;

    human_delay(55800, 62400).await;
    debug!("Audio uploaded");

    // Set title
    client
        .find(Locator::Css("input#title-input"))
        .await?
        .send_keys(title)
        .await?;
    human_delay(1000, 2000).await;
    debug!("Title entered");

    let description = "Join us online for our Sunday services @ 9AM & 11AM.";
    let desc_field = client
        .find(Locator::Css(
            "div[role='textbox'][data-slate-editor='true']",
        ))
        .await?;

    // Focus the description field (double click for some editors)
    desc_field.click().await?;
    human_delay(200, 400).await;
    desc_field.click().await?;
    human_delay(400, 800).await;

    // Select all (Cmd + A on macOS, Ctrl + A otherwise)
    #[cfg(target_os = "macos")]
    desc_field.send_keys(&(Key::Meta + "a")).await?;
    #[cfg(not(target_os = "macos"))]
    desc_field.send_keys(&(Key::Control + "a")).await?;

    human_delay(200, 400).await;

    // Delete selected text
    desc_field.send_keys(&Key::Backspace).await?;
    human_delay(400, 800).await;

    // Click to ensure focus is reset
    desc_field.click().await?;
    human_delay(200, 400).await;

    // Set description (works better for Spotify's text editor Slate.js)
    for c in description.chars() {
        desc_field.send_keys(&c.to_string()).await?;
        human_delay(50, 120).await;
    }
    debug!("Description entered");
    human_delay(1000, 2000).await;

    // schedule_episode(&client).await?;

    // Click 'Close' to trigger save-draft modal/dialog
    if let Ok(close_btn) = client
        .find(Locator::Css(
            "button[aria-label='Close'][data-encore-id='buttonTertiary']",
        ))
        .await
    {
        close_btn.click().await?;
    }
    human_delay(2700, 3100).await;

    // Click 'Save draft' in the dialog
    if let Ok(save_btn) = client
        .find(Locator::XPath(
            "//span[text()='Save draft']/ancestor::button",
        ))
        .await
    {
        save_btn.click().await?;
    }

    human_delay(1200, 2000).await;
    info!("Episode successfully saved as draft");

    // Allow Spotify UI to settle
    human_delay(3000, 5000).await;

    Ok(())
}

/// Handles publishing episode to Spotify/Anchor.fm
#[allow(dead_code)]
async fn schedule_episode(client: &Client) -> Result<()> {
    // Click 'Next' button
    if let Ok(next_button) = client
        .find(Locator::Css("button[form='details-form'][type='submit']"))
        .await
    {
        next_button.click().await?;
        debug!("Clicked 'Next'");
    }
    human_delay(19000, 22000).await; // Wait for UI to load

    // Click 'Now' option
    if let Ok(now_option) = client.find(Locator::Css("input#publish-date-now")).await {
        now_option.click().await?;
        debug!("Selected 'Now' for publishing");
    }
    human_delay(3000, 5000).await;

    // Click 'Schedule' button
    if let Ok(schedule_button) = client
        .find(Locator::Css("button[form='review-form'][type='submit']"))
        .await
    {
        schedule_button.click().await?;
    }
    human_delay(6000, 9000).await;
    info!("Episode successfully published");

    Ok(())
}

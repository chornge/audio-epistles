use dotenvy::dotenv;
use fantoccini::key::Key;
use fantoccini::{error, Client, Locator};
use rand::{rng, Rng};
use std::env;
use std::process::{Child, Command, Stdio};
use tokio::time::{sleep, Duration};

fn start_webdriver() -> std::io::Result<Child> {
    Command::new("chromedriver")
        .arg("--port=64175")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
}

/// Add randomized human-like delay
async fn human_delay(min_ms: u64, max_ms: u64) {
    let mut rng = rng();
    let delay_ms = rng.random_range(min_ms..=max_ms);
    sleep(Duration::from_millis(delay_ms)).await;
}

/// Schedule episode on Spotify/Anchor.fm
#[allow(deprecated)]
#[allow(unused_variables)]
#[allow(clippy::zombie_processes)]
pub async fn schedule(title: &str) -> Result<(), error::CmdError> {
    dotenv().ok();

    let mut webdriver = start_webdriver().expect("Failed to start chromedriver");

    // Wait for chromedriver to be ready
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    let email = env::var("SPOTIFY_EMAIL").expect("SPOTIFY_EMAIL must be set");
    let password = env::var("SPOTIFY_PASSWORD").expect("SPOTIFY_PASSWORD must be set");

    let client = Client::new("http://localhost:64175")
        .await
        .expect("Failed to connect to WebDriver");

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

    println!("✅ Spotify login successful!");

    draft_episode(title, &client).await?;

    client.close().await?;
    webdriver.kill().expect("Failed to terminate chromedriver");

    Ok(())
}

/// Handles saving episode as draft after filling in episode details
#[allow(deprecated)]
pub async fn draft_episode(title: &str, client: &Client) -> Result<(), error::CmdError> {
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

    let audio_file_path = env::var("AUDIO_FILE").expect("AUDIO_FILE must be set");
    client
        .find(Locator::Css("input[type='file']"))
        .await?
        .send_keys(&audio_file_path)
        .await?;

    human_delay(55800, 62400).await;
    println!("Audio uploaded.");

    // Set title
    client
        .find(Locator::Css("input#title-input"))
        .await?
        .send_keys(title)
        .await?;
    human_delay(1000, 2000).await;
    println!("Title entered.");

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
    println!("Description entered.");
    human_delay(1000, 2000).await;

    println!("Episode details filled.");
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
    println!("✅ Episode successfully saved as draft.");

    // Allow Spotify UI to settle
    human_delay(3000, 5000).await;

    Ok(())
}

/// Handles publishing episode to Spotify/Anchor.fm
#[allow(dead_code)]
async fn schedule_episode(client: &Client) -> Result<(), error::CmdError> {
    // Click 'Next' button
    if let Ok(next_button) = client
        .find(Locator::Css("button[form='details-form'][type='submit']"))
        .await
    {
        next_button.click().await?;
        println!("➡️ Clicked Next button.");
    }
    human_delay(19000, 22000).await; // Wait ~20 seconds

    // Click 'Now' option
    if let Ok(now_option) = client.find(Locator::Css("input#publish-date-now")).await {
        now_option.click().await?;
        println!("🕒 Selected 'Now' publishing option.");
    }
    human_delay(5000, 6000).await; // Wait ~5 seconds

    // Click 'Schedule' button
    if let Ok(schedule_button) = client
        .find(Locator::Css("button[form='review-form'][type='submit']"))
        .await
    {
        schedule_button.click().await?;
        println!("✅ Clicked Schedule button.");
    }

    Ok(())
}

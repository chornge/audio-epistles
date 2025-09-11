use dotenvy::dotenv;
// use fantoccini::{Client, Locator};
use rand::{rng, Rng};
// use std::env;
use tokio::time::{sleep, Duration};

#[allow(dead_code)]
/// Add randomized human-like delay
async fn human_delay(min_ms: u64, max_ms: u64) {
    let mut rng = rng();
    let delay_ms = rng.random_range(min_ms..=max_ms);
    sleep(Duration::from_millis(delay_ms)).await;
}

#[tokio::main]
#[allow(dead_code)]
#[allow(deprecated)]
pub async fn schedule() -> Result<(), fantoccini::error::CmdError> {
    dotenv().ok();

    // let email = env::var("SPOTIFY_EMAIL").expect("SPOTIFY_EMAIL must be set");
    // let password = env::var("SPOTIFY_PASSWORD").expect("SPOTIFY_PASSWORD must be set");

    // let client = Client::new("http://localhost:9515")
    //     .await
    //     .expect("failed to connect to WebDriver");

    // client.goto("https://podcasters.spotify.com/").await?;
    // human_delay(6000, 9000).await;

    // // Click "Log in"
    // client
    //     .find(Locator::Css("a[href='/login']"))
    //     .await?
    //     .click()
    //     .await?;
    // human_delay(3500, 4500).await;

    // // Input email
    // client
    //     .find(Locator::Css("input#login-username"))
    //     .await?
    //     .send_keys(&email)
    //     .await?;
    // human_delay(1000, 2000).await;

    // // Click Login button (initial)
    // client
    //     .find(Locator::Css("button#login-button"))
    //     .await?
    //     .click()
    //     .await?;
    // human_delay(1800, 2800).await;

    // // Optional: Click 'Login with password' if needed
    // if let Ok(pw_input) = client.find(Locator::Css("input#login-password")).await {
    //     pw_input.send_keys(&password).await?;
    //     human_delay(1600, 3300).await;

    //     client
    //         .find(Locator::Css("button#login-button"))
    //         .await?
    //         .click()
    //         .await?;
    // }

    // println!("Logged in. Waiting...");
    // human_delay(4000, 6000).await;

    // // Go to episode upload wizard
    // client
    //     .goto("https://podcasters.spotify.com/pod/dashboard/episode/wizard")
    //     .await?;
    // human_delay(7600, 9300).await;

    // // Upload audio
    // let audio_file_path = env::var("AUDIO_FILE").expect("AUDIO_FILE must be set");
    // client
    //     .find(Locator::Css("input[type='file']"))
    //     .await?
    //     .send_keys(&audio_file_path)
    //     .await?;

    // println!("Audio uploaded. Waiting...");
    // human_delay(15800, 22400).await;

    // // Set title
    // let title = env::var("UPLOAD_TITLE").unwrap_or_else(|_| "Untitled Episode".to_string());
    // client
    //     .find(Locator::Css("#title-input"))
    //     .await?
    //     .send_keys(&title)
    //     .await?;
    // human_delay(1000, 2000).await;

    // // Set description
    // let description = env::var("UPLOAD_DESCRIPTION").unwrap_or_default();
    // client
    //     .find(Locator::Css("div[role='textbox']"))
    //     .await?
    //     .send_keys(&description)
    //     .await?;
    // human_delay(1000, 2000).await;

    // println!("Details filled.");

    // // Click "Next"
    // client
    //     .find(Locator::XPath("//span[text()='Next']/parent::button"))
    //     .await?
    //     .click()
    //     .await?;
    // human_delay(2000, 3000).await;

    // // Choose "Publish now"
    // client
    //     .find(Locator::Css("input#publish-date-now"))
    //     .await?
    //     .click()
    //     .await?;
    // human_delay(1000, 2000).await;

    // // Click "Publish"
    // client
    //     .find(Locator::XPath("//span[text()='Publish']/parent::button"))
    //     .await?
    //     .click()
    //     .await?;

    println!("✅ Episode published!");

    // // Allow Spotify UI to settle
    // human_delay(3000, 5000).await;

    // // Close browser
    // client.close().await?;

    Ok(())
}

# Audio Epistles

![CI/CD](https://github.com/chornge/audio-epistles/actions/workflows/build.yml/badge.svg?branch=main)

An automated service for fetching the latest video from a YouTube playlist, extracting its details, and publishing the audio to Spotify/Anchor.fm. The service is designed to run periodically (e.g., every hour via cron), ensuring new podcasts are published reliably.

## Architecture

![Design Doc](./DESIGN-DOC.excalidraw.png)

- **Video Service:** Fetches the latest video ID from a YouTube playlist.
- **Processor Service:** Checks if ID in Database and triggers the Chrome WebDriver.
- **Episode Service:** Extracts sermon chapter, trims audio and stores audio file.
- **WebDriver Service:** Bundles audio file, title & description into an episode to upload to Spotify/Anchor.fm.
- **CronJob:** Runs the application every hour.

## Features/Qualities

- **Automated Fetch & Publish:** Finds the most recent sermon in a predetermined playlist and publishes it to Spotify.
- **Low Latency:** Publishes new sermons within ~1 hour of playlist upload.
- **No Double Publishing:** Keeps track of the last published video to avoid duplicates.

## File Structure

```
audio_epistles/
├── assets/                 # Stores downloaded files - video.mp4 and audio.mp3
├── src/
│   ├── db.rs               # Performs Database operations
│   ├── episode.rs          # Extracts and trims audio
│   ├── main.rs             # Entry point
│   ├── processor.rs        # Triggers WebDriver
│   └── video.rs            # Fetches latest video ID from YouTube
│   ├── webdriver.rs        # Handles publishing to Spotify/Anchor.fm
├── .env                    # Stores sermon playlist ID, audio file path & DB url
├── build.rs                # Ensures Database exists
├── Cargo.toml
├── LICENSE
├── README
└── videos.db               # Stores last uploaded video ID
```

## Requirements

- Rust ([rustup](https://rustup.rs/))
- FFMPEG (`brew install ffmpeg` on macOS)
- yt-dlp (`brew install yt-dlp` on macOS)
- chromedriver (`brew install chromedriver` on macOS)

## Setup

**Clone and Setup Repo**

```
git clone https://github.com/chornge/audio-epistles.git
cd audio-epistles
touch .env
```

Copy the following into .env and replace with the appropriate values

```
SPOTIFY_EMAIL=email@spotify.com
SPOTIFY_PASSWORD=password@spotify
SERMON_PLAYLIST_ID=playlist@id
AUDIO_FILE=path/to/audio-epistles/assets/audio.mp3
DB_URL=videos.db
```

More secure way to store Spotify credentials is to host on Github/GitLab & store as secrets.

## Automation

To use cron, run `crontab -e`

For every hour (on the hour), paste into crontab:

```
0 * * * * cd audio-epistles && cargo run --release >> cron.log 2>&1
```

OR to run Weekdays (M-F) at noon, paste:

```
0 12 * * 1-5 cd audio-epistles && cargo run --release >> cron.log 2>&1
```

OR to run Wednesdays at 9:00pm, paste:

```
0 21 * * 3 cd audio-epistles && cargo run --release >> cron.log 2>&1
```

## Build & Run App

```
cargo build --release
cargo run --release
```

## License

MIT

## Special Thanks

A special shout-out to the authors and contributors of [Schroedinger-Hat](https://github.com/Schroedinger-Hat/youtube-to-spotify), whose work directly inspired this project. Your efforts in building and maintaining youtube-to-spotify make seamless publishing possible. Thank you for your dedication and open-source spirit!

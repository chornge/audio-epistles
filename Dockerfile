# Stage 1: Build
FROM rust:1.85.1-slim-bookworm AS builder

WORKDIR /app

COPY . .

RUN apt-get update && apt-get install -y \
    ffmpeg \
    libsqlite3-dev \
    pkg-config \
    libssl-dev \
    python3 \
    python3-venv \
    python3-pip \
    && python3 -m venv /venv \
    && /venv/bin/pip install --upgrade pip \
    && /venv/bin/pip install yt-dlp \
    && cp /venv/bin/yt-dlp /usr/local/bin/yt-dlp \
    && cargo build --release \
    && rm -rf /var/lib/apt/lists/*

# Stage 2: Run
FROM debian:bookworm-slim AS runner

WORKDIR /app

# Note: .env should be passed at runtime via docker-compose or environment variables
# Do NOT copy .env into the image - it embeds credentials in image layers
COPY --from=builder /app/init-db.sh .
COPY --from=builder /app/videos.db ./videos.db.template
COPY --from=builder /usr/local/bin/yt-dlp /usr/local/bin/yt-dlp
COPY --from=builder /app/target/release/audio_epistles ./audio_epistles

# Install Chrome, ChromeDriver, ffmpeg, and dependencies
RUN apt-get update && apt-get install -y \
    wget \
    gnupg \
    unzip \
    ca-certificates \
    ffmpeg \
    libasound2 \
    libcups2 \
    libdbus-1-3 \
    libgbm1 \
    libgtk-3-0 \
    libnspr4 \
    libnss3 \
    libsqlite3-0 \
    libxcomposite1 \
    libxcursor1 \
    libxdamage1 \
    libxkbcommon0 \
    libxrandr2 \
    libxshmfence1 \
    libxtst6 \
    xdg-utils \
    # Modern apt key management (apt-key is deprecated)
    && mkdir -p /etc/apt/keyrings \
    && wget -q -O /etc/apt/keyrings/google-chrome.asc https://dl.google.com/linux/linux_signing_key.pub \
    && echo "deb [arch=amd64 signed-by=/etc/apt/keyrings/google-chrome.asc] http://dl.google.com/linux/chrome/deb/ stable main" > /etc/apt/sources.list.d/google-chrome.list \
    && apt-get update && apt-get install -y google-chrome-stable \
    # Install matching ChromeDriver dynamically based on installed Chrome version
    && CHROME_VERSION=$(google-chrome --version | grep -oP '\d+\.\d+\.\d+\.\d+') \
    && echo "Chrome version: $CHROME_VERSION" \
    && wget -q -O /tmp/chromedriver.zip "https://storage.googleapis.com/chrome-for-testing-public/${CHROME_VERSION}/linux64/chromedriver-linux64.zip" \
    && unzip /tmp/chromedriver.zip -d /tmp/ \
    && mv /tmp/chromedriver-linux64/chromedriver /usr/local/bin/chromedriver \
    && chmod +x /usr/local/bin/chromedriver \
    && rm -rf /tmp/chromedriver.zip /tmp/chromedriver-linux64 \
    && rm -rf /var/lib/apt/lists/* \
    && chmod +x init-db.sh

ENTRYPOINT [ "./init-db.sh" ]
CMD [ "./audio_epistles", "--no-sandbox", "--disable-dev-shm-usage" ]

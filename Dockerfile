# Stage 1: Install
FROM node:18-slim AS installer

WORKDIR /app

# Force Puppeteer to download Chromium
ENV PUPPETEER_SKIP_DOWNLOAD=false \
    PUPPETEER_CACHE_DIR=/app/.cache/puppeteer

RUN npm install puppeteer

# Stage 2: Build
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

# Stage 3: Run
FROM debian:bookworm-slim AS runner

WORKDIR /app

# Note: .env should be passed at runtime via docker-compose or environment variables
# Do NOT copy .env into the image - it embeds credentials in image layers
COPY --from=builder /app/init-db.sh .
COPY --from=builder /usr/bin/ffmpeg /usr/bin/ffmpeg
COPY --from=builder /usr/bin/python3 /usr/bin/python3
COPY --from=builder /app/videos.db ./videos.db.template
COPY --from=builder /usr/local/bin/yt-dlp /usr/local/bin/yt-dlp
COPY --from=builder /app/target/release/audio_epistles ./audio_epistles

# Install Chrome
RUN apt-get update && apt-get install -y \
    wget \
    gnupg \
    unzip \
    ca-certificates \
    && wget -q -O - https://dl.google.com/linux/linux_signing_key.pub | apt-key add - \
    && echo "deb [arch=amd64] http://dl.google.com/linux/chrome/deb/ stable main" > /etc/apt/sources.list.d/google-chrome.list \
    && apt-get update && apt-get install -y google-chrome-stable

# Install ChromeDriver
RUN wget -q -P /tmp/ https://storage.googleapis.com/chrome-for-testing-public/141.0.7390.76/linux64/chromedriver-linux64.zip \
    && unzip /tmp/chromedriver-linux64.zip -d /usr/local/bin/chromedriver \
    && chmod +x /usr/local/bin/chromedriver \
    && rm -rf /tmp/chromedriver-linux64.zip

# Additional dependencies for Chromium & related tools
RUN apt-get install -y \
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
    && rm -rf /var/lib/apt/lists/*

# Setup Puppeteer
COPY --from=installer /app/.cache/puppeteer /usr/bin/.local-chromium

# Create symlink to ChromeDriver and set permissions
RUN ln -s /usr/bin/.local-chromium/chrome-linux/chromedriver /usr/bin/chromedriver \
    && chmod +x init-db.sh

ENTRYPOINT [ "./init-db.sh" ]
CMD [ "./audio_epistles", "--no-sandbox", "--disable-dev-shm-usage" ]

FROM rust:1.86

RUN apt-get update \
    && apt-get install -y --no-install-recommends curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .

WORKDIR /app/server
EXPOSE 3001

CMD ["cargo", "run", "--bin", "agora-server"]

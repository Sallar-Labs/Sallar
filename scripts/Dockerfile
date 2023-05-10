FROM debian:bullseye-slim

WORKDIR /usr/src/app
ARG SOLANA_VERSION
ARG ANCHOR_VERSION

### Install Required Packages ###
RUN apt update
RUN apt install -y curl

### Install Rust ###
RUN curl --proto '=https' --tlsv1.2 -sSf "https://sh.rustup.rs" | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN ["/bin/bash", "-c", "source $HOME/.cargo/env"]
RUN rustup component add rustfmt

### Install Solana ###
RUN sh -c "$(curl -sSfL https://release.solana.com/v${SOLANA_VERSION}/install)"
ENV PATH="/root/.local/share/solana/install/active_release/bin:${PATH}"

### Generate Solana keypair ###
RUN solana-keygen new --no-bip39-passphrase -s -o ~/.config/solana/id.json &&\
    solana config set --keypair ~/.config/solana/id.json

### Install Node ###
RUN curl -fsSL https://deb.nodesource.com/setup_19.x | bash - && apt install -y nodejs

### Install Yarn ###
RUN corepack enable
RUN corepack prepare yarn@stable --activate
RUN yarn set version classic

### Install Anchor ###
RUN apt install -y pkg-config build-essential libudev-dev libssl-dev
RUN cargo install --git https://github.com/project-serum/anchor avm --locked --force
RUN avm install $ANCHOR_VERSION
RUN avm use $ANCHOR_VERSION

### Copy Sallar to Docker ###
COPY Anchor.toml Cargo.toml package.json tsconfig.json rust-toolchain.toml .
COPY programs programs
COPY tests tests

### Build Sallar ###
RUN anchor build
RUN cargo build
RUN yarn install

### Update Program ID ###
RUN programId=$(solana address -k target/deploy/sallar-keypair.json) && \
    sed -i 's/sallar = ".*/sallar = "'"$programId"'"/' Anchor.toml && \
    sed -i 's/declare_id!.*/declare_id!("'"$programId"'");/' programs/sallar/src/lib.rs && \
    anchor build

### Configure Solana to use local test Solana validator by default ###
RUN solana config set --url http://localhost:8899

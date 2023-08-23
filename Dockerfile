# In this one, we are going to use cargo chef for compiling the dependencies in a separate step, so that the results of that step are cacheable

FROM lukemathwalker/cargo-chef:latest-rust-1.63.0 as chef

WORKDIR /app

RUN apt update && apt install lld clang -y

FROM chef as planner

COPY . .

# Computes a lock file for the project
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder

COPY --from=planner /app/recipe.json recipe.json

# build the project dependencies
RUN cargo chef cook --release --recipe-path recipe.json

# evth up to this point will stay cached as long as our dependencies do not change

COPY . .

ENV SQLX_OFFLINE true

RUN cargo build --release --bin zero_2_prod

FROM debian:bullseye-slim AS runtime

WORKDIR /app

RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates \
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/zero_2_prod zero_2_prod
COPY configuration configuration
ENV APP_ENVIRONMENT production
ENTRYPOINT [ "./zero_2_prod" ]


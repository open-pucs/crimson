FROM rust:1.86 as build

# create a new empty shell project
RUN USER=root cargo new --bin crimson
WORKDIR /crimson

# copy over your manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# this build step will cache your dependencies
RUN cargo build --release
RUN rm src/*.rs

# copy your source tree
COPY ./src ./src

# build for release
RUN cargo build --release

# our final base
FROM alpine:latest

# copy the build artifact from the build stage
COPY --from=build /crimson/target/release/crimson .

# set the startup command to run your binary
CMD ["./crimson"]



FROM rust:slim-bullseye AS builder
WORKDIR /app
RUN apt-get update
RUN apt-get install -y protobuf-compiler pkg-config libssl-dev
COPY . .
RUN cargo build --release
EXPOSE 9000
# ENV PORT=9000
ENTRYPOINT ["/app/target/release/veresiye-paxos"]

# FROM scratch
# USER 1000:1000
# COPY --from=builder ./app/target/release/veresiye-paxos /veresiye-paxos

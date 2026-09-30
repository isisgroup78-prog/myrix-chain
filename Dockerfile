FROM rust:1.80
WORKDIR /app
COPY . ./
RUN cargo build --release
EXPOSE 3000
CMD ["./target/release/myrix_chain"]

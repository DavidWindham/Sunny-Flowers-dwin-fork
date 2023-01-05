FROM rust:latest

# Dependencies
RUN apt-get update
RUN apt-get install -y ffmpeg
RUN apt-get install -y youtube-dl
# Currently youtube-dl seems to have some weird bug in it? This fixes it, no idea why currently, not worth investigating for now"
RUN apt-get install -y python3-pip
RUN pip install --upgrade youtube-dl

WORKDIR /Sunny-Flowers-dwin-fork
COPY Cargo.toml .
COPY . .
RUN cargo build --release

CMD ["/Sunny-Flowers-dwin-fork/target/release/sunny-flowers-dwin-fork"]

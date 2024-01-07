FROM ubuntu:22.04

# Update default packages
RUN apt-get update

# Get Ubuntu packages
RUN apt install -y \
    build-essential \
    curl

# Update new packages
RUN apt update

# Get Rust
RUN curl https://sh.rustup.rs -sSf | bash -s -- -y

ENV PATH="/root/.cargo/bin:${PATH}"


RUN apt install -y software-properties-common
RUN add-apt-repository ppa:deadsnakes/ppa
RUN apt update
RUN DEBIAN_FRONTEND=noninteractive apt install -y python3.12
RUN apt install -y python3.12-distutils
RUN apt install -y python3.12-venv
RUN python3.12 -m ensurepip

COPY . /app
WORKDIR /app
RUN python3.12 -m pip install -r requirements.txt
RUN cargo build --release

WORKDIR /app/scripts
ENTRYPOINT ["python3.12", "-u", "lichess-bot.py"]
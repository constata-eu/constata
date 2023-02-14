FROM ubuntu:18.04
 
RUN apt-get update && apt-get install -y curl gettext
RUN apt-get install build-essential libssl-dev pkg-config -y
 
RUN mkdir -p /user/constata-rust-builder/src
WORKDIR /user/constata-rust-builder/src

RUN curl --proto '=https' --tls1.2 -sSf https://sh.rustup.rs/ | sh -s --default-toolchain=1.67.1 -- -y

ENV PATH="/root/.cargo/bin:${PATH}"

FROM rust

COPY Cargo.* /root/
COPY src /root/src

WORKDIR /root

RUN cargo build

ENTRYPOINT [ "/root/target/debug/cabin-temp-logger" ]

FROM rust:1.53

WORKDIR /usr/src/

ADD . .
RUN cargo install --path .

CMD ["minictrl"]
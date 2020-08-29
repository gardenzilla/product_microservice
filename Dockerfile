FROM ubuntu:latest
WORKDIR /usr/local/bin
COPY ./target/release/product_microservice /usr/local/bin/product_microservice
RUN apt-get update && apt-get install -y
RUN apt-get install curl -y
STOPSIGNAL SIGINT
ENTRYPOINT ["product_microservice"]
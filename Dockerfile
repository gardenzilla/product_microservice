FROM fedora:33
RUN dnf update -y && dnf clean all -y
WORKDIR /usr/local/bin
COPY ./target/release/product_microservice /usr/local/bin/product_microservice
STOPSIGNAL SIGINT
ENTRYPOINT ["product_microservice"]

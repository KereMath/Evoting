FROM debian:bookworm-slim

# Install dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    libgmp-dev \
    libssl-dev \
    flex \
    bison \
    wget \
    netcat-openbsd \
    curl \
    dos2unix \
    && rm -rf /var/lib/apt/lists/*

# Build and install PBC library
WORKDIR /tmp
RUN wget https://crypto.stanford.edu/pbc/files/pbc-0.5.14.tar.gz && \
    tar xzf pbc-0.5.14.tar.gz && \
    cd pbc-0.5.14 && \
    ./configure && \
    make && \
    make install && \
    ldconfig && \
    cd .. && \
    rm -rf pbc-0.5.14*

WORKDIR /app

COPY trustee-server.sh /app/
RUN dos2unix /app/trustee-server.sh && chmod +x /app/trustee-server.sh

CMD ["/app/trustee-server.sh"]

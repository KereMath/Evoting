FROM debian:bookworm-slim

# Install dependencies for PBC library
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

# Create a simple server script
COPY ttp-server.sh /app/
RUN dos2unix /app/ttp-server.sh && chmod +x /app/ttp-server.sh

EXPOSE 9000

CMD ["/app/ttp-server.sh"]

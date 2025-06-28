FROM postgres:15-alpine

# Install build dependencies
RUN apk add --no-cache --virtual .build-deps \
    git \
    make \
    gcc \
    g++ \
    clang15 \
    llvm15-dev

# Install pgvector
RUN cd /tmp && \
    git clone --branch v0.5.1 https://github.com/pgvector/pgvector.git && \
    cd pgvector && \
    make && \
    make install && \
    cd / && \
    rm -rf /tmp/pgvector && \
    apk del .build-deps

# Copy initialization scripts
COPY init-databases.sql /docker-entrypoint-initdb.d/
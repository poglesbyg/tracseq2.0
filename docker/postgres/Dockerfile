FROM postgres:15-alpine

# Install pgvector extension
# Using a simpler approach that doesn't require clang
RUN apk add --no-cache --virtual .build-deps \
    git \
    make \
    gcc \
    g++ \
    musl-dev && \
    cd /tmp && \
    git clone --branch v0.5.1 https://github.com/pgvector/pgvector.git && \
    cd pgvector && \
    # Disable JIT compilation to avoid clang dependency
    make OPTFLAGS="" && \
    make install && \
    cd / && \
    rm -rf /tmp/pgvector && \
    apk del .build-deps

# Copy initialization scripts
COPY init-databases.sql /docker-entrypoint-initdb.d/
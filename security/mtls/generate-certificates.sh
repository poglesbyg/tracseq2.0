#!/bin/bash

# TracSeq 2.0 - mTLS Certificate Generation Script
# Generates certificates for secure service-to-service communication

set -euo pipefail

# Configuration
CERT_DIR="./certificates"
CA_KEY="$CERT_DIR/ca.key"
CA_CERT="$CERT_DIR/ca.crt"
VALIDITY_DAYS=365

# Service list
SERVICES=(
    "auth-service"
    "sample-service"
    "enhanced-storage-service"
    "template-service"
    "sequencing-service"
    "notification-service"
    "enhanced-rag-service"
    "event-service"
    "transaction-service"
    "api-gateway"
)

# Create certificate directory
mkdir -p "$CERT_DIR"

# Generate CA private key
echo "🔐 Generating CA private key..."
openssl genrsa -out "$CA_KEY" 4096

# Generate CA certificate
echo "📜 Generating CA certificate..."
openssl req -new -x509 -days $VALIDITY_DAYS -key "$CA_KEY" -out "$CA_CERT" \
    -subj "/C=US/ST=State/L=City/O=TracSeq/OU=Security/CN=TracSeq-CA"

# Generate certificates for each service
for service in "${SERVICES[@]}"; do
    echo "🔧 Generating certificates for $service..."
    
    # Service private key
    openssl genrsa -out "$CERT_DIR/$service.key" 2048
    
    # Certificate signing request
    openssl req -new -key "$CERT_DIR/$service.key" \
        -out "$CERT_DIR/$service.csr" \
        -subj "/C=US/ST=State/L=City/O=TracSeq/OU=Microservices/CN=$service"
    
    # Sign the certificate
    openssl x509 -req -days $VALIDITY_DAYS \
        -in "$CERT_DIR/$service.csr" \
        -CA "$CA_CERT" -CAkey "$CA_KEY" -CAcreateserial \
        -out "$CERT_DIR/$service.crt"
    
    # Create PEM bundle
    cat "$CERT_DIR/$service.crt" "$CERT_DIR/$service.key" > "$CERT_DIR/$service.pem"
    
    # Clean up CSR
    rm "$CERT_DIR/$service.csr"
done

echo "✅ Certificate generation complete!"
echo "📁 Certificates stored in: $CERT_DIR"

# Create certificate verification script
cat > "$CERT_DIR/verify-certificates.sh" << 'EOF'
#!/bin/bash
# Verify all certificates
for cert in *.crt; do
    if [[ "$cert" != "ca.crt" ]]; then
        echo "Verifying $cert..."
        openssl verify -CAfile ca.crt "$cert"
    fi
done
EOF

chmod +x "$CERT_DIR/verify-certificates.sh"
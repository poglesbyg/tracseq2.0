#!/bin/bash
# Verify all certificates
for cert in *.crt; do
    if [[ "$cert" != "ca.crt" ]]; then
        echo "Verifying $cert..."
        openssl verify -CAfile ca.crt "$cert"
    fi
done

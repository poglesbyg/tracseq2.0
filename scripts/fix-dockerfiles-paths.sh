#!/bin/bash
# Fix Dockerfile paths after restructuring

echo "🔧 Fixing Dockerfile paths in lims-core services..."

# Find all Dockerfiles in lims-core
for dockerfile in lims-core/*/Dockerfile; do
    if [ -f "$dockerfile" ]; then
        service_dir=$(dirname "$dockerfile")
        service_name=$(basename "$service_dir")
        
        echo "  Fixing $service_name..."
        
        # Fix COPY commands that reference the service directory
        sed -i.bak -E "s|COPY $service_name/Cargo.toml|COPY Cargo.toml|g" "$dockerfile"
        sed -i.bak -E "s|COPY $service_name/src|COPY src|g" "$dockerfile"
        sed -i.bak -E "s|COPY $service_name/migrations|COPY migrations|g" "$dockerfile"
        
        # Clean up backup files
        rm -f "${dockerfile}.bak"
    fi
done

echo "✅ Dockerfile paths fixed!"
echo "🚀 You can now run: ./quick-start.sh" 
#!/bin/bash

# TracSeq 2.0 Individual Service Builder
# Usage: ./build-individual-service.sh <service-name> [options]

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Print functions
print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_header() {
    echo ""
    echo -e "${BLUE}================================================${NC}"
    echo -e "${BLUE} $1${NC}"
    echo -e "${BLUE}================================================${NC}"
    echo ""
}

# Available services
RUST_SERVICES=(
    "auth_service"
    "sample_service"
    "sequencing_service"
    "notification_service"
    "enhanced_storage_service"
    "template_service"
    "transaction_service"
    "event_service"
    "lab_manager"
    "library_details_service"
    "qaqc_service"
    "spreadsheet_versioning_service"
    "config-service"
)

PYTHON_SERVICES=(
    "enhanced_rag_service"
    "api_gateway"
    "lab_submission_rag"
)

ALL_SERVICES=("${RUST_SERVICES[@]}" "${PYTHON_SERVICES[@]}")

# Default values
SERVICE_NAME=""
BUILD_ARGS=""
NO_CACHE=false
VERBOSE=false
PUSH_IMAGE=false
IMAGE_TAG="latest"
DOCKERFILE="Dockerfile"

# Help function
show_help() {
    echo "TracSeq 2.0 Individual Service Builder"
    echo ""
    echo "Usage: $0 <service-name> [options]"
    echo ""
    echo "Available services:"
    echo "  Rust services:"
    for service in "${RUST_SERVICES[@]}"; do
        echo "    - $service"
    done
    echo ""
    echo "  Python services:"
    for service in "${PYTHON_SERVICES[@]}"; do
        echo "    - $service"
    done
    echo ""
    echo "Options:"
    echo "  -h, --help          Show this help message"
    echo "  -t, --tag TAG       Image tag (default: latest)"
    echo "  -f, --file FILE     Dockerfile name (default: Dockerfile)"
    echo "  --no-cache          Build without cache"
    echo "  --push              Push image to registry after build"
    echo "  --build-arg ARG     Pass build argument to Docker"
    echo "  -v, --verbose       Verbose output"
    echo ""
    echo "Examples:"
    echo "  $0 auth_service"
    echo "  $0 auth_service --tag v1.0.0"
    echo "  $0 enhanced_storage_service --no-cache"
    echo "  $0 api_gateway --build-arg ENV=production"
}

# Parse command line arguments
parse_args() {
    if [ $# -eq 0 ]; then
        show_help
        exit 1
    fi

    SERVICE_NAME="$1"
    shift

    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                show_help
                exit 0
                ;;
            -t|--tag)
                IMAGE_TAG="$2"
                shift 2
                ;;
            -f|--file)
                DOCKERFILE="$2"
                shift 2
                ;;
            --no-cache)
                NO_CACHE=true
                shift
                ;;
            --push)
                PUSH_IMAGE=true
                shift
                ;;
            --build-arg)
                BUILD_ARGS="$BUILD_ARGS --build-arg $2"
                shift 2
                ;;
            -v|--verbose)
                VERBOSE=true
                shift
                ;;
            *)
                print_error "Unknown option: $1"
                show_help
                exit 1
                ;;
        esac
    done
}

# Validate service name
validate_service() {
    local service_found=false
    for service in "${ALL_SERVICES[@]}"; do
        if [ "$service" = "$SERVICE_NAME" ]; then
            service_found=true
            break
        fi
    done

    if [ "$service_found" = false ]; then
        print_error "Service '$SERVICE_NAME' not found!"
        echo ""
        echo "Available services:"
        printf '%s\n' "${ALL_SERVICES[@]}"
        exit 1
    fi
}

# Check if service directory exists
check_service_directory() {
    if [ ! -d "$SERVICE_NAME" ]; then
        print_error "Service directory '$SERVICE_NAME' does not exist!"
        exit 1
    fi

    if [ ! -f "$SERVICE_NAME/$DOCKERFILE" ]; then
        print_error "Dockerfile not found at '$SERVICE_NAME/$DOCKERFILE'!"
        exit 1
    fi

    print_success "Service directory and Dockerfile found"
}

# Build the service
build_service() {
    local image_name="tracseq-$SERVICE_NAME:$IMAGE_TAG"
    local build_command="docker build"
    
    # Add build arguments
    if [ "$NO_CACHE" = true ]; then
        build_command="$build_command --no-cache"
    fi
    
    if [ "$VERBOSE" = true ]; then
        build_command="$build_command --progress=plain"
    fi
    
    # Add custom build args
    build_command="$build_command $BUILD_ARGS"
    
    # Add dockerfile and tag
    build_command="$build_command -f $SERVICE_NAME/$DOCKERFILE -t $image_name $SERVICE_NAME"
    
    print_info "Building image: $image_name"
    print_info "Build command: $build_command"
    
    if $build_command; then
        print_success "Successfully built $image_name"
    else
        print_error "Failed to build $image_name"
        exit 1
    fi
}

# Push image to registry
push_image() {
    if [ "$PUSH_IMAGE" = true ]; then
        local image_name="tracseq-$SERVICE_NAME:$IMAGE_TAG"
        print_info "Pushing image: $image_name"
        
        if docker push "$image_name"; then
            print_success "Successfully pushed $image_name"
        else
            print_error "Failed to push $image_name"
            exit 1
        fi
    fi
}

# Test the built image
test_image() {
    local image_name="tracseq-$SERVICE_NAME:$IMAGE_TAG"
    print_info "Testing built image..."
    
    # Check if image exists
    if docker image inspect "$image_name" >/dev/null 2>&1; then
        print_success "Image $image_name exists and is ready"
        
        # Show image details
        if [ "$VERBOSE" = true ]; then
            echo ""
            echo "Image details:"
            docker image inspect "$image_name" --format "table {{.Id}}\t{{.Size}}\t{{.Created}}"
        fi
    else
        print_error "Image $image_name was not created successfully"
        exit 1
    fi
}

# Show build summary
show_summary() {
    local image_name="tracseq-$SERVICE_NAME:$IMAGE_TAG"
    
    print_header "Build Summary"
    echo "Service: $SERVICE_NAME"
    echo "Image: $image_name"
    echo "Tag: $IMAGE_TAG"
    echo "Dockerfile: $SERVICE_NAME/$DOCKERFILE"
    echo "No Cache: $NO_CACHE"
    echo "Pushed: $PUSH_IMAGE"
    
    if [ "$VERBOSE" = true ]; then
        echo ""
        echo "Available images for this service:"
        docker images | grep "tracseq-$SERVICE_NAME" || echo "No other images found"
    fi
}

# Main execution
main() {
    print_header "TracSeq 2.0 Individual Service Builder"
    
    parse_args "$@"
    validate_service
    check_service_directory
    build_service
    test_image
    push_image
    show_summary
    
    print_success "Build completed successfully! 🎉"
}

# Execute main function with all arguments
main "$@" 

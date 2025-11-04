#!/bin/bash
# Hodei Verified Permissions - Development Startup Script
# This script provides an interactive way to start and manage the development environment

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Project root
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$PROJECT_ROOT"

# Help function
show_help() {
    cat << EOF
${CYAN}Hodei Verified Permissions - Development Tool${NC}

${YELLOW}Usage:${NC}
    $0 [COMMAND] [OPTIONS]

${YELLOW}Commands:${NC}
    ${GREEN}start${NC}          Start all services in development mode
    ${GREEN}server${NC}         Start only the gRPC server
    ${GREEN}web${NC}            Start only the web interface
    ${GREEN}test${NC}           Run all tests
    ${GREEN}test-unit${NC}      Run unit tests only
    ${GREEN}test-integration${NC} Run integration tests only
    ${GREEN}build${NC}          Build all components
    ${GREEN}clean${NC}          Clean build artifacts
    ${GREEN}format${NC}         Format code
    ${GREEN}lint${NC}           Run linters
    ${GREEN}logs${NC}           Show service logs
    ${GREEN}status${NC}         Show service status
    ${GREEN}stop${NC}           Stop all services
    ${GREEN}reset${NC}          Reset database (WARNING: Deletes all data)
    ${GREEN}grpc-test${NC}      Test gRPC connection
    ${GREEN}health${NC}         Check service health
    ${GREEN}postman${NC}        Open Postman setup guide
    ${GREEN}open${NC}           Open web interface in browser
    ${GREEN}example${NC}        Run example workflow
    ${GREEN}help${NC}           Show this help message

${YELLOW}Options:${NC}
    -h, --help         Show this help message
    -v, --verbose      Enable verbose output
    -d, --daemon       Run in daemon mode (background)

${YELLOW}Examples:${NC}
    $0 start           # Start all services
    $0 test            # Run all tests
    $0 build --verbose # Build with verbose output
    $0 logs -f         # Follow logs

${YELLOW}Environment Variables:${NC}
    DATABASE_URL       Database connection string (default: sqlite:///home/rubentxu/hodei-data/hodei.db)
    API_URL           Web interface URL (default: http://localhost:3000)
    GRPC_URL          gRPC server URL (default: localhost:50051)

EOF
}

# Check prerequisites
check_prerequisites() {
    echo -e "${BLUE}Checking prerequisites...${NC}"

    # Check Rust
    if ! command -v cargo &> /dev/null; then
        echo -e "${RED}❌ Rust/Cargo not found. Please install Rust:${NC}"
        echo "   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        exit 1
    fi

    # Check Node.js
    if ! command -v node &> /dev/null; then
        echo -e "${RED}❌ Node.js not found. Please install Node.js:${NC}"
        echo "   https://nodejs.org/"
        exit 1
    fi

    # Check npm
    if ! command -v npm &> /dev/null; then
        echo -e "${RED}❌ npm not found.${NC}"
        exit 1
    fi

    # Check grpcurl
    if ! command -v grpcurl &> /dev/null; then
        echo -e "${YELLOW}⚠️  grpcurl not found. Installing...${NC}"
        go install github.com/fullstorydev/grpcurl/cmd/grpcurl@latest
    fi

    echo -e "${GREEN}✅ All prerequisites met!${NC}"
}

# Initialize environment
init_environment() {
    echo -e "${BLUE}Initializing environment...${NC}"

    # Set default environment variables
    export DATABASE_URL="${DATABASE_URL:-sqlite:///home/rubentxu/hodei-data/hodei.db}"
    export API_URL="${API_URL:-http://localhost:3000}"
    export GRPC_URL="${GRPC_URL:-localhost:50051}"

    # Create data directory
    mkdir -p /home/rubentxu/hodei-data
    touch /home/rubentxu/hodei-data/hodei.db

    # Install Node.js dependencies
    if [ ! -d "web-nextjs/node_modules" ]; then
        echo -e "${YELLOW}Installing Node.js dependencies...${NC}"
        cd web-nextjs
        npm install
        cd ..
    fi

    echo -e "${GREEN}✅ Environment initialized!${NC}"
}

# Start services
start_services() {
    echo -e "${BLUE}Starting services...${NC}"

    # Start database initialization
    make db-init > /dev/null 2>&1

    # Start gRPC server in background
    echo -e "${YELLOW}Starting gRPC server on port 50051...${NC}"
    cd verified-permissions
    cargo build --bin hodei-verified-permissions > /dev/null 2>&1
    DATABASE_URL="$DATABASE_URL" cargo run --bin hodei-verified-permissions > /tmp/hodei-server.log 2>&1 &
    SERVER_PID=$!
    cd ..

    # Wait for server to start
    echo -e "${YELLOW}Waiting for server to start...${NC}"
    sleep 5

    # Test connection
    if grpcurl -plaintext localhost:50051 list &> /dev/null; then
        echo -e "${GREEN}✅ gRPC server is running!${NC}"
    else
        echo -e "${RED}❌ Failed to start gRPC server${NC}"
        kill $SERVER_PID 2>/dev/null || true
        exit 1
    fi

    # Start web interface
    echo -e "${YELLOW}Starting web interface on port 3000...${NC}"
    cd web-nextjs
    npm run dev > /tmp/hodei-web.log 2>&1 &
    WEB_PID=$!
    cd ..

    # Wait for web interface
    sleep 3

    # Save PIDs
    echo $SERVER_PID > /tmp/hodei-server.pid
    echo $WEB_PID > /tmp/hodei-web.pid

    echo ""
    echo -e "${GREEN}🎉 All services started successfully!${NC}"
    echo ""
    echo -e "${CYAN}Services:${NC}"
    echo -e "  ${YELLOW}gRPC API:${NC}      ${GRPC_URL}"
    echo -e "  ${YELLOW}Web Interface:${NC}  ${API_URL}"
    echo ""
    echo -e "${CYAN}Logs:${NC}"
    echo -e "  Server: ${YELLOW}tail -f /tmp/hodei-server.log${NC}"
    echo -e "  Web:    ${YELLOW}tail -f /tmp/hodei-web.log${NC}"
    echo ""
    echo -e "${CYAN}Useful Commands:${NC}"
    echo -e "  ${YELLOW}Open web interface:${NC} $0 open"
    echo -e "  ${YELLOW}View logs:${NC}        $0 logs"
    echo -e "  ${YELLOW}Test gRPC:${NC}        $0 grpc-test"
    echo -e "  ${YELLOW}Run example:${NC}      $0 example"
    echo -e "  ${YELLOW}Stop services:${NC}    $0 stop"
    echo ""
}

# Stop services
stop_services() {
    echo -e "${BLUE}Stopping services...${NC}"

    # Stop server
    if [ -f /tmp/hodei-server.pid ]; then
        SERVER_PID=$(cat /tmp/hodei-server.pid)
        if kill -0 $SERVER_PID 2>/dev/null; then
            kill $SERVER_PID
            echo -e "${GREEN}✅ gRPC server stopped${NC}"
        fi
        rm /tmp/hodei-server.pid
    fi

    # Stop web interface
    if [ -f /tmp/hodei-web.pid ]; then
        WEB_PID=$(cat /tmp/hodei-web.pid)
        if kill -0 $WEB_PID 2>/dev/null; then
            kill $WEB_PID
            echo -e "${GREEN}✅ Web interface stopped${NC}"
        fi
        rm /tmp/hodei-web.pid
    fi

    # Kill any remaining processes
    pkill -f "hodei-verified-permissions" 2>/dev/null || true
    pkill -f "nextjs" 2>/dev/null || true

    echo -e "${GREEN}✅ All services stopped${NC}"
}

# Show status
show_status() {
    echo -e "${CYAN}Service Status:${NC}"
    echo ""

    # Check server
    if grpcurl -plaintext localhost:50051 list &> /dev/null; then
        echo -e "  ${GREEN}✅${NC} gRPC Server (localhost:50051)"
    else
        echo -e "  ${RED}❌${NC} gRPC Server (localhost:50051)"
    fi

    # Check web interface
    if curl -s http://localhost:3000/api/health &> /dev/null; then
        echo -e "  ${GREEN}✅${NC} Web Interface (http://localhost:3000)"
    else
        echo -e "  ${RED}❌${NC} Web Interface (http://localhost:3000)"
    fi

    # Check database
    if [ -f /home/rubentxu/hodei-data/hodei.db ]; then
        echo -e "  ${GREEN}✅${NC} Database (/home/rubentxu/hodei-data/hodei.db)"
    else
        echo -e "  ${RED}❌${NC} Database"
    fi

    echo ""
    echo -e "${CYAN}Environment:${NC}"
    echo -e "  DATABASE_URL: ${YELLOW}${DATABASE_URL}${NC}"
    echo -e "  API_URL:      ${YELLOW}${API_URL}${NC}"
    echo -e "  GRPC_URL:     ${YELLOW}${GRPC_URL}${NC}"
}

# Run tests
run_tests() {
    echo -e "${BLUE}Running tests...${NC}"
    cd verified-permissions
    cargo test --lib
    cargo test --test '*'
    cd ..
    echo -e "${GREEN}✅ All tests passed!${NC}"
}

# Run example workflow
run_example() {
    echo -e "${BLUE}Running example workflow...${NC}"
    echo ""

    # Create policy store
    echo -e "${YELLOW}1. Creating policy store...${NC}"
    STORE_ID=$(grpcurl -plaintext -d '{
        "name": "Example Store",
        "description": "Example policy store for demonstration"
    }' localhost:50051 authorization.AuthorizationControl.CreatePolicyStore | jq -r '.policy_store_id')
    echo -e "   Created: ${GREEN}${STORE_ID}${NC}"
    echo ""

    # Add schema
    echo -e "${YELLOW}2. Adding schema...${NC}"
    grpcurl -plaintext -d "{
        \"policy_store_id\": \"${STORE_ID}\",
        \"schema\": \"{\\\"entities\\\":{\\\"User\\\":{\\\"shape\\\":{\\\"type\\\":\\\"record\\\",\\\"attributes\\\":{\\\"role\\\":{\\\"type\\\":\\\"String\\\"}}}}}}\"
    }" localhost:50051 authorization.AuthorizationControl.PutSchema
    echo -e "   ${GREEN}Schema added${NC}"
    echo ""

    # Create policy
    echo -e "${YELLOW}3. Creating policy...${NC}"
    grpcurl -plaintext -d "{
        \"policy_store_id\": \"${STORE_ID}\",
        \"policy_id\": \"example_policy\",
        \"statement\": \"permit(principal, action, resource) when { principal.role == \\\"admin\\\" };\",
        \"description\": \"Example admin access policy\"
    }" localhost:50051 authorization.AuthorizationControl.CreatePolicy
    echo -e "   ${GREEN}Policy created${NC}"
    echo ""

    # Test authorization
    echo -e "${YELLOW}4. Testing authorization...${NC}"
    grpcurl -plaintext -d "{
        \"policy_store_id\": \"${STORE_ID}\",
        \"principal\": \"User::\\\"alice\\\"\",
        \"action\": \"Action::\\\"viewDocument\\\"\",
        \"resource\": \"Document::\\\"doc123\\\",
        \"entities\": [
            {
                \"identifier\": \"User::\\\"alice\\\"\",
                \"attrs_json\": \"{\\\"role\\\":\\\"admin\\\"}\"
            }
        ]
    }" localhost:50051 authorization.AuthorizationData/IsAuthorized
    echo ""

    # Query audit log
    echo -e "${YELLOW}5. Querying audit log...${NC}"
    grpcurl -plaintext -d "{
        \"policy_store_id\": \"${STORE_ID}\",
        \"max_results\": 5
    }" localhost:50051 authorization.AuthorizationControl.GetPolicyStoreAuditLog | jq '.'
    echo ""

    echo -e "${GREEN}🎉 Example workflow completed!${NC}"
    echo ""
    echo -e "${CYAN}Next steps:${NC}"
    echo -e "  ${YELLOW}• Open web interface:${NC} $0 open"
    echo -e "  ${YELLOW}• View full API documentation:${NC} cat docs/API_DOCUMENTATION.md"
    echo -e "  ${YELLOW}• Import Postman collection:${NC} postman/VerifiedPermissions.postman_collection.json"
}

# Open web interface
open_web() {
    if command -v xdg-open &> /dev/null; then
        xdg-open http://localhost:3000
    elif command -v open &> /dev/null; then
        open http://localhost:3000
    else
        echo -e "${CYAN}Please open:${NC} http://localhost:3000"
    fi
}

# Show Postman guide
show_postman_guide() {
    cat << EOF
${CYAN}Postman gRPC Setup Guide${NC}

${YELLOW}Step 1: Install Postman${NC}
Download Postman v10+ from: https://www.postman.com/downloads/

${YELLOW}Step 2: Import Collection${NC}
1. Open Postman
2. Click "Import" button
3. Select "postman/VerifiedPermissions.postman_collection.json"
4. Click "Import"

${YAMEEN}Step 3: Configure Environment${NC}
1. Click the gear icon (⚙️) in the top right
2. Click "Add Environment"
3. Set name to: "Local Development"
4. Add variable:
   - Name: GRPC_URL
   - Value: localhost:50051
5. Click "Save"
6. Select the environment from the dropdown

${YELLOW}Step 4: Test Connection${NC}
1. Expand "Audit Trail" folder
2. Open "Get Policy Store Audit Log"
3. Click "Send" button
4. You should see a response

${YELLOW}Available Collections:${NC}
  • Policy Stores - CRUD operations
  • Schema - Schema management
  • Policies - Policy CRUD
  • Authorization - Authorization checks
  • Audit Trail - Audit log queries

${YELLOW}Tips:${NC}
  • Use double quotes in JSON (not single quotes)
  • Use escape sequences for quotes in strings
  • Check the response tab for results
  • Use the "Pre-request Script" tab for setup

EOF
}

# Main script
main() {
    # Parse arguments
    COMMAND="${1:-help}"
    shift || true

    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                show_help
                exit 0
                ;;
            -v|--verbose)
                set -x
                shift
                ;;
            -d|--daemon)
                DAEMON=true
                shift
                ;;
            *)
                echo -e "${RED}Unknown option: $1${NC}"
                show_help
                exit 1
                ;;
        esac
    done

    # Execute command
    case $COMMAND in
        start)
            check_prerequisites
            init_environment
            start_services
            ;;
        server)
            check_prerequisites
            init_environment
            echo -e "${BLUE}Starting gRPC server...${NC}"
            cd verified-permissions
            DATABASE_URL="$DATABASE_URL" cargo run --bin hodei-verified-permissions
            ;;
        web)
            check_prerequisites
            init_environment
            echo -e "${BLUE}Starting web interface...${NC}"
            cd web-nextjs
            npm run dev
            ;;
        test)
            check_prerequisites
            run_tests
            ;;
        test-unit)
            check_prerequisites
            echo -e "${BLUE}Running unit tests...${NC}"
            cd verified-permissions
            cargo test --lib
            ;;
        test-integration)
            check_prerequisites
            echo -e "${BLUE}Running integration tests...${NC}"
            cd verified-permissions
            cargo test --test '*'
            ;;
        build)
            echo -e "${BLUE}Building project...${NC}"
            make build
            ;;
        clean)
            echo -e "${BLUE}Cleaning build artifacts...${NC}"
            make clean
            rm -f /tmp/hodei-*.log /tmp/hodei-*.pid
            ;;
        format)
            echo -e "${BLUE}Formatting code...${NC}"
            make format
            ;;
        lint)
            echo -e "${BLUE}Running linters...${NC}"
            make lint
            ;;
        logs)
            if [ -f /tmp/hodei-server.log ]; then
                tail -f /tmp/hodei-server.log
            else
                echo -e "${YELLOW}No logs found. Start services first with: $0 start${NC}"
            fi
            ;;
        status)
            show_status
            ;;
        stop)
            stop_services
            ;;
        reset)
            echo -e "${RED}⚠️  WARNING: This will delete all data!${NC}"
            read -p "Are you sure? (y/N) " -n 1 -r
            echo
            if [[ $REPLY =~ ^[Yy]$ ]]; then
                make db-reset
            else
                echo -e "${YELLOW}Operation cancelled${NC}"
            fi
            ;;
        grpc-test)
            echo -e "${BLUE}Testing gRPC connection...${NC}"
            if grpcurl -plaintext localhost:50051 list; then
                echo -e "${GREEN}✅ gRPC server is reachable!${NC}"
            else
                echo -e "${RED}❌ Cannot connect to gRPC server${NC}"
                echo -e "${YELLOW}Start it with: $0 start${NC}"
            fi
            ;;
        health)
            echo -e "${BLUE}Checking service health...${NC}"
            show_status
            ;;
        postman)
            show_postman_guide
            ;;
        open)
            open_web
            ;;
        example)
            check_prerequisites
            if ! grpcurl -plaintext localhost:50051 list &> /dev/null; then
                echo -e "${YELLOW}Server not running. Starting services...${NC}"
                start_services
                sleep 5
            fi
            run_example
            ;;
        help)
            show_help
            ;;
        *)
            echo -e "${RED}Unknown command: $COMMAND${NC}"
            show_help
            exit 1
            ;;
    esac
}

# Run main function
main "$@"

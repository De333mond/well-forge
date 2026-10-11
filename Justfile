default:
    @just --list

# Run the desktop app in development mode.
run:
    npm run tauri -- dev

# Build the desktop app and its installers.
build:
    npm run tauri -- build

# Build a Linux AppImage in Docker and copy it to build/linux/well-forge.AppImage.
docker-build:
    docker build --file Dockerfile.desktop --output type=local,dest=./build/linux .

# Run only the frontend in a browser.
frontend-run:
    npm run dev

# Type-check and build only the frontend.
frontend-build:
    npm run build

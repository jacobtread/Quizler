# Move to frontend
cd frontend

# Build the frontend
yarn dist

# Move to backend
cd ../backend

# Build backend in release
cargo build --release
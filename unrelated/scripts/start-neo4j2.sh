#!/bin/bash

# A simple bash script to start a Neo4j Community Edition Docker container.
# It sets up a persistent data volume and disables authentication for easy access.

# --- Configuration ---

# Set a name for the Docker container.
IMAGE_NAME="docker.io/neo4j:5.26.12-community"
CONTAINER_NAME="neo4j-db"

# Set the path for the persistent data volume on your local machine.
# This directory will be created if it doesn't exist.
NEO4J_DATA_DIR="$HOME/neo4j/data"

# --- Script Logic ---

# Check if Docker is installed.
if ! command -v docker &> /dev/null
then
    echo "Docker is not installed. Please install Docker to run this script."
    exit 1
fi

# Check if the Neo4j data directory exists and create it if not.
if [ ! -d "$NEO4J_DATA_DIR" ]; then
    echo "Creating Neo4j data directory at: $NEO4J_DATA_DIR"
    mkdir -p "$NEO4J_DATA_DIR"
    # Set permissions to ensure Neo4j user in container can write to the directory.
    # The default Neo4j user has uid 7474.
    chmod -R 777 "$NEO4J_DATA_DIR"
fi

# Stop and remove any pre-existing container with the same name.
# This ensures a clean start and avoids conflicts.
echo "Checking for and removing any existing container named '$CONTAINER_NAME'..."
docker rm -f "$CONTAINER_NAME" &> /dev/null

# Pull the latest Neo4j Community Edition image.
echo "Pulling the latest Neo4j Community Edition image from Docker Hub..."
docker pull $IMAGE_NAME

# Run the Neo4j container with authentication disabled.
echo "Starting the Neo4j container with authentication disabled..."
docker run \
    --detach \
    --name "$CONTAINER_NAME" \
    --publish=7474:7474 --publish=7687:7687 \
    --env=NEO4J_AUTH=none \
    --volume="$NEO4J_DATA_DIR":/data \
    $IMAGE_NAME

echo "Neo4j container started successfully!"
echo "---------------------------------------------------"
echo "You can access the Neo4j Browser at http://localhost:7474"
echo "Note: Authentication is disabled."
echo "The database data is being persisted in the following local directory:"
echo "$NEO4J_DATA_DIR"
echo "---------------------------------------------------"

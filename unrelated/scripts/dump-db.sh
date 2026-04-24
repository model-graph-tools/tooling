#!/usr/bin/env bash
set -euo pipefail

# Configuration
CONTAINER_NAME=my-neo4j
DB_NAME=neo4j
DUMP_FILE=neo4j.dump
DOCKERFILE=Dockerfile.neo4j
IMAGE_NAME=my-neo4j-image:latest

# Step 1: Dump the database from the running container
echo "==> Dumping database '$DB_NAME' from container '$CONTAINER_NAME'..."
docker exec -t "$CONTAINER_NAME" \
  bin/neo4j-admin database dump "$DB_NAME" --to=/var/lib/neo4j/backups

# Copy the dump file to host
echo "==> Copying dump file to host..."
docker cp "$CONTAINER_NAME:/var/lib/neo4j/backups/$DB_NAME.dump" "./$DUMP_FILE"

# Step 2: Create a Dockerfile for the new image
echo "==> Creating Dockerfile..."
cat > "$DOCKERFILE" <<'EOF'
FROM neo4j:5.23
ENV NEO4J_AUTH=none

# Copy dump into container
COPY neo4j.dump /var/lib/neo4j/import/

# Load database into $DB_NAME on startup
RUN bin/neo4j-admin database load neo4j --from-path=/var/lib/neo4j/import --overwrite-destination
EOF

# Step 3: Build the new image
echo "==> Building image '$IMAGE_NAME'..."
docker build -f "$DOCKERFILE" -t "$IMAGE_NAME" .

# Step 4: Cleanup temporary files
echo "==> Cleaning up..."
rm -f "$DUMP_FILE" "$DOCKERFILE"

echo "==> Done. You can run the new image with:"
echo "    docker run -it --rm -p 7474:7474 -p 7687:7687 $IMAGE_NAME"

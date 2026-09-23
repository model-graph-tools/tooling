# Data Image Design

Decouple Neo4J database files from the model image by introducing a separate OCI "data" image. This eliminates the VOLUME-related `podman cp` failure in repack and establishes a clean data-source-of-truth for both analyze and repack pipelines.

## Problem

The Neo4J base image declares `VOLUME /data`. When repack creates a container from the model image and tries to `podman cp /data/databases` out, the volume mount shadows the image layer data, producing an empty directory. Starting the container works around this but is wasteful — it boots Neo4J, nginx, and the REST API just to extract files.

## Solution

Introduce a data image (`quay.io/modelgraphtools/data:<tag>`) that stores only the `databases/` and `transactions/` directories. The analyze pipeline produces this image after running the analyzer. Both analyze and repack then build the model image using a multi-stage Dockerfile that copies data from the data image via `COPY --from`.

## Data Image

- **Repository:** `quay.io/modelgraphtools/data`
- **Tags:** Same scheme as model images — `41.0.1`, `ai-0.9.1`, etc.
- **Base:** `FROM scratch` (pure filesystem, no runtime)
- **Architecture:** Single-arch (`linux/amd64`). The content is platform-independent (Neo4J database files, no binaries), so no multi-arch manifest is needed.
- **Dockerfile:**
  ```dockerfile
  FROM scratch
  COPY databases /databases
  COPY transactions /transactions
  ```

## Model Image Dockerfile

The model image Dockerfile gains a multi-stage `FROM` and replaces local `COPY` with `COPY --from`:

```dockerfile
FROM quay.io/modelgraphtools/data:<tag> AS data
FROM neo4j:<version>
ARG TARGETARCH

# ... nginx, REST API, welcome page setup (unchanged) ...

USER neo4j
COPY --from=data /databases /data/databases
COPY --from=data /transactions /data/transactions
ENV NEO4J_AUTH=none
ENV NEO4J_server_databases_default__to__read__only=true
ENV NEO4J_server_http_listen__address=:7475
ENTRYPOINT ["/entrypoint.sh"]
```

The model image remains fully self-contained. The data exists in both images, but the data image is the canonical source.

## Analyze Pipeline Changes

Current flow in `cleanup.rs::build_neo4j_image`:
1. Stop Neo4J container
2. `podman cp` databases and transactions from stopped container
3. Build model image

New flow:
1. Stop Neo4J container
2. `podman cp` databases and transactions from stopped container to temp dir
3. **Build data image** from temp dir, tag as `quay.io/modelgraphtools/data:<tag>`
4. Build model image using multi-stage Dockerfile referencing the data image

Step 2 still uses `podman cp` from the stopped analyze container. This works because the analyze pipeline mounts a named volume (`--volume <name>:/data`) that persists the data written by Neo4J during analysis. This is distinct from the repack scenario where `podman create` from an image with `VOLUME /data` produced an empty anonymous volume.

## Repack Changes

Current flow:
1. Pull model image
2. `podman run -d` to start container
3. `podman cp` databases and transactions from running container
4. Build new model image
5. Stop and remove container

New flow:
1. Pull data image (`quay.io/modelgraphtools/data:<tag>`)
2. Build model image using multi-stage Dockerfile with updated REST API version

No container lifecycle at all. The entire `podman create`/`run`/`stop`/`rm` logic is removed.

## Push Changes

`mgt push <identifier>` pushes both images in parallel:
1. Push `quay.io/modelgraphtools/data:<tag>`
2. Push `quay.io/modelgraphtools/model:<tag>`

## Code Changes

### `constants.rs`
- Add `DATA_REPOSITORY` constant (`quay.io/modelgraphtools/data`)

### `neo4j.rs`
- Add `Neo4JImage::data_image_tag()` — returns the data image tag for this source
- Add `Neo4JImage::build_data_image(temp_dir, progress)` — builds and tags the data image from a directory containing `databases/` and `transactions/`
- Change `Neo4JImage::build_image(rest_api_version, progress)` — remove the `container_name` parameter; the Dockerfile now uses `COPY --from` referencing the data image tag
- Update `model_db_dockerfile(source_name, rest_api_version, data_image_tag)` — add `data_image_tag` parameter for the multi-stage `FROM` line
- Remove `copy_from_container` helper (no longer needed)

### `command/analyze/cleanup.rs`
- In `build_neo4j_image`: after stopping Neo4J, `podman cp` to temp dir, call `build_data_image`, then call the updated `build_image`

### `command/repack.rs`
- Replace container lifecycle with: pull data image, call `build_image`
- Remove `stop_container`/`remove_container` imports and logic
- The `repack_image` function becomes significantly simpler

### `command/push.rs`
- Push data image alongside model image for each identifier
- Both pushed in parallel within each batch

## Testing

- Update existing `model_db_dockerfile` tests to verify the multi-stage `FROM` line and `COPY --from=data` directives
- Add test for `data_image_tag()` returning correct tags
- Manual verification: `mgt analyze`, `mgt push`, `mgt repack` end-to-end

//! Analyzer version and download URL.

/// Container image registry and organization prefix.
pub static MODEL_GRAPH_TOOLS_REPOSITORY: &str = "quay.io/modelgraphtools/model";

/// A static string that defines the current version of the analyzer.
pub static ANALYZER_VERSION: &str = "0.1.2";

/// Returns the GitHub release download URL for the current analyzer version.
pub fn analyzer_url() -> String {
    format!(
        "https://github.com/model-graph-tools/analyzer/releases/download/v{v}/analyzer-{v}.jar",
        v = ANALYZER_VERSION
    )
}

/// Neo4J Docker image version tag used for the base image.
pub static NEO4J_VERSION: &str = "5.26-community";

/// Neo4J Docker image repository.
pub static NEO4J_IMAGE: &str = "docker.io/neo4j";

/// Target platforms for multi-arch image builds.
pub static PLATFORMS: &str = "linux/amd64,linux/arm64";

/// URL for the welcome page shown in the Neo4J browser after connecting.
pub static WELCOME_URL: &str = "https://model-graph-tools.github.io/assets/welcome.html";

/// URL for the schema SVG graphic referenced by the welcome page.
pub static SCHEMA_SVG_URL: &str = "https://model-graph-tools.github.io/assets/schema.svg";

/// Returns a Dockerfile for building a Neo4J image with pre-populated databases
/// and an nginx reverse proxy that serves the welcome page from the same origin.
pub fn model_db_dockerfile(source_name: &str) -> String {
    format!(
        r#"FROM neo4j:{NEO4J_VERSION}

USER root
RUN apt-get update && apt-get install -y --no-install-recommends nginx curl \
    && rm -rf /var/lib/apt/lists/*

RUN mkdir -p /var/www/html /var/lib/nginx/body /var/lib/nginx/proxy \
        /var/lib/nginx/fastcgi /var/lib/nginx/uwsgi /var/lib/nginx/scgi \
        /var/log/nginx /run \
    && chown -R neo4j:neo4j /var/www/html /var/lib/nginx /var/log/nginx /run \
    && curl -fsSL -o /var/www/html/welcome.html \
       {WELCOME_URL} \
    && curl -fsSL -o /var/www/html/schema.svg \
       {SCHEMA_SVG_URL} \
    && sed -i 's|{{{{SOURCE_NAME}}}}|{source_name}|g' /var/www/html/welcome.html

RUN printf 'pid /run/nginx.pid;\n\
error_log /var/log/nginx/error.log;\n\
events {{\n\
    worker_connections 64;\n\
}}\n\
http {{\n\
    include /etc/nginx/mime.types;\n\
    access_log /var/log/nginx/access.log;\n\
    client_body_temp_path /var/lib/nginx/body;\n\
    proxy_temp_path /var/lib/nginx/proxy;\n\
    server {{\n\
        listen 7474;\n\
        location = /welcome.html {{\n\
            root /var/www/html;\n\
        }}\n\
        location = /schema.svg {{\n\
            root /var/www/html;\n\
        }}\n\
        location / {{\n\
            proxy_pass http://localhost:7475;\n\
            proxy_http_version 1.1;\n\
            proxy_set_header Upgrade $http_upgrade;\n\
            proxy_set_header Connection "upgrade";\n\
            proxy_set_header Host $host;\n\
        }}\n\
    }}\n\
}}\n' > /etc/nginx/nginx.conf

RUN printf '#!/bin/bash\nnginx -c /etc/nginx/nginx.conf\nexec /startup/docker-entrypoint.sh neo4j\n' \
    > /entrypoint.sh && chmod +x /entrypoint.sh

USER neo4j
COPY --chown=neo4j:neo4j databases /data/databases
COPY --chown=neo4j:neo4j transactions /data/transactions
ENV NEO4J_AUTH=none
ENV NEO4J_server_databases_default__to__read__only=true
ENV NEO4J_server_http_listen__address=:7475
ENTRYPOINT ["/entrypoint.sh"]
"#
    )
}

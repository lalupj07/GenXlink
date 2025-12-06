# GenXLink Dockerfile v1.0.0
FROM debian:bullseye-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y ca-certificates curl && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -r -s /bin/false genxlink

# Copy binaries
COPY api-server.exe /usr/local/bin/
COPY signaling-server.exe /usr/local/bin/

# Create startup script
RUN echo '#!/bin/bash' > /usr/local/bin/genxlink-start && \
    echo 'api-server.exe &' >> /usr/local/bin/genxlink-start && \
    echo 'signaling-server.exe &' >> /usr/local/bin/genxlink-start && \
    echo 'wait' >> /usr/local/bin/genxlink-start && \
    chmod +x /usr/local/bin/genxlink-start

USER genxlink
EXPOSE 8000 8081
ENTRYPOINT ["/usr/local/bin/genxlink-start"]

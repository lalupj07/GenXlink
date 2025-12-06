# TURN Server for GenXLink - NAT Traversal
FROM coturn/coturn:latest

# Copy configuration
COPY turnserver.conf /etc/coturn/turnserver.conf

EXPOSE 3478/udp
EXPOSE 3478/tcp
EXPOSE 5349/tcp
EXPOSE 49152-65535/udp

CMD ["turnserver", "-c", "/etc/coturn/turnserver.conf"]

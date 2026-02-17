# 1f2a234458cb85b08f51a9383dd5eacc783ae094

# Stage 1: Build
FROM node:18-slim AS builder

RUN corepack enable && corepack prepare pnpm@10.28.2 --activate

WORKDIR /app

# Copy workspace config and lockfile first for better caching
COPY pnpm-workspace.yaml package.json pnpm-lock.yaml ./
COPY packages/common/package.json packages/common/
COPY packages/server/package.json packages/server/
COPY packages/plugin/package.json packages/plugin/

RUN pnpm install --frozen-lockfile

# Copy source code
COPY packages/common/ packages/common/
COPY packages/server/ packages/server/
COPY packages/plugin/ packages/plugin/

# Build all packages (common -> server + plugin)
ENV MULTI_USER_MODE=true
RUN pnpm run build:multi-user

# Stage 2: Runtime
FROM node:18-slim AS runtime

RUN corepack enable && corepack prepare pnpm@10.28.2 --activate

WORKDIR /app

# Copy workspace config and install production deps
COPY pnpm-workspace.yaml package.json pnpm-lock.yaml ./
COPY packages/common/package.json packages/common/
COPY packages/server/package.json packages/server/
COPY packages/plugin/package.json packages/plugin/

RUN pnpm install --frozen-lockfile --prod

# Copy built artifacts
COPY --from=builder /app/packages/common/dist/ packages/common/dist/
COPY --from=builder /app/packages/server/dist/ packages/server/dist/
COPY --from=builder /app/packages/plugin/dist/ packages/plugin/dist/
COPY --from=builder /app/packages/plugin/index.html packages/plugin/index.html

# MCP Server HTTP (SSE)
EXPOSE 4401
# MCP WebSocket
EXPOSE 4402
# REPL
EXPOSE 4403
# Plugin UI
EXPOSE 4400

ENV PENPOT_MCP_SERVER_LISTEN_ADDRESS=0.0.0.0
ENV PENPOT_MCP_SERVER_ADDRESS=0.0.0.0

# Use a simple entrypoint script to run both servers
COPY <<'EOF' /app/start.sh
#!/bin/sh
set -e

# Start MCP server in background
node packages/server/dist/index.js --multi-user &
MCP_PID=$!

# Serve plugin static files with a simple HTTP server
npx --yes serve packages/plugin/dist -l 4400 --cors --no-clipboard &
PLUGIN_PID=$!

# Handle shutdown signals
trap "kill $MCP_PID $PLUGIN_PID 2>/dev/null; exit 0" SIGTERM SIGINT

# Wait for either process to exit
wait -n $MCP_PID $PLUGIN_PID 2>/dev/null || true
kill $MCP_PID $PLUGIN_PID 2>/dev/null
exit 1
EOF

RUN chmod +x /app/start.sh

CMD ["/app/start.sh"]

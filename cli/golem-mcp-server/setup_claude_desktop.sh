#!/bin/bash
set -e

echo "🔧 Setting up Claude Desktop integration"

# Find Claude Desktop config location
if [[ "$OSTYPE" == "darwin"* ]]; then
    CONFIG_DIR="$HOME/Library/Application Support/Claude"
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    CONFIG_DIR="$HOME/.config/claude"
else
    echo "❌ Unsupported OS. Please manually configure Claude Desktop."
    exit 1
fi

CONFIG_FILE="$CONFIG_DIR/claude_desktop_config.json"
mkdir -p "$CONFIG_DIR"

# Build the MCP server binary
echo "📦 Building Golem MCP Server..."
cargo build --release --bin golem-mcp-server

BINARY_PATH="$(pwd)/target/release/golem-mcp-server"
GOLEM_CLI_PATH=$(which golem-cli || echo "/usr/local/bin/golem-cli")

# Create or update Claude Desktop config
echo "📝 Updating Claude Desktop config at: $CONFIG_FILE"

cat > "$CONFIG_FILE" << EOF
{
  "mcpServers": {
    "golem-cli": {
      "command": "$BINARY_PATH",
      "args": ["--transport", "stdio"],
      "env": {
        "RUST_LOG": "info",
        "GOLEM_CLI_PATH": "$GOLEM_CLI_PATH"
      }
    }
  }
}
EOF

echo "✅ Claude Desktop configured!"
echo "📋 Config location: $CONFIG_FILE"
echo "🔄 Restart Claude Desktop to load the new MCP server"
echo ""
echo "🎯 You can now use Claude Desktop with commands like:"
echo "   - 'List my Golem component templates'"
echo "   - 'Create a new Rust component for payment processing'"
echo "   - 'Deploy my application to production'"
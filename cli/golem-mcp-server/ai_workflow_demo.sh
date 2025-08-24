#!/bin/bash
BASE_URL="http://127.0.0.1:8080"

echo "🤖 Demonstrating AI workflow with Golem MCP tools"

# Workflow 1: Project scaffolding
echo -e "\n1️⃣ AI Workflow: Create new component"
echo "This is what Claude Code would do when you say:"
echo "'Create a new Rust component for payment processing'"

echo -e "\nStep 1: List available templates..."
curl -s -X POST "$BASE_URL/mcp" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0", "id": "w1",
    "method": "tools/call",
    "params": {
      "name": "golem_project_management",
      "arguments": {"operation": "list_templates"}
    }
  }' | jq -r '.result.content[0].text // "Template listing result"'

echo -e "\nStep 2: Create new component..."
curl -s -X POST "$BASE_URL/mcp" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0", "id": "w2", 
    "method": "tools/call",
    "params": {
      "name": "golem_project_management",
      "arguments": {
        "operation": "new",
        "project_name": "payment-processor",
        "template_name": "rust"
      }
    }
  }' | jq -r '.result.content[0].text // "Component creation result"'

# Workflow 2: Build and deploy
echo -e "\n2️⃣ AI Workflow: Build and deploy"
echo "This is what Claude Code would do when you say:"
echo "'Build my payment component and deploy it'"

echo -e "\nStep 1: Build component..."
curl -s -X POST "$BASE_URL/mcp" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0", "id": "w3",
    "method": "tools/call", 
    "params": {
      "name": "golem_component_management",
      "arguments": {
        "operation": "build",
        "component_name": "payment-processor"
      }
    }
  }' | jq -r '.result.content[0].text // "Build result"'

echo -e "\nStep 2: Deploy application..."
curl -s -X POST "$BASE_URL/mcp" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0", "id": "w4",
    "method": "tools/call",
    "params": {
      "name": "golem_application_management", 
      "arguments": {
        "operation": "deploy",
        "app_name": "payment-processor"
      }
    }
  }' | jq -r '.result.content[0].text // "Deployment result"'

echo -e "\n✅ AI workflow simulation complete!"
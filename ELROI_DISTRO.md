# ElRoi Custom Distribution

This fork is an ElRoi-branded distribution of the open-source goose project.

Keep the upstream Apache 2.0 license, attribution, and notices intact. ElRoi
branding changes should stay focused on packaging metadata, visible UI copy,
icons, default prompts, and bundled extensions so the fork can continue to pull
upstream goose updates without a large merge burden.

## Local RAG Extension

The desktop app bundles an enabled stdio MCP extension named `rag`:

```yaml
cmd: /opt/anaconda3/envs/rag/bin/python
args:
  - /Users/ash/Development/RAG/mcp_server.py
```

The server must keep stdout reserved for MCP protocol messages. Human-readable
startup logging belongs on stderr.

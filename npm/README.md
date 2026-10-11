# LLM Pulse CLI

`llmpulse` is the official command-line client for the [LLM Pulse API](https://llmpulse.ai/api-docs): visibility metrics, mentions, citations, sentiment, AI traffic, Search Console, recommendations and webhooks from a terminal or CI job.

This package installs the native binary for your platform (macOS on Apple silicon and Intel, Linux x86-64 with glibc, Windows x86-64).

```bash
npm install -g @llmpulse/cli
# or
pnpm add -g @llmpulse/cli
# or run it once
npx @llmpulse/cli --help
```

Get an API key from the [LLM Pulse dashboard](https://llmpulse.ai/app/api_keys), then:

```bash
llmpulse login
llmpulse status
llmpulse metrics summary --range 28
```

In CI, set `LLMPULSE_API_KEY` (and optionally `LLMPULSE_PROJECT_ID`) instead of running `login`.

Full documentation, archives for other installs and the source: [github.com/LLM-Pulse/llmpulse-cli](https://github.com/LLM-Pulse/llmpulse-cli).

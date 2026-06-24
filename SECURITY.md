# Security Policy

## Supported versions

| Version | Supported |
|---------|-----------|
| latest release | yes |
| main branch | best effort |

## Reporting a vulnerability

Prompt Saviour is designed to be **local-only** - it does not send prompts or keystrokes over the network.

If you find a security issue (local privilege escalation, path traversal in storage, etc.):

1. **Do not** open a public issue for exploitable bugs.
2. Email the maintainer via GitHub (profile contact) or open a private security advisory on the repository.
3. Include steps to reproduce and affected version.

We aim to acknowledge reports within 7 days.

## Scope

In scope:

- Unauthorized file access outside `PROMPT_SAVIOUR_HOME`
- Command injection via CLI or inject channel
- Sandbox escapes in the Tauri shell

Out of scope:

- OS-level permission prompts the user explicitly grants
- Third-party agent apps reading their own clipboard

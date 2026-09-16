# Security policy

Grannus is pre-alpha and has no stable release channel yet.

Do not open a public issue for a vulnerability that could enable unauthorized
input, session takeover, secret disclosure, remote code execution, or unsafe
network exposure. Use GitHub's private vulnerability reporting for
`msork/grannus` when enabled. If it is not enabled, contact the repository owner
privately through the address listed on their GitHub profile.

Include affected commit/version, environment, impact, reproduction steps, and a
minimal proof of concept. Do not include real pairing secrets or third-party
personal data. Maintainers will acknowledge receipt, assess severity, prepare a
fix, and coordinate disclosure when a supported release exists.

Security expectations are detailed in `docs/threat-model.md`. Internet-facing
deployment is unsupported until the relevant roadmap milestone is complete.

# Security policy

Clovercut is in early development. Security fixes target the current code and
next release; older versions have no separate maintenance branch.

Report vulnerabilities through
[Clovercut's private vulnerability reporting](https://github.com/tulcoturo/clovercut/security/advisories/new)
when available. Include the Clovercut version, Linux distribution, reproduction
steps and a minimal sample. Avoid posting private projects, credentials or
exploitable details in public issues.

Relevant reports include unsafe handling of projects or media, writes outside
intended directories, failed model integrity checks and authentication bypasses
in the optional local API. The API is disabled by default and uses a token.

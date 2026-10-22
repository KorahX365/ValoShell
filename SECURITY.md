# Security

## Reporting a vulnerability

Please don't open a public issue. Report it privately through
[a security advisory](https://github.com/Soulhackzlol/ValoShell/security/advisories/new) with
the steps to reproduce and what an attacker could do.

You'll get a reply within a week, and a fix or a plan before anything is made public.

## Supported versions

Only the latest release gets fixes while ValoShell is in early preview.

## Scope

ValoShell reads the Riot Client's local credentials to talk to its API. Anything that could leak
those credentials, the session token or account data off the machine is in scope.

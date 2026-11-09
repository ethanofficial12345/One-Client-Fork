---
default: minor
---

# Allow offline accounts without a Microsoft account

Upstream 2.10.0 added offline accounts but gated creating and launching them on
having at least one Microsoft account signed in. This fork removes that gate so
a local username can be used on its own:

- Offline accounts can be added with no Microsoft account present.
- An offline account is a valid default account on its own.
- Launching an offline account no longer requires a Microsoft account; it
  passes `--userType legacy` and the vanilla dummy access token `0`.
- Onboarding offers the "Play offline" card again.

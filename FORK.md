# This is a fork

This repository is a modified build of **[Polyfrost/OneLauncher](https://github.com/Polyfrost/OneLauncher)**, version **2.10.0**.

It is **not** an official Polyfrost release and carries no warranty from the upstream project.

## License

This code is licensed under the same terms as upstream: **GPL-3.0-only**. See [`LICENSE`](./LICENSE) and [`ATTRIBUTION.md`](./ATTRIBUTION.md) for the full text and the list of third-party notices.

## What is different

Upstream 2.10.0 already ships offline accounts, but only lets you create or
launch one when at least one Microsoft account is signed in. This fork removes
that restriction:

- The Microsoft sign-in button is retained, so premium accounts still work.
- Offline accounts are stored locally in `auth.json` and persist across launches.
- An offline username can be added with no Microsoft account present.
- An offline account can be the active/default account on its own.
- Launching an offline account passes `--userType legacy` and the vanilla dummy
  access token `0` to the game, exactly as vanilla does for offline players.
- A UUID is derived for each username using the same MD5-v3 algorithm vanilla
  uses (`OfflinePlayer:<name>`), so the server assigns the identical UUID.
- Onboarding exposes an inline-validated "Play offline" card, and Settings →
  Accounts shows an offline badge plus add / set-default / remove.

Poly+ / wardrobe features still require a Microsoft account, matching upstream.

See the `.changeset/add-offline-accounts.md` change file for the full scope.

## Building locally

```sh
# Debug
SQLX_OFFLINE=true cargo run -p oneclient_app

# Release + package (host formats only)
SQLX_OFFLINE=true cargo build --release -p oneclient_app
cargo install cargo-packager --locked
cargo packager --release -p oneclient_app --formats nsis
```

See [`.github/workflows/build-local.yml`](./.github/workflows/build-local.yml) for the CI equivalent.

## Syncing from upstream

```sh
git remote add upstream https://github.com/Polyfrost/OneLauncher.git
git fetch upstream
git rebase upstream/main offline-accounts-2.10
```

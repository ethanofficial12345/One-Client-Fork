# This is a fork

This repository is a modified build of **[Polyfrost/OneLauncher](https://github.com/Polyfrost/OneLauncher)**, version **2.9.0**.

It is **not** an official Polyfrost release and carries no warranty from the upstream project.

## License

This code is licensed under the same terms as upstream: **GPL-3.0-only**. See [`LICENSE`](./LICENSE) and [`ATTRIBUTION.md`](./ATTRIBUTION.md) for the full text and the list of third-party notices.

## What is different

This fork adds **offline account support**: the ability to create, rename, and launch with a local username without a Microsoft account.

- The Microsoft sign-in button is retained, so premium accounts still work.
- Offline accounts are stored locally in `auth.json` and persist across launches.
- Launching an offline account passes `--userType legacy` and a dummy access token to the game, exactly as vanilla does for offline players.
- A UUID is derived for each username using the same MD5-v3 algorithm vanilla uses (`OfflinePlayer:<name>`), so the server assigns the identical UUID.
- Onboarding exposes an inline-validated "Play offline" card, and Settings → Accounts shows an offline badge plus add / set-default / rename / delete.

See the `.changeset/add-offline-accounts.md` change file for the full scope.

## Building locally

```sh
# Debug
SQLX_OFFLINE=true cargo run -p oneclient_app

# Release + package (host formats only)
SQLX_OFFLINE=true cargo build --release -p oneclient_app
cargo install cargo-packager --locked
cargo packager --release -p oneclient_app --formats nsis,deb,appimage
```

See [`.github/workflows/build-local.yml`](./.github/workflows/build-local.yml) for the CI equivalent.

## Syncing from upstream

```sh
git remote add upstream https://github.com/Polyfrost/OneLauncher.git
git fetch upstream
git rebase upstream/main offline-accounts
```
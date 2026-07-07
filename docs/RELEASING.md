# Releasing

This repository publishes the `solana-primitives` crate through crates.io
trusted publishing from `.github/workflows/publish.yml`.

## Publishing a version

1. Update `workspace.package.version` in `Cargo.toml`.
2. Run `cargo update --workspace` if `Cargo.lock` needs the crate version update.
3. Run `cargo fmt --all`, `just lint`, and `just test`.
4. Commit the version change.
5. Create and publish a GitHub Release for tag `vX.Y.Z`, or run the `Publish`
   workflow manually from GitHub Actions.

The publish workflow verifies the package with `cargo publish --dry-run --locked`
before requesting a short-lived crates.io token through OIDC.

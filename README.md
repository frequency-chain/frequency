# CI Base Docker Image Builder

A branch specifically for building the CI Docker image and publishing it to the GitHub Container Registry

## Updating Rust Version

- Go to [.github/workflows/publish-dev-ci-base-image.yml](.github/workflows/publish-dev-ci-base-image.yml)
  - Update `RUST_VERSION` env var
  - Bump `IMAGE_VERSION` env var
- Go to [./ci-base-image.dockerfile](./ci-base-image.dockerfile)
  - Update the `toolchain` references
- Push
- Wait for the Action to publish the new version
- Update the CI image used in the main branch

---
title: Release packaging
description: Shared packaging automation and application-specific release definitions.
nav_order: 20
---

Fastpotify keeps release asset definitions, nFPM configuration and native AUR and
Homebrew templates in `native-packages.yaml` and `packaging/`. Common automation comes from the pinned
[native-packages](https://github.com/crmne/native-packages) gem, installed with `gem install native-packages --version 0.2.0`.

Stable releases build the existing Linux, macOS and Windows artifacts first.
The shared packaging workflow then verifies their published checksums and attaches
Linux DEB/RPM packages and a recipe archive. AUR and Homebrew publication require
their configured repository variables and secrets. PRs only validate recipes.

The application retains its Flatpak manifests, macOS bundle/signing configuration
and Windows installer configuration. nFPM does not replace these platform tools.
See the repository's [maintainer packaging guide](https://github.com/crmne/fastpotify/blob/main/PACKAGING.md)
for commands and the shared tool's [platform coverage](https://github.com/crmne/native-packages/blob/main/docs/platforms.md)
for the boundaries.

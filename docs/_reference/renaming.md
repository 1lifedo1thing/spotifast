---
title: The Rename
description: How existing installations, commands, settings and links survive the new name.
---

Fastpotify is now **Spotifast**, at [spotifast.rocks](https://spotifast.rocks/).
Version 0.8.0 is the first release with the new name. Releases through 0.7.1
use the Fastpotify name.

## Existing installations

Settings, saved sign-ins, local pins, history, caches and window positions stay
where they are. There is no migration to a second set of directories. Saved
Spotify Connect names also stay as you chose them; new settings use Spotifast.

The `spotifast` and `fastpotify` commands open and control the same application.
Linux packages provide `spotifast` as an alias, without installing a second
copy of the app. Existing `playerctl --player=fastpotify` commands keep working.

AUR packages are now `spotifast`, `spotifast-bin` and `spotifast-git`.
The old packages have a packaging-only update that announces the move.
Install the matching new package and accept the replacement, for example:

```sh
yay -S spotifast-bin
```

There is no need to uninstall first or remove settings. The source and binary
release packages still use the same 0.8.0 application code. The `-git` variant
continues to build the current development revision.

The Homebrew cask is now `crmne/tap/spotifast`. Its rename metadata lets
Homebrew migrate existing installations during updates, or explicitly with
`brew migrate --cask fastpotify` after updating the tap. DEB/RPM packages are
also named `spotifast` and declare replacement of `fastpotify`.
Nix exposes `spotifast` and, on macOS, `spotifast-app`, alongside the old
attribute names. Community-maintained distribution packages may still use
the old name until their maintainers update them.

## Updates and packaging

Public release downloads use the `spotifast-` prefix. The 0.8.0 downloads
were renamed without changing their bytes. Byte-identical `fastpotify-`
compatibility downloads remain for installed update clients that request
those exact filenames, with both names recorded in `checksums.txt`. The compatibility command keeps its `fastpotify VERSION`
response. The Spotifast command reports `spotifast VERSION`; new update clients
accept either name and still require the exact expected version and checksum.

New macOS installations use `Spotifast.app`. The bundle ID remains
`me.paolino.fastpotify`, and its internal executable remains `fastpotify`.
The disk image also includes a hidden, signed copy named `Fastpotify.app` for
older updaters that require that path. Updating an existing installation
preserves its current bundle location. Homebrew updates remain owned by
Homebrew, whichever bundle name is installed.

Windows keeps its original installer ID, registry identities and installation
directory. Its app name and new shortcuts say Spotifast. The previous command
remains installed for existing shortcuts and scripts.

The Flatpak ID `rocks.fastpotify.Fastpotify`, protected credential-store service,
MPRIS names and single-instance protocol retain their original identities.
Changing these would create a separate app or disconnect existing integrations.

## Website links

The guides now live at `/using-spotifast/` and `/what-is-spotifast/`.
`jekyll-redirect-from` generates redirects from their old Fastpotify URLs.
The download page continues to link to the existing stable artifacts until a
new release is available.

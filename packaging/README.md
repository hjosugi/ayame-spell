# Distribution manifests

`generate_manifests.py` turns a release tag and its `SHA256SUMS` into:

- `ayame-spell.rb` for direct Homebrew formula installation or publication in
  `hjosugi/homebrew-tap`;
- `ayame-spell.json` for direct Scoop installation or a Scoop bucket;
- `PKGBUILD` and `.SRCINFO` for the x86_64/aarch64
  `ayame-spell-bin` AUR package.

The release workflow generates and attaches all four only after every
platform archive exists and its checksum is known:

```sh
python3 packaging/generate_manifests.py \
  --tag v0.4.0 \
  --checksums SHA256SUMS \
  --output-dir release-manifests
```

Publishing the Homebrew tap or AUR repository is intentionally a separate
external-repository action. This repository produces the exact ready-to-publish
files without writing to those repositories.

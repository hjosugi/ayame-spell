# ayame-spell for Node projects

This package installs the matching checksum-verified native ayame-spell release
binary for Linux x64/ARM64, macOS Intel/Apple silicon, or Windows x64.

```sh
npx ayame-spell check .
# or
npm install --save-dev ayame-spell
```

The package contains no spell-checking JavaScript implementation. Its
postinstall script downloads the GitHub release with the same version as the
npm package, verifies `SHA256SUMS`, and exposes the native binary through the
`ayame-spell` npm bin entry.

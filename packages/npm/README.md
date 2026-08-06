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

## Publishing

The release workflow uses npm trusted publishing from
`.github/workflows/release.yml`; no long-lived npm token is required after
bootstrap. Before setting the repository variable `PUBLISH_NPM=true`, the
repository owner must publish the package once if it does not exist, then
configure its npm trusted publisher as GitHub Actions repository
`hjosugi/ayame-spell` with workflow filename `release.yml`, allowing
`npm publish`. Trusted publishing adds provenance automatically.

Those npm account and policy operations are intentionally not automated here.

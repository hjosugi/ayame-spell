#!/usr/bin/env python3
"""Regression tests for generated distribution manifests."""

from __future__ import annotations

import unittest

from generate_manifests import TARGETS, asset, pkgbuild, srcinfo


class AurManifestTests(unittest.TestCase):
    def setUp(self) -> None:
        self.tag = "v9.8.7"
        self.checksums = {
            asset(self.tag, key): str(index) * 64
            for index, key in enumerate(TARGETS, start=1)
        }

    def test_pkgbuild_and_srcinfo_cover_both_linux_architectures(self) -> None:
        build = pkgbuild(self.tag, self.checksums)
        info = srcinfo(self.tag, self.checksums)
        self.assertIn("arch=('x86_64' 'aarch64')", build)
        self.assertIn("\tarch = x86_64", info)
        self.assertIn("\tarch = aarch64", info)
        for key, arch in (("linux-x64", "x86_64"), ("linux-arm64", "aarch64")):
            name = asset(self.tag, key)
            self.assertIn(f"source_{arch}=(\"{name}::", build)
            self.assertIn(f"\tsource_{arch} = {name}::", info)
            self.assertIn(self.checksums[name], build)
            self.assertIn(self.checksums[name], info)

    def test_aur_runtime_metadata_is_kept_in_sync(self) -> None:
        build = pkgbuild(self.tag, self.checksums)
        info = srcinfo(self.tag, self.checksums)
        for dependency in ("gcc-libs", "glibc"):
            self.assertIn(dependency, build)
            self.assertIn(f"\tdepends = {dependency}", info)
        self.assertIn('provides=("ayame-spell=$pkgver")', build)
        self.assertIn("\tprovides = ayame-spell=9.8.7", info)
        self.assertIn("conflicts=('ayame-spell')", build)
        self.assertIn("\tconflicts = ayame-spell", info)


if __name__ == "__main__":
    unittest.main()

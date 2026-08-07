#!/usr/bin/env python3
"""Regression tests for generated distribution manifests."""

from __future__ import annotations

import tempfile
import unittest
import unittest.mock
from pathlib import Path

from generate_manifests import SRCINFO_ASSET, TARGETS, asset, pkgbuild, srcinfo


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


class SrcinfoAssetNameTests(unittest.TestCase):
    def test_the_asset_name_survives_a_github_release_upload(self) -> None:
        # v0.5.0 shipped this as `.SRCINFO` and GitHub published it as
        # `default.SRCINFO`, because a release asset cannot lead with a dot.
        self.assertFalse(SRCINFO_ASSET.startswith("."))
        self.assertTrue(SRCINFO_ASSET.endswith(".SRCINFO"))

    def test_main_writes_every_manifest_under_a_publishable_name(self) -> None:
        import generate_manifests

        tag = "v9.8.7"
        checksums = {
            asset(tag, key): str(index) * 64
            for index, key in enumerate(TARGETS, start=1)
        }
        with tempfile.TemporaryDirectory() as directory:
            out = Path(directory)
            sums = out / "SHA256SUMS"
            sums.write_text(
                "".join(f"{digest}  {name}\n" for name, digest in checksums.items())
            )
            argv = ["--tag", tag, "--checksums", str(sums), "--output-dir", str(out)]
            with unittest.mock.patch("sys.argv", ["generate_manifests", *argv]):
                generate_manifests.main()

            written = {path.name for path in out.iterdir()} - {"SHA256SUMS"}
            self.assertEqual(
                written,
                {"ayame-spell.rb", "ayame-spell.json", "PKGBUILD", SRCINFO_ASSET},
            )
            for name in written:
                self.assertFalse(
                    name.startswith("."), f"{name} would be renamed on upload"
                )
            self.assertIn("pkgver = 9.8.7", (out / SRCINFO_ASSET).read_text())


if __name__ == "__main__":
    unittest.main()

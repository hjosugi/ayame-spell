import { defineConfig } from "astro/config";
import starlight from "@astrojs/starlight";

const base = "/ayame-spell";

const localeScript = `
(() => {
  const key = "ayame-spell:locale";
  const base = "${base}/";
  const path = window.location.pathname;
  const stored = window.localStorage.getItem(key);
  const root = path === base || path === base.slice(0, -1);
  const current = path === base + "ja" || path.startsWith(base + "ja/") ? "ja" : "en";

  if (root && stored === "ja") {
    window.location.replace(base + "ja/");
    return;
  }
  if (root && !stored && navigator.languages?.some((lang) => lang.toLowerCase().startsWith("ja"))) {
    window.localStorage.setItem(key, "ja");
    window.location.replace(base + "ja/");
    return;
  }
  window.localStorage.setItem(key, current);

  const remember = (value) => {
    if (!value) return;
    const url = new URL(value, window.location.href);
    if (url.pathname === base || url.pathname === base.slice(0, -1)) {
      window.localStorage.setItem(key, "en");
    } else if (url.pathname === base + "ja" || url.pathname.startsWith(base + "ja/")) {
      window.localStorage.setItem(key, "ja");
    }
  };
  document.addEventListener("click", (event) => {
    const link = event.target.closest?.("a[href]");
    if (link) remember(link.href);
  });
  document.addEventListener("change", (event) => {
    const target = event.target;
    if (target instanceof HTMLSelectElement) remember(target.value);
  });
})();
`;

export default defineConfig({
  site: "https://hjosugi.github.io",
  base,
  trailingSlash: "always",
  integrations: [
    starlight({
      title: "ayame-spell",
      description:
        "Fast, low-noise spell checking for English and Japanese code and prose.",
      customCss: ["./src/styles/custom.css"],
      editLink: {
        baseUrl: "https://github.com/ayame-editor/ayame-spell/edit/main/site/",
      },
      locales: {
        root: {
          label: "English",
          lang: "en",
        },
        ja: {
          label: "日本語",
          lang: "ja",
        },
      },
      social: [
        {
          icon: "github",
          label: "GitHub",
          href: "https://github.com/ayame-editor/ayame-spell",
        },
      ],
      components: {
        Head: "./src/components/Head.astro",
        SocialIcons: "./src/components/HeaderTools.astro",
      },
      head: [
        {
          tag: "script",
          content: localeScript,
        },
      ],
      sidebar: [
        {
          label: "Start here",
          translations: { ja: "はじめに" },
          items: [
            { slug: "getting-started" },
            { slug: "modes" },
            { slug: "editors" },
            { slug: "ci" },
          ],
        },
        {
          label: "Reference",
          translations: { ja: "リファレンス" },
          items: [
            { slug: "reference/configuration" },
            { slug: "reference/rules" },
            { slug: "reference/cli" },
            { slug: "reference/directives" },
            { slug: "reference/output" },
            { slug: "reference/environment" },
          ],
        },
        {
          label: "Guides",
          translations: { ja: "ガイド" },
          items: [
            { slug: "japanese" },
            { slug: "english" },
            { slug: "syntax" },
            { slug: "registry" },
            { slug: "migration" },
            { slug: "benchmarks" },
          ],
        },
        {
          label: "Help",
          translations: { ja: "ヘルプ" },
          items: [{ slug: "faq" }, { slug: "troubleshooting" }],
        },
      ],
    }),
  ],
});

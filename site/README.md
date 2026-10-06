# OneSource website

The website is built from `index.html`, `site/docs.html`, `site/install-content.html`, `site/docs.en.json`, `site/docs.zh.json`, the English and Traditional Chinese UI dictionaries, and `site/config.json`. Edit these sources, not generated HTML. Node.js 24 or newer is required; there are no npm dependencies to install.

```sh
npm run build:site
npm run test:site
npm run test:docs # requires the Rust toolchain and cached dependencies
npm run preview:site
```

Preview at http://127.0.0.1:4173. The preview server mirrors directory redirects and returns the custom 404 page with a 404 status. `dist/` is generated and ignored by Git.

## URLs and language selection

The published origin is https://onesource.zynimous.dev. `/en/` is English, `/zh/` is Traditional Chinese. Documentation lives at `/en/docs/` and `/zh/docs/`. Each language has an index plus install, getting-started, cli, filtering, tree, profiles, explain, output, update and troubleshooting pages. Body sections and stable section IDs live in `site/docs.{en,zh}.json`; the installation body uses `site/install-content.html` and the UI dictionaries. Keep section IDs and page structure identical across languages. Old `/en/install/` and `/zh/install/` pages redirect to `docs/install/` with query/anchor preservation, with a native link for JavaScript-disabled browsers. Both homepages and docs contain complete static content and work without JavaScript. `/` contains an English fallback and redirects using a saved manual preference, then the first supported browser language, then English. Chinese variants all select Traditional Chinese. Explicit language URLs never redirect based on preferences. Manual switches preserve the current page, query parameters and section anchors and save `onesource-language`; existing `zh-Hant` preferences remain supported.

Each language URL has its own canonical, metadata, and JSON-LD. The root fallback canonical points to `/en/`; `x-default` points to `/`. The sitemap lists the two homepages and 22 docs pages, excluding old installation redirects. Docs language alternates point to the corresponding page; their `x-default` is the English docs page. Every docs page has its own metadata and structured data. Shared navigation provides breadcrumbs, a desktop sidebar, a native expandable mobile index, a page contents list and previous/next links. Clipboard controls are progressively enhanced by `site/install.js`, leaving all commands readable without JavaScript. GitHub Pages adds trailing slashes when visiting `/en` or `/zh`.

## GitHub Pages setup

1. Commit the website sources, optimized media, scripts, tests, and Pages workflow to `main`.
2. In repository **Settings → Pages**, set **Source** to **GitHub Actions**.
3. Set **Custom domain** to `onesource.zynimous.dev` in Pages settings. With an Actions publishing workflow, a repository `CNAME` file is not required and is ignored by GitHub Pages.
4. At the DNS provider, add a **CNAME** record for `onesource` pointing to `TW-RF54732.github.io` (no protocol or repository path). Remove conflicting records for that same hostname if present. If the parent domain has CAA records, ensure GitHub's certificate authority is permitted.
5. Run **Website checks and GitHub Pages** manually or push website changes to `main`. PRs only build and test; they do not deploy.
6. Once GitHub confirms the DNS configuration and issues a certificate, enable **Enforce HTTPS**.

The existing Rust test and release workflows are independent. Only website artifacts are uploaded; the repository itself is not published as the site. A manual workflow on a non-main branch runs checks but does not deploy.

## Verification after publishing

- Check HTTPS, `/en/`, `/zh/`, `/en/docs/`, `/zh/docs/`, all documentation routes, old installation redirects, `/robots.txt`, `/sitemap.xml`, the share image, and a nonexistent URL (which must return HTTP 404).
- Test the root with English, Chinese, unsupported languages, saved preferences, and JavaScript disabled. Check language links and anchors on desktop/mobile, keyboard navigation, and reduced motion.
- Verify `zynimous.dev` or `onesource.zynimous.dev` as a Search Console domain property using Google's DNS TXT record; submit `https://onesource.zynimous.dev/sitemap.xml` and inspect all 24 canonical URLs.
- Run Lighthouse and validate structured data. Software markup describes real project facts; there are no invented ratings/reviews, so eligibility for a software rich result is not assumed.

## Image maintenance

The checked-in responsive assets avoid image tooling during CI. To regenerate after changing the original transparent logo, use ImageMagick:

```sh
magick medias/oneSourceLogoTransparent.png -strip -resize 320x -quality 82 medias/logo-320.webp
magick medias/oneSourceLogoTransparent.png -strip -resize 640x -quality 82 medias/logo-640.webp
magick medias/oneSourceLogoTransparent.png -strip -resize 960x -quality 82 medias/logo-960.webp
magick medias/oneSourceLogoTransparent.png -strip -resize 960x -define png:compression-level=9 medias/logo.png
magick medias/oneSourceLogoTransparent.png -strip -resize 32x32 -gravity center -extent 32x32 medias/favicon.png
magick medias/oneSourceLogoTransparent.png -strip -resize 180x180 -gravity center -extent 180x180 medias/apple-touch-icon.png
```

Sources: [GitHub Actions publishing](https://docs.github.com/en/pages/getting-started-with-github-pages/using-custom-workflows-with-github-pages), [custom domains](https://docs.github.com/en/pages/configuring-a-custom-domain-for-your-github-pages-site/managing-a-custom-domain-for-your-github-pages-site), [Google multilingual sites](https://developers.google.com/search/docs/specialty/international/managing-multi-regional-sites).

## Documentation verification

`npm run test:site` validates every docs route, local resource and anchor, matching bilingual section IDs, metadata, sitemap, redirects and progressive clipboard controls. It also compares documented arguments, profile fields, blacklist entries and diagnostic output with Rust sources. `npm run test:docs` builds the local CLI offline and runs a temporary-project integration test of the published examples and important combinations. If dependencies are not cached, run `cargo build` first. These CLI checks are separate from the existing Node-only Pages build. After changing Rust behavior, update both docs languages, check nested command help, run both test commands and manually verify responsive/keyboard/no-JavaScript behavior.

## Writing topic tutorials

Keep CLI reference as a lookup page. Topic pages are independently runnable lessons: state the goal and prerequisites, provide their own starting files, explain each command where it is used, show concrete expected selections/output or configuration changes, and explain correction plus re-verification. Do not replace a lesson with an option list or an unexplained multi-command block. Preserve existing section IDs when expanding content and match new IDs and runnable command sequences across languages.

In topic body HTML, mark executable snippets with `<pre data-kind="command">`, terminal/bundle excerpts with `data-kind="output"`, JSON with `data-kind="config"`, and non-executable lists with `data-kind="reference"`; each contains a `<code>` element. Only commands receive copy controls. Explain excerpt omissions and variable values in the surrounding prose. The installation fragment uses its existing explicit command panels.

The CLI integration suite executes the command blocks published on six independent tutorial pages against fresh fixtures, checks selection/tree/profile/output results, and verifies the published old-to-new configuration migration. User-specific placeholder roots, desktop clipboard delivery, network installation and executable replacement are not performed by that suite. Clipboard controls are tested by the website suite; update/install behavior is checked against the repository implementation, with final delivery/platform verification left to the reader.

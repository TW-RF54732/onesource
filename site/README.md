# OneSource website

The website is built from `index.html`, `site/install.html`, the English and Traditional Chinese JSON dictionaries, and `site/config.json`. Edit these sources, not generated HTML. Node.js 24 or newer is required; there are no npm dependencies to install.

```sh
npm run build:site
npm run test:site
npm run preview:site
```

Preview at http://127.0.0.1:4173. The preview server mirrors directory redirects and returns the custom 404 page with a 404 status. `dist/` is generated and ignored by Git.

## URLs and language selection

The published origin is https://onesource.zynimous.dev. `/en/` is English, `/zh/` is Traditional Chinese. Installation guides live at `/en/install/` and `/zh/install/`, with commands and copy labels in the same language dictionaries. Both homepages and guides contain complete static content and work without JavaScript. `/` contains an English fallback and redirects using a saved manual preference, then the first supported browser language, then English. Chinese variants all select Traditional Chinese. Explicit language URLs never redirect based on preferences. Manual switches preserve the current page, query parameters and section anchors and save `onesource-language`; existing `zh-Hant` preferences remain supported.

Each language URL has its own canonical, metadata, and JSON-LD. The root fallback canonical points to `/en/`; `x-default` points to `/`. The sitemap lists the two homepages and two installation guides. Guide language alternates point to the corresponding guide; their `x-default` is `/en/install/`. Clipboard controls are progressively enhanced by `site/install.js`, leaving all commands readable without JavaScript. GitHub Pages adds trailing slashes when visiting `/en` or `/zh`.

## GitHub Pages setup

1. Commit the website sources, optimized media, scripts, tests, and Pages workflow to `main`.
2. In repository **Settings → Pages**, set **Source** to **GitHub Actions**.
3. Set **Custom domain** to `onesource.zynimous.dev` in Pages settings. With an Actions publishing workflow, a repository `CNAME` file is not required and is ignored by GitHub Pages.
4. At the DNS provider, add a **CNAME** record for `onesource` pointing to `TW-RF54732.github.io` (no protocol or repository path). Remove conflicting records for that same hostname if present. If the parent domain has CAA records, ensure GitHub's certificate authority is permitted.
5. Run **Website checks and GitHub Pages** manually or push website changes to `main`. PRs only build and test; they do not deploy.
6. Once GitHub confirms the DNS configuration and issues a certificate, enable **Enforce HTTPS**.

The existing Rust test and release workflows are independent. Only website artifacts are uploaded; the repository itself is not published as the site. A manual workflow on a non-main branch runs checks but does not deploy.

## Verification after publishing

- Check HTTPS, `/en/`, `/zh/`, `/en/install/`, `/zh/install/`, `/robots.txt`, `/sitemap.xml`, the share image, and a nonexistent URL (which must return HTTP 404).
- Test the root with English, Chinese, unsupported languages, saved preferences, and JavaScript disabled. Check language links and anchors on desktop/mobile, keyboard navigation, and reduced motion.
- Verify `zynimous.dev` or `onesource.zynimous.dev` as a Search Console domain property using Google's DNS TXT record; submit `https://onesource.zynimous.dev/sitemap.xml` and inspect all four canonical URLs.
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

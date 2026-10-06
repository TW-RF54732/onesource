# Website validation

Validated on 2026-10-06 against the generated site at http://127.0.0.1:4173 using Lighthouse 13.5.0 mobile emulation and Chromium 153. These are local measurements; verify HTTPS, caching, canonical responses and Search Console indexing again after publishing.

| Page | Performance | Accessibility | SEO | LCP | CLS |
| --- | ---: | ---: | ---: | ---: | ---: |
| /en/ | 100 | 100 | 100 | 1.2 s | 0 |
| /zh/ | 100 | 100 | 100 | 1.2 s | 0 |

- `npm run build:site` completed successfully.
- `npm run test:site`: 7 test groups passed, including localized HTML and metadata, JSON-LD, canonical/hreflang, sitemap, local assets, URL preference resolution, storage failure, preserved anchors, and actual HTTP 404/301/200 responses.
- 16 browser scenarios passed: root language selection, legacy saved preference, unavailable storage, keyboard switching, JavaScript disabled, reduced motion, 404 status, and both languages at 360/768/1440 px without horizontal overflow.
- Responsive WebP assets: 8,464 / 19,966 / 37,030 bytes, compared with the original 793,571-byte PNG.
- Node syntax checks, sitemap XML parsing and Git whitespace checks passed.

The browser and Lighthouse tools were installed in /tmp without adding project dependencies. Only the automated Node checks are part of the Pages CI workflow. Browser screenshots and full Lighthouse reports from this run are also under /tmp/onesource-seo-*; they are temporary and not deployed.

## Installation guides — 2026-10-06

Validated the new `/en/install/` and `/zh/install/` pages locally with Chromium 153. The earlier Lighthouse scores above apply to the homepages only; the guides were not measured with Lighthouse.

- `npm run build:site` passed; `npm run test:site` passed all 10 test groups, including guide metadata, canonical/hreflang, sitemap, static content, local resources, language switching, clipboard success/failure/unavailability, and HTTP 200/301/404 behavior.
- 13 browser scenarios passed: both guide languages at 360/768/1440 px without horizontal overflow; homepage entry links; keyboard language switching with query and anchor preservation; JavaScript disabled; blocked storage; clipboard success, denied access and unavailable API; reduced motion.
- Browser screenshots were visually inspected for the Chinese desktop and English mobile layouts. Temporary screenshots are in `/tmp/onesource-install-*.png`; the browser check script is `/tmp/onesource-install-browser.mjs`.
- JavaScript syntax checks and `git diff --check` passed. Local HTTP and browser checks ran with sandbox escalation to allow loopback listening and Chromium launch.
- The guide changes have not been deployed. Verify HTTPS, guide URLs and indexing after the existing GitHub Pages workflow publishes them.

## Complete bilingual documentation — 2026-10-06

- Added 22 static docs pages at `/en/docs/` and `/zh/docs/`, with matched section IDs and 24 canonical sitemap entries including homepages. Old installation URLs are excluded from the sitemap and provide query/hash-preserving JavaScript redirects plus native fallback links.
- `npm run build:site` and all 15 `npm run test:site` groups passed. Checks cover all docs routes/resources/anchors, localized SEO/JSON-LD, navigation, homepage entry/download/release links, redirects, language switching, clipboard enhancement and source-based CLI/profile/blacklist/diagnostic coverage.
- `npm run test:docs` passed against the locally built CLI using an automatically cleaned temporary project. It exercises file output, exact 1024/1025-byte boundaries, NUL detection, lossy UTF-8, ignore/blacklist diagnostics, independent tree selection, empty filters, explicit false, dry-run/copy/save, explain's ignored action flags, profile merge/replacement, legacy aliases, invalid globs and directory symlinks.
- `cargo test --offline`: all 29 Rust tests passed. Rust implementation was not changed. All options from 11 built-in help screens were checked against both languages.
- 75 Chromium scenarios passed: every docs page in both languages at 360/768/1440 px without document-level horizontal overflow, readable static content with JavaScript disabled, expandable mobile index, native legacy fallback links, query/hash-preserving automatic redirects, keyboard language switching and homepage Hero installation navigation. Desktop and mobile screenshots were visually inspected. Temporary browser script/screenshots: `/tmp/onesource-docs-browser.mjs` and `/tmp/onesource-docs-*.png`.
- Syntax and Git whitespace checks passed. Local HTTP/browser checks required sandbox escalation for loopback listening and Chromium launch. No new Lighthouse scores are claimed for these docs pages.
- Changes have not been deployed; publishing continues through the existing GitHub Pages workflow.

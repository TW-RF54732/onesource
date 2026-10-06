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

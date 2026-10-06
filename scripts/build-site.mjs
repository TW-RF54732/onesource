import { readFile, writeFile, mkdir, copyFile, rm } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

export const root = fileURLToPath(new URL('../', import.meta.url));
const read = name => readFile(path.join(root, name), 'utf8');
const escape = value => String(value).replace(/[&<>"']/g, char => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' })[char]);
const serialize = value => JSON.stringify(value).replace(/</g, '\\u003c');
const locales = { en: { lang: 'en', og: 'en_US' }, zh: { lang: 'zh-Hant', og: 'zh_TW' } };

export function validateConfig(config) {
  const origin = new URL(config.origin);
  if (origin.protocol !== 'https:' || origin.origin !== config.origin || origin.username || origin.password) {
    throw new Error('Site origin must be an HTTPS origin without a trailing slash or path.');
  }
  const repository = new URL(config.repository);
  if (repository.protocol !== 'https:' || repository.hostname !== 'github.com' || repository.search || repository.hash || repository.href !== config.repository) {
    throw new Error('Repository must be an HTTPS GitHub URL.');
  }
}

export function render(template, values, raw = new Set()) {
  const html = template.replace(/{{([a-zA-Z0-9]+)}}/g, (_, key) => {
    if (typeof values[key] !== 'string' || !values[key].trim()) throw new Error(`Missing translation or template value: ${key}`);
    return raw.has(key) ? values[key] : escape(values[key]);
  });
  if (/{{|data-i18n=|data-label=/.test(html)) throw new Error('Unresolved template markup.');
  return html;
}

export async function buildSite() {
  const output = path.join(root, 'dist');
  const config = JSON.parse(await read('site/config.json'));
  validateConfig(config);
  const template = await read('index.html');
  const translations = Object.fromEntries(await Promise.all(Object.keys(locales).map(async key => [key, JSON.parse(await read(`site/${key}.json`))])));
  const keys = Object.keys(translations.en).sort();
  if (JSON.stringify(keys) !== JSON.stringify(Object.keys(translations.zh).sort())) throw new Error('Translation keys must match.');
  for (const [locale, copy] of Object.entries(translations)) {
    for (const key of keys) if (typeof copy[key] !== 'string' || !copy[key].trim()) throw new Error(`Missing translation: ${locale}.${key}`);
  }
  const alternates = `${Object.entries(locales).map(([key, value]) => `<link rel="alternate" hreflang="${value.lang}" href="${config.origin}/${key}/">`).join('\n')}\n<link rel="alternate" hreflang="x-default" href="${config.origin}/">`;
  const image = `${config.origin}/medias/OneSourceSocialCard.png`;
  const pages = [];
  for (const [route, locale] of [['', 'en'], ['en', 'en'], ['zh', 'zh']]) {
    const copy = translations[locale];
    const canonical = `${config.origin}/${locale}/`;
    const jsonld = {
      '@context': 'https://schema.org',
      '@graph': [
        { '@type': 'WebSite', '@id': `${config.origin}/#website`, url: `${config.origin}/`, name: 'OneSource', inLanguage: ['en', 'zh-Hant'] },
        { '@type': 'WebPage', '@id': `${canonical}#webpage`, url: canonical, name: copy.title, description: copy.description, inLanguage: locales[locale].lang, isPartOf: { '@id': `${config.origin}/#website` }, about: { '@id': `${config.origin}/#software` } },
        { '@type': 'SoftwareApplication', '@id': `${config.origin}/#software`, name: 'OneSource', url: canonical, description: copy.description, applicationCategory: 'DeveloperApplication', operatingSystem: 'Windows, macOS, Linux', license: `${config.repository}/blob/main/LICENSE`, isAccessibleForFree: true, offers: { '@type': 'Offer', price: '0', priceCurrency: 'USD' }, downloadUrl: `${config.repository}/releases`, sameAs: [config.repository] }
      ]
    };
    const head = `<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>${escape(copy.title)}</title>
<meta name="description" content="${escape(copy.description)}">
<meta name="color-scheme" content="dark">
<link rel="canonical" href="${canonical}">
${alternates}
<meta property="og:type" content="website">
<meta property="og:site_name" content="OneSource">
<meta property="og:title" content="${escape(copy.title)}">
<meta property="og:description" content="${escape(copy.description)}">
<meta property="og:url" content="${canonical}">
<meta property="og:locale" content="${locales[locale].og}">
<meta property="og:locale:alternate" content="${locales[locale === 'en' ? 'zh' : 'en'].og}">
<meta property="og:image" content="${image}">
<meta property="og:image:width" content="1280">
<meta property="og:image:height" content="640">
<meta property="og:image:alt" content="${escape(copy.socialAlt)}">
<meta name="twitter:card" content="summary_large_image">
<meta name="twitter:title" content="${escape(copy.title)}">
<meta name="twitter:description" content="${escape(copy.description)}">
<meta name="twitter:image" content="${image}">
<meta name="twitter:image:alt" content="${escape(copy.socialAlt)}">
<link rel="icon" href="/medias/favicon.png" type="image/png" sizes="32x32">
<link rel="apple-touch-icon" href="/medias/apple-touch-icon.png" sizes="180x180">
<link rel="stylesheet" href="/styles.css">
<script type="application/ld+json">${serialize(jsonld)}</script>
<script src="/language.js"${route ? ' defer' : ''}></script>
<script src="/script.js" defer></script>`;
    pages.push([path.join(route, 'index.html'), render(template, {
      ...copy, head, lang: locales[locale].lang,
      enCurrent: locale === 'en' ? 'aria-current="true"' : 'class="language-alternate"',
      zhCurrent: locale === 'zh' ? 'aria-current="true"' : 'class="language-alternate"',
      docsUrl: locale === 'en' ? `${config.repository}#readme` : `${config.repository}/blob/main/README_zh.md`
    }, new Set(['head', 'enCurrent', 'zhCurrent']))]);
  }
  const xmlAlternates = `${Object.entries(locales).map(([key, value]) => `<xhtml:link rel="alternate" hreflang="${value.lang}" href="${config.origin}/${key}/"/>`).join('')}<xhtml:link rel="alternate" hreflang="x-default" href="${config.origin}/"/>`;
  pages.push(['sitemap.xml', `<?xml version="1.0" encoding="UTF-8"?>\n<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9" xmlns:xhtml="http://www.w3.org/1999/xhtml">${Object.keys(locales).map(key => `<url><loc>${config.origin}/${key}/</loc>${xmlAlternates}</url>`).join('')}</urlset>\n`]);
  pages.push(['robots.txt', `User-agent: *\nAllow: /\n\nSitemap: ${config.origin}/sitemap.xml\n`]);
  pages.push(['404.html', `<!doctype html><html lang="en"><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width, initial-scale=1"><meta name="robots" content="noindex"><title>Page not found — OneSource</title><link rel="stylesheet" href="/styles.css"></head><body><main class="container error-page"><h1>404</h1><h2>Page not found</h2><p>This page may have moved. Choose a language to visit OneSource.</p><nav aria-label="Language selection"><a class="button primary" href="/en/" lang="en">English</a> <a class="button secondary" href="/zh/" lang="zh-Hant">繁體中文</a></nav></main></body></html>\n`]);
  pages.push(['.nojekyll', '']);
  await rm(output, { recursive: true, force: true });
  await mkdir(path.join(output, 'medias'), { recursive: true });
  for (const [name, content] of pages) {
    await mkdir(path.dirname(path.join(output, name)), { recursive: true });
    await writeFile(path.join(output, name), content);
  }
  const assets = ['styles.css', 'script.js', 'site/language.js', ...['logo-320.webp', 'logo-640.webp', 'logo-960.webp', 'logo.png', 'favicon.png', 'apple-touch-icon.png', 'OneSourceSocialCard.png'].map(name => `medias/${name}`)];
  for (const asset of assets) await copyFile(path.join(root, asset), path.join(output, asset === 'site/language.js' ? 'language.js' : asset));
  return output;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  await buildSite();
  console.log('Static website built in dist/');
}

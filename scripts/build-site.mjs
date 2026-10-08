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
  const installTemplate = await read('site/install-content.html');
  const docsTemplate = await read('site/docs.html');
  const docs = Object.fromEntries(await Promise.all(Object.keys(locales).map(async key => [key, JSON.parse(await read(`site/docs.${key}.json`))])));
  const slugs = ['', 'install', 'getting-started', 'cli', 'filtering', 'tree', 'profiles', 'explain', 'output', 'update', 'troubleshooting'];
  for (const slug of slugs.filter(slug => slug !== 'install')) {
    if (!docs.en[slug] || !docs.zh[slug]) throw new Error(`Missing docs page: ${slug}`);
    if (JSON.stringify(docs.en[slug].sections.map(s => s.id)) !== JSON.stringify(docs.zh[slug].sections.map(s => s.id))) throw new Error(`Docs section mismatch: ${slug}`);
  }
  const translations = Object.fromEntries(await Promise.all(Object.keys(locales).map(async key => [key, JSON.parse(await read(`site/${key}.json`))])));
  const keys = Object.keys(translations.en).sort();
  if (JSON.stringify(keys) !== JSON.stringify(Object.keys(translations.zh).sort())) throw new Error('Translation keys must match.');
  for (const [locale, copy] of Object.entries(translations)) {
    for (const key of keys) if (typeof copy[key] !== 'string' || !copy[key].trim()) throw new Error(`Missing translation: ${locale}.${key}`);
  }
  const image = `${config.origin}/medias/OneSourceSocialCard.png`;
  const pages = [];
  const routes = [['', 'en'], ['en', 'en'], ['zh', 'zh'], ...Object.keys(locales).flatMap(locale => slugs.map(slug => [`${locale}/docs/${slug}`, locale, true, slug]))];
  for (const [route, locale, isDocs = false, slug = ''] of routes) {
    const guide = isDocs && slug === 'install';
    const suffix = isDocs ? `docs/${slug ? slug + '/' : ''}` : '';
    const copy = { ...translations[locale] };
    if (isDocs) { copy.title = guide ? copy.guideTitle : `${docs[locale][slug].title} — OneSource`; copy.description = guide ? copy.guideDescription : docs[locale][slug].description; copy.socialAlt = copy.title; }
    const canonical = `${config.origin}/${locale}/${suffix}`;
    const alternates = `${Object.entries(locales).map(([key, value]) => `<link rel="alternate" hreflang="${value.lang}" href="${config.origin}/${key}/${suffix}">`).join('\n')}\n<link rel="alternate" hreflang="x-default" href="${config.origin}/${isDocs ? 'en/' + suffix : ''}">`;
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
<script src="/${isDocs ? 'install.js' : 'script.js'}" defer></script>`;
    const docsUrl = `/${locale}/docs/`;
    const pageUrl = (language, page) => `/${language}/docs/${page ? page + '/' : ''}`;
    const titleFor = page => page === 'install' ? copy.guideLink : docs[locale][page].title;
    let body = '';
    let toc = '';
    let heading = '';
    if (isDocs) {
      heading = guide ? copy.guideHeading : titleFor(slug);
      const index = `<ul>${slugs.slice(1).map(page => `<li><a href="${pageUrl(locale, page)}">${escape(titleFor(page))}</a></li>`).join('')}</ul>`;
      if (guide) {
        body = render(installTemplate, { ...copy, docsUrl });
        toc = [...body.matchAll(/<section[^>]*id="([^"]+)"[^>]*>[\s\S]*?<h2[^>]*>([^<]+)<\/h2>/g)].map(m => `<a href="#${m[1]}">${m[2]}</a>`).join('');
      } else {
        body = docs[locale][slug].sections.map(section => {
          if (!section.id || !section.title || !section.html) throw new Error(`Incomplete docs section: ${locale}/${slug}`);
          return `<section class="guide-section" id="${escape(section.id)}"><h2>${escape(section.title)}</h2>${section.html.replace('{{index}}', index)}</section>`;
        }).join('');
        toc = docs[locale][slug].sections.map(section => `<a href="#${escape(section.id)}">${escape(section.title)}</a>`).join('');
      }
      // Copy controls belong only to runnable commands, never output or JSON examples.
      body = body.replace(/<pre(?: data-kind="(command|output|config|reference)")?><code>([\s\S]*?)<\/code><\/pre>/g, (block, kind = 'command') => {
        if (guide) return block; // Installation command panels already include their controls.
        const label = kind === 'output' ? copy.expectedOutput : kind === 'config' ? copy.configurationExample : kind === 'reference' ? copy.referenceExample : copy.commandLabel;
        if (kind !== 'command') return `<div class="doc-example doc-${kind}"><div class="code-heading"><span>${escape(label)}</span></div>${block}</div>`;
        return `<div class="guide-command"><div class="code-heading"><span>${escape(label)}</span><button type="button" class="copy-command" hidden>${escape(copy.guideCopy)}</button></div>${block.replace('<pre', '<pre tabindex="0"')}<p class="copy-status" role="status" aria-live="polite"></p></div>`;
      });
    }
    const position = slugs.indexOf(slug);
    const pagination = [position > 0 ? [slugs[position - 1], locale === 'en' ? 'Previous' : '上一頁'] : null, position < slugs.length - 1 ? [slugs[position + 1], locale === 'en' ? 'Next' : '下一頁'] : null].filter(Boolean).map(([page, label]) => `<a href="${pageUrl(locale, page)}">${label}: ${escape(titleFor(page))}</a>`).join('');
    pages.push([path.join(route, 'index.html'), render(isDocs ? docsTemplate : template, {
      ...copy, head, lang: locales[locale].lang, heading, body, toc, pagination,
      headerPath: slug ? `<a href="${docsUrl}">${escape(copy.docsLabel)}</a><span aria-hidden="true">/</span><span aria-current="page">${escape(titleFor(slug))}</span>` : `<span aria-current="page">${escape(copy.docsLabel)}</span>`,
      sidebar: slugs.map(page => `<a href="${pageUrl(locale, page)}"${page === slug ? ' aria-current="page"' : ''}>${escape(titleFor(page))}</a>`).join(''),
      navigationLabel: locale === 'en' ? 'Main navigation' : '主要導覽',
      breadcrumbLabel: locale === 'en' ? 'Breadcrumbs' : '麵包屑',
      paginationLabel: locale === 'en' ? 'Previous and next pages' : '前後頁導覽',
      homeUrl: `/${locale}/`, installUrl: `/${locale}/docs/install/`, repositoryUrl: config.repository,
      enUrl: `/en/${suffix}`, zhUrl: `/zh/${suffix}`,
      enCurrent: locale === 'en' ? 'aria-current="true"' : 'class="language-alternate"',
      zhCurrent: locale === 'zh' ? 'aria-current="true"' : 'class="language-alternate"', docsUrl
    }, new Set(['head', 'enCurrent', 'zhCurrent', 'body', 'toc', 'sidebar', 'pagination', 'headerPath']))]);
  }
  for (const locale of Object.keys(locales)) {
    const target = `/${locale}/docs/install/`;
    pages.push([`${locale}/install/index.html`, `<!doctype html><html lang="${locales[locale].lang}"><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width, initial-scale=1"><meta name="robots" content="noindex"><link rel="canonical" href="${config.origin}${target}"><title>${escape(translations[locale].guideTitle)}</title><link rel="stylesheet" href="/styles.css"><script src="/install-redirect.js" defer></script></head><body><main class="container error-page"><h1>${escape(translations[locale].guideHeading)}</h1><p><a id="install-destination" href="${target}">${escape(translations[locale].guideLink)} →</a></p></main></body></html>`]);
  }
  const sitemapEntries = ['', ...slugs.map(slug => `docs/${slug ? slug + '/' : ''}`)].flatMap(suffix => {
    const alternates = Object.entries(locales).map(([key, value]) => `<xhtml:link rel="alternate" hreflang="${value.lang}" href="${config.origin}/${key}/${suffix}"/>`).join('') + `<xhtml:link rel="alternate" hreflang="x-default" href="${config.origin}/${suffix ? 'en/' + suffix : ''}"/>`;
    return Object.keys(locales).map(key => `<url><loc>${config.origin}/${key}/${suffix}</loc>${alternates}</url>`);
  });
  pages.push(['sitemap.xml', `<?xml version="1.0" encoding="UTF-8"?>\n<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9" xmlns:xhtml="http://www.w3.org/1999/xhtml">${sitemapEntries.join('')}</urlset>\n`]);
  pages.push(['robots.txt', `User-agent: *\nAllow: /\n\nSitemap: ${config.origin}/sitemap.xml\n`]);
  pages.push(['404.html', `<!doctype html><html lang="en"><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width, initial-scale=1"><meta name="robots" content="noindex"><title>Page not found — OneSource</title><link rel="stylesheet" href="/styles.css"></head><body><main class="container error-page"><h1>404</h1><h2>Page not found</h2><p>This page may have moved. Choose a language to visit OneSource.</p><nav aria-label="Language selection"><a class="button primary" href="/en/" lang="en">English</a> <a class="button secondary" href="/zh/" lang="zh-Hant">繁體中文</a></nav></main></body></html>\n`]);
  pages.push(['.nojekyll', '']);
  await rm(output, { recursive: true, force: true });
  await mkdir(path.join(output, 'medias'), { recursive: true });
  for (const [name, content] of pages) {
    await mkdir(path.dirname(path.join(output, name)), { recursive: true });
    await writeFile(path.join(output, name), content);
  }
  const assets = ['styles.css', 'script.js', 'site/language.js', 'site/install.js', 'site/install-redirect.js', ...['logo-320.webp', 'logo-640.webp', 'logo-960.webp', 'logo.png', 'favicon.png', 'apple-touch-icon.png', 'OneSourceSocialCard.png'].map(name => `medias/${name}`)];
  for (const asset of assets) await copyFile(path.join(root, asset), path.join(output, asset.startsWith('site/') ? path.basename(asset) : asset));
  return output;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  await buildSite();
  console.log('Static website built in dist/');
}

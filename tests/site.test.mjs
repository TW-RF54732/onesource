import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile, access, readdir } from 'node:fs/promises';
import path from 'node:path';
import vm from 'node:vm';
import { root, validateConfig, render } from '../scripts/build-site.mjs';
import { createPreviewServer } from '../scripts/preview-site.mjs';

const read = name => readFile(path.join(root, name), 'utf8');
const origin = 'https://onesource.zynimous.dev';
const languageScript = await read('site/language.js');
function languageContext({ pathname = '/', languages = ['en-US'], language = 'en-US', saved = null, brokenStorage = false, query = '', hash = '' } = {}) {
  let destination;
  const stored = [];
  const links = ['en', 'zh'].map(value => ({ dataset: { language: value }, href: `/${value}/`, addEventListener(name, callback) { this[name] = callback; } }));
  const context = {
    location: { pathname, search: query, hash, replace(value) { destination = value; } },
    navigator: { languages, language },
    localStorage: { getItem() { if (brokenStorage) throw Error('Blocked'); return saved; }, setItem(key, value) { if (brokenStorage) throw Error('Blocked'); stored.push([key, value]); } },
    document: { addEventListener(name, callback) { callback(); }, querySelectorAll() { return links; } }
  };
  vm.runInNewContext(languageScript, context);
  return { destination, stored, links };
}

test('root preference resolution and fallback', () => {
  for (const [options, expected] of [
    [{}, '/en/'], [{ languages: ['zh-TW'] }, '/zh/'], [{ languages: ['zh-CN'] }, '/zh/'],
    [{ languages: ['fr-FR', 'zh-HK', 'en-US'] }, '/zh/'], [{ languages: ['en-GB', 'zh-TW'] }, '/en/'],
    [{ languages: ['fr-FR'] }, '/en/'], [{ languages: [], language: 'zh-TW' }, '/zh/'],
    [{ saved: 'en', languages: ['zh-TW'] }, '/en/'], [{ saved: 'zh', languages: ['en-US'] }, '/zh/'],
    [{ saved: 'zh-Hant' }, '/zh/'], [{ saved: 'invalid' }, '/en/'], [{ brokenStorage: true, languages: ['zh-TW'] }, '/zh/'],
    [{ query: '?ref=docs', hash: '#install' }, '/en/?ref=docs#install'], [{ pathname: '/index.html' }, '/en/']
  ]) assert.equal(languageContext(options).destination, expected, JSON.stringify(options));
});

test('explicit language and missing URLs never redirect', () => {
  for (const pathname of ['/en/', '/zh/', '/en', '/zh', '/zh/index.html', '/missing']) {
    assert.equal(languageContext({ pathname, saved: 'zh', languages: ['zh-TW'] }).destination, undefined);
  }
});

test('native switches preserve anchors/query and remember manual choice', () => {
  const { links, stored } = languageContext({ pathname: '/en/', hash: '#install', query: '?ref=github' });
  assert.equal(links[1].href, '/zh/?ref=github#install');
  links[1].click();
  assert.deepEqual(stored, [['onesource-language', 'zh']]);
  const blocked = languageContext({ pathname: '/en/', brokenStorage: true, hash: '#about' });
  assert.doesNotThrow(() => blocked.links[1].click());
  assert.equal(blocked.links[1].href, '/zh/#about');
});

test('build rejects invalid origin and unresolved translations', () => {
  validateConfig({ origin, repository: 'https://github.com/TW-RF54732/onesource' });
  for (const bad of ['http://example.com', origin + '/', origin + '/en/', 'not-a-url']) {
    assert.throws(() => validateConfig({ origin: bad, repository: 'https://github.com/TW-RF54732/onesource' }));
  }
  assert.throws(() => render('<p>{{missing}}</p>', {}));
  assert.throws(() => render('<p>{{missing-key}}</p>', {}));
  assert.equal(render('<p>{{text}}</p>', { text: '<script>&"' }), '<p>&lt;script&gt;&amp;&quot;</p>');
});

test('static HTML includes localized SEO, software facts and working local resources', async () => {
  for (const [route, locale] of [['', 'en'], ['en', 'en'], ['zh', 'zh']]) {
    const html = await read(`dist/${route ? route + '/' : ''}index.html`);
    const copy = JSON.parse(await read(`site/${locale}.json`));
    assert.ok(html.includes(`<html lang="${locale === 'en' ? 'en' : 'zh-Hant'}">`));
    assert.equal((html.match(/<h1\b/g) || []).length, 1);
    assert.equal((html.match(/rel="canonical"/g) || []).length, 1);
    assert.ok(html.includes(`rel="canonical" href="${origin}/${locale}/"`));
    assert.ok(html.includes(`<title>${copy.title}</title>`));
    assert.ok(html.includes(copy.intro));
    assert.ok(html.includes(copy.installTitle));
    assert.ok(html.includes('cargo install onesource'));
    assert.ok(!/data-i18n|{{|aria-pressed|<button/.test(html));
    for (const [tag, target] of [['en', '/en/'], ['zh-Hant', '/zh/'], ['x-default', '/']]) {
      assert.ok(html.includes(`hreflang="${tag}" href="${origin}${target}"`));
    }
    for (const name of ['og:title', 'og:description', 'og:url', 'og:locale', 'og:locale:alternate', 'og:image', 'og:image:width', 'og:image:height', 'og:image:alt', 'twitter:card', 'twitter:image:alt']) assert.ok(html.includes(`="${name}"`), name);
    assert.ok(html.includes('name="twitter:card" content="summary_large_image"'));
    assert.ok(html.includes(`href="/${locale}/" data-language="${locale}" lang="${locale === 'en' ? 'en' : 'zh-Hant'}" aria-current="true"`));
    const data = JSON.parse(html.match(/<script type="application\/ld\+json">([\s\S]*?)<\/script>/)[1]);
    assert.deepEqual(data['@graph'].map(item => item['@type']), ['WebSite', 'WebPage', 'SoftwareApplication']);
    assert.equal(data['@graph'][1].url, `${origin}/${locale}/`);
    assert.equal(data['@graph'][2].offers.price, '0');
    assert.equal(data['@graph'][2].aggregateRating, undefined);
    assert.equal(data['@graph'][2].review, undefined);
    assert.ok(html.includes(locale === 'en' ? 'https://github.com/TW-RF54732/onesource#readme' : 'https://github.com/TW-RF54732/onesource/blob/main/README_zh.md'));
    const ids = new Set([...html.matchAll(/\bid="([^"]+)"/g)].map(match => match[1]));
    for (const match of html.matchAll(/(?:src|href)="([^"#]+)(?:#[^"]*)?"/g)) {
      if (match[1].startsWith('/')) {
        const local = match[1].endsWith('/') ? match[1] + 'index.html' : match[1];
        await access(path.join(root, 'dist', local));
      }
    }
    for (const match of html.matchAll(/href="#([^"]+)"/g)) assert.ok(ids.has(match[1]), match[1]);
    for (const match of html.matchAll(/aria-labelledby="([^"]+)"/g)) assert.ok(ids.has(match[1]), match[1]);
    for (const asset of ['logo-320.webp', 'logo-640.webp', 'logo-960.webp']) await access(path.join(root, 'dist/medias', asset));
  }
});

test('sitemap contains only canonical pages and matching language alternates', async () => {
  const sitemap = await read('dist/sitemap.xml');
  assert.deepEqual([...sitemap.matchAll(/<loc>([^<]+)<\/loc>/g)].map(match => match[1]), [`${origin}/en/`, `${origin}/zh/`]);
  assert.equal((sitemap.match(/hreflang="x-default"/g) || []).length, 2);
  assert.ok(!sitemap.includes('lastmod'));
  assert.ok((await read('dist/robots.txt')).includes(`Sitemap: ${origin}/sitemap.xml`));
  assert.ok((await read('dist/404.html')).includes('content="noindex"'));
  assert.ok(!(await read('dist/404.html')).includes('language.js'));
  assert.deepEqual((await readdir(path.join(root, 'dist'))).sort(), ['.nojekyll', '404.html', 'en', 'index.html', 'language.js', 'medias', 'robots.txt', 'script.js', 'sitemap.xml', 'styles.css', 'zh']);
});

test('preview serves real 404 and directory redirects', async () => {
  const server = createPreviewServer();
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  try {
    const base = `http://127.0.0.1:${server.address().port}`;
    for (const route of ['/en/', '/zh/', '/robots.txt', '/sitemap.xml', '/medias/logo-320.webp']) assert.equal((await fetch(base + route)).status, 200);
    const redirect = await fetch(base + '/en?ref=test', { redirect: 'manual' });
    assert.equal(redirect.status, 301);
    assert.equal(redirect.headers.get('location'), '/en/?ref=test');
    const missing = await fetch(base + '/does-not-exist');
    assert.equal(missing.status, 404);
    assert.ok((await missing.text()).includes('Page not found'));
  } finally {
    await new Promise(resolve => server.close(resolve));
  }
});

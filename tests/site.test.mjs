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
  const links = ['en', 'zh'].map(value => ({ dataset: { language: value }, href: `/${value}/${pathname.replace(/^\/(en|zh)\//, '').replace(/^\/$/, '')}`, getAttribute() { return this.href; }, addEventListener(name, callback) { this[name] = callback; } }));
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
  for (const pathname of ['/en/', '/zh/', '/en', '/zh', '/zh/index.html', '/en/install/', '/zh/install/', '/en/install', '/missing']) {
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
    assert.ok(html.includes(`/${locale}/docs/`));
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
  assert.deepEqual([...sitemap.matchAll(/<loc>([^<]+)<\/loc>/g)].map(match => match[1]), ['', 'docs/', ...['install', 'getting-started', 'cli', 'filtering', 'tree', 'profiles', 'explain', 'output', 'update', 'troubleshooting'].map(slug => `docs/${slug}/`)].flatMap(suffix => ['en', 'zh'].map(locale => `${origin}/${locale}/${suffix}`)));
  assert.equal((sitemap.match(/hreflang="x-default"/g) || []).length, 24);
  assert.ok(!sitemap.includes('lastmod'));
  assert.ok((await read('dist/robots.txt')).includes(`Sitemap: ${origin}/sitemap.xml`));
  assert.ok((await read('dist/404.html')).includes('content="noindex"'));
  assert.ok(!(await read('dist/404.html')).includes('language.js'));
  assert.deepEqual((await readdir(path.join(root, 'dist'))).sort(), ['.nojekyll', '404.html', 'en', 'index.html', 'install-redirect.js', 'install.js', 'language.js', 'medias', 'robots.txt', 'script.js', 'sitemap.xml', 'styles.css', 'zh']);
});

test('preview serves real 404 and directory redirects', async () => {
  const server = createPreviewServer();
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  try {
    const base = `http://127.0.0.1:${server.address().port}`;
    for (const route of ['/en/', '/zh/', '/en/install/', '/zh/install/', '/robots.txt', '/sitemap.xml', '/medias/logo-320.webp', ...['en', 'zh'].flatMap(locale => docSlugs.map(slug => `/${locale}/docs/${slug ? slug + '/' : ''}`))]) assert.equal((await fetch(base + route)).status, 200);
    const redirect = await fetch(base + '/en?ref=test', { redirect: 'manual' });
    assert.equal(redirect.status, 301);
    assert.equal(redirect.headers.get('location'), '/en/?ref=test');
    for (const locale of ['en', 'zh']) {
      const guideRedirect = await fetch(base + `/${locale}/install?ref=test`, { redirect: 'manual' });
      assert.equal(guideRedirect.status, 301);
      assert.equal(guideRedirect.headers.get('location'), `/${locale}/install/?ref=test`);
    }
    const missing = await fetch(base + '/does-not-exist');
    assert.equal(missing.status, 404);
    assert.ok((await missing.text()).includes('Page not found'));
  } finally {
    await new Promise(resolve => server.close(resolve));
  }
});

test('guide language links preserve page, query and anchor in both directions', () => {
  for (const [source, target] of [['en', 'zh'], ['zh', 'en']]) {
    for (const brokenStorage of [false, true]) {
      const result = languageContext({ pathname: `/${source}/install/`, query: '?ref=guide', hash: '#verify', brokenStorage });
      assert.equal(result.destination, undefined);
      const link = result.links.find(link => link.dataset.language === target);
      assert.equal(link.href, `/${target}/install/?ref=guide#verify`);
      assert.doesNotThrow(() => link.click());
    }
  }
});

test('installation pages are complete static guides with their own SEO and valid navigation', async () => {
  for (const locale of ['en', 'zh']) {
    const html = await read(`dist/${locale}/docs/install/index.html`);
    const copy = JSON.parse(await read(`site/${locale}.json`));
    assert.equal((html.match(/<h1\b/g) || []).length, 1);
    assert.ok(html.includes(`<title>${copy.guideTitle}</title>`));
    assert.ok(html.includes(copy.guideDescription));
    assert.ok(html.includes(`rel="canonical" href="${origin}/${locale}/docs/install/"`));
    for (const [lang, target] of [['en', 'en'], ['zh-Hant', 'zh'], ['x-default', 'en']]) {
      assert.ok(html.includes(`hreflang="${lang}" href="${origin}/${target}/docs/install/"`));
    }
    assert.ok(html.includes(`href="/${locale}/"`));
    for (const target of ['en', 'zh']) assert.ok(html.includes(`href="/${target}/docs/install/" data-language="${target}"`));
    const jsonld = JSON.parse(html.match(/<script type="application\/ld\+json">([\s\S]*?)<\/script>/)[1]);
    assert.equal(jsonld['@graph'][1].url, `${origin}/${locale}/docs/install/`);
    assert.equal(jsonld['@graph'][1].name, copy.guideTitle);
    assert.ok(!html.includes('/script.js'));
    assert.ok(!/{{|data-i18n/.test(html));
    for (const command of ['cargo install onesource', 'onesource --version', 'onesource --help', 'onesource --dry-run', 'onesource --copy', 'onesource update', 'cargo install onesource --force', 'chmod +x onesource']) assert.ok(html.includes(command), command);
    for (const asset of ['onesource.exe', 'onesource-linux', 'onesource-macos', 'onesource-macos-arm64']) assert.ok(html.includes(asset));
    const ids = [...html.matchAll(/\bid="([^"]+)"/g)].map(match => match[1]);
    assert.equal(ids.length, new Set(ids).size);
    for (const match of html.matchAll(/href="#([^"]+)"|aria-labelledby="([^"]+)"/g)) assert.ok(ids.includes(match[1] || match[2]));
    for (const match of html.matchAll(/(?:src|href)="(\/[^"#]+)"/g)) {
      await access(path.join(root, 'dist', match[1].endsWith('/') ? match[1] + 'index.html' : match[1]));
    }
    const home = await read(`dist/${locale}/index.html`);
    assert.ok(home.includes(`href="/${locale}/docs/install/"`));
    assert.ok(html.includes('class="copy-command" hidden'));
    assert.equal((html.match(/<details>/g) || []).length, 5);
  }
});

test('copy controls copy the displayed command and recover from denied clipboard access', async () => {
  const script = await read('site/install.js');
  for (const outcome of ['success', 'denied', 'unavailable']) {
    let copied;
    const button = { hidden: true, disabled: false, addEventListener(name, callback) { this[name] = callback; } };
    const status = { textContent: '' };
    const code = { textContent: 'onesource --version\nonesource --help' };
    const panel = { querySelector(selector) { return selector === 'code' ? code : selector === '.copy-status' ? status : button; } };
    const content = { dataset: { copiedLabel: 'Copied', copyFailed: 'Copy manually' }, querySelectorAll() { return [panel]; } };
    const navigator = outcome === 'unavailable' ? {} : { clipboard: { async writeText(value) { if (outcome === 'denied') throw Error('Denied'); copied = value; } } };
    vm.runInNewContext(script, { navigator, document: { querySelector() { return content; } } });
    if (outcome === 'unavailable') {
      assert.equal(button.hidden, true);
      assert.equal(button.click, undefined);
    } else {
      assert.equal(button.hidden, false);
      const pending = button.click();
      assert.equal(button.disabled, true);
      await pending;
      assert.equal(button.disabled, false);
      assert.equal(status.textContent, outcome === 'success' ? 'Copied' : 'Copy manually');
      assert.equal(copied, outcome === 'success' ? code.textContent : undefined);
    }
  }
});

const docSlugs = ['', 'install', 'getting-started', 'cli', 'filtering', 'tree', 'profiles', 'explain', 'output', 'update', 'troubleshooting'];
test('all docs have matching sections, complete static navigation and valid local links/anchors', async () => {
  for (const slug of docSlugs) {
    const idsByLanguage = [];
    for (const locale of ['en', 'zh']) {
      const suffix = `docs/${slug ? slug + '/' : ''}`;
      const html = await read(`dist/${locale}/${suffix}index.html`);
      const ids = [...html.matchAll(/\bid="([^"]+)"/g)].map(m => m[1]);
      idsByLanguage.push(ids);
      assert.equal(ids.length, new Set(ids).size);
      assert.equal((html.match(/<h1\b/g) || []).length, 1);
      assert.ok(html.includes(`rel="canonical" href="${origin}/${locale}/${suffix}"`));
      assert.ok(html.includes(`hreflang="x-default" href="${origin}/en/${suffix}"`));
      assert.ok(html.includes('<details class="mobile-index">'));
      assert.ok(html.includes('aria-current="page"'));
      assert.ok(html.includes('id="content"'));
      for (const language of ['en', 'zh']) assert.ok(html.includes(`href="/${language}/${suffix}" data-language="${language}"`));
      for (const m of html.matchAll(/(?:href|src)="(\/[^"?#]*)(?:\?[^"#]*)?(?:#([^"]*))?"/g)) {
        const target = m[1].endsWith('/') ? m[1] + 'index.html' : m[1];
        await access(path.join(root, 'dist', target));
        if (m[2]) assert.ok((await read(`dist${target}`)).includes(`id="${m[2]}"`));
      }
      for (const m of html.matchAll(/href="#([^"]+)"/g)) assert.ok(ids.includes(m[1]), `${locale}/${slug}#${m[1]}`);
      assert.ok(!/{{|data-i18n/.test(html));
    }
    assert.deepEqual(idsByLanguage[0], idsByLanguage[1], slug);
  }
});

test('old installation URLs redirect with query/hash and provide a no-JavaScript link', async () => {
  const script = await read('site/install-redirect.js');
  for (const locale of ['en', 'zh']) {
    const html = await read(`dist/${locale}/install/index.html`);
    assert.ok(html.includes(`id="install-destination" href="/${locale}/docs/install/"`));
    assert.ok(html.includes('content="noindex"'));
    assert.ok(!html.includes('id="windows"'));
    let destination;
    vm.runInNewContext(script, {
      document: { getElementById() { return { getAttribute() { return `/${locale}/docs/install/`; } }; } },
      location: { search: '?ref=old', hash: '#verify', replace(value) { destination = value; } }
    });
    assert.equal(destination, `/${locale}/docs/install/?ref=old#verify`);
  }
  assert.ok(!(await read('dist/sitemap.xml')).includes('<loc>' + origin + '/en/install/'));
});

test('docs language switches preserve each route, query and common anchor', () => {
  for (const slug of docSlugs) for (const source of ['en', 'zh']) {
    const target = source === 'en' ? 'zh' : 'en';
    const suffix = `docs/${slug ? slug + '/' : ''}`;
    const result = languageContext({ pathname: `/${source}/${suffix}`, query: '?ref=docs', hash: '#examples' });
    assert.equal(result.links.find(link => link.dataset.language === target).href, `/${target}/${suffix}?ref=docs#examples`);
  }
});

test('homepage hero, navigation, download and release-note links', async () => {
  for (const locale of ['en', 'zh']) {
    const html = await read(`dist/${locale}/index.html`);
    assert.match(html, new RegExp(`class="button secondary" href="/${locale}/docs/install/"`));
    const header = html.match(/<header[\s\S]*?<\/header>/)[0];
    assert.ok(header.includes(`href="/${locale}/docs/"`));
    assert.ok(header.includes('href="https://github.com/TW-RF54732/onesource"'));
    assert.ok(html.includes(locale === 'en' ? 'Download portable binaries from GitHub Releases' : '從 GitHub Releases 下載免安裝執行檔'));
    assert.ok(html.includes(locale === 'en' ? 'View release notes' : '查看釋出版本日誌'));
  }
});

test('documented CLI options, profile fields, blacklist and decisions cover implementation', async () => {
  const config = await read('src/configs.rs');
  const filter = await read('src/filter_utils.rs');
  const explain = await read('src/explain.rs');
  for (const locale of ['en', 'zh']) {
    const cli = await read(`dist/${locale}/docs/cli/index.html`);
    const profiles = await read(`dist/${locale}/docs/profiles/index.html`);
    const filtering = await read(`dist/${locale}/docs/filtering/index.html`);
    const diagnostics = await read(`dist/${locale}/docs/explain/index.html`);
    const args = config.split('pub struct Args {')[1].split('#[derive(Subcommand')[0];
    for (const m of args.matchAll(/pub (\w+):/g)) {
      if (m[1] !== 'command' && m[1] !== 'desc') assert.ok(cli.includes(`<code>${m[1]}</code>`), m[1]);
    }
    const fields = config.split('pub struct ProfileConfig {')[1].split('impl ProfileConfig')[0];
    for (const m of fields.matchAll(/pub (\w+):/g)) assert.ok(profiles.includes(`&quot;${m[1]}&quot;`) || profiles.includes(`"${m[1]}"`), m[1]);
    for (const m of filter.split('pub struct FileFilter')[0].matchAll(/"([^"]+)"/g)) assert.ok(filtering.includes(m[1]), m[1]);
    const decisions = explain.split('fn decision_text')[1].split('#[cfg(test)]')[0];
    for (const m of decisions.matchAll(/"([^"]+)"/g)) assert.ok(diagnostics.includes(m[1]), m[1]);
    for (const name of ['list','show','create','update','delete','rename','desc','ls','rm']) assert.ok(profiles.includes(`profile ${name}`), name);
  }
});

test('tutorial commands match across languages and output/configuration never get copy controls', async () => {
  const en = JSON.parse(await read('site/docs.en.json'));
  const zh = JSON.parse(await read('site/docs.zh.json'));
  const commands = page => [...page.sections.map(section => section.html).join('').matchAll(/<pre data-kind="command"><code>([\s\S]*?)<\/code><\/pre>/g)].map(m => m[1]);
  for (const slug of Object.keys(en).filter(slug => slug !== 'cli' && slug !== '')) {
    assert.deepEqual(commands(en[slug]), commands(zh[slug]), `${slug} command sequences`);
    for (const locale of ['en', 'zh']) {
      const html = await read(`dist/${locale}/docs/${slug}/index.html`);
      for (const panel of html.matchAll(/<div class="guide-command"><div class="code-heading">[\s\S]*?<\/div>(<pre[^>]*>)/g)) assert.ok(!/data-kind="(?:output|config|reference)"/.test(panel[1]));
      for (const kind of ['output', 'config', 'reference']) {
        const examples = [...html.matchAll(new RegExp(`<div class="doc-example doc-${kind}">([\\s\\S]*?)</pre></div>`, 'g'))];
        for (const example of examples) assert.ok(!example[1].includes('copy-command'));
        assert.equal((html.match(new RegExp(`<pre data-kind="${kind}">`, 'g')) || []).length, (html.match(new RegExp(`class="doc-example doc-${kind}"`, 'g')) || []).length);
      }
    }
  }
});

(() => {
  'use strict';
  const hero = document.querySelector('.hero');
  const layout = document.querySelector('.hero-layout');
  const logo = document.querySelector('.logo-card');
  const field = document.querySelector('.file-field');
  const hoverElements = hero.querySelectorAll('.logo-card img, .hero-copy .button');
  const hoverText = hero.querySelectorAll('.hero-copy h1, .hero-copy .tagline');
  const hoverRange = document.createRange();
  const pointer = matchMedia('(hover: hover) and (pointer: fine)');
  const reducedMotion = matchMedia('(prefers-reduced-motion: reduce)');
  const filenames = [
    'main.rs', 'index.html', 'styles.css', 'script.js', 'app.tsx', 'package.json',
    'Cargo.toml', 'README.md', 'lib.rs', 'config.ts', 'utils.ts', 'server.js',
    'routes.ts', 'schema.sql', 'main.py', 'Dockerfile', 'Makefile', '.gitignore',
    'vite.config.ts', 'tsconfig.json', 'api.ts', 'types.ts', 'layout.tsx', 'page.tsx',
    'test.rs', 'settings.json', 'go.mod', 'main.go', 'App.vue', 'index.ts',
    'requirements.txt', 'theme.css', 'components.tsx', 'helpers.py', 'build.sh', 'data.json',
    'hooks.ts', 'store.ts', 'router.ts', 'client.ts', 'constants.ts', 'parser.rs',
    'config.rs', 'models.rs', 'handlers.go', 'utils.go', 'app.py', 'views.py',
    'models.py', 'pyproject.toml', 'compose.yaml', 'LICENSE', 'CHANGELOG.md',
    'robots.txt', 'manifest.json', 'tokens.css', 'reset.css', 'Header.tsx',
    'Footer.tsx', 'Button.tsx', 'Card.vue', 'useTheme.ts', 'eslint.config.js', 'test.py'
  ];
  const cards = [];
  let width = 0, height = 0, cardWidth = 120, cardHeight = 80;
  let frame = 0, lastTime = 0, elapsed = 0;
  let visible = true, stacked = false, pointerActive = false;
  let mouseX = 0, mouseY = 0, stackX = 0, stackY = 0;
  let pointerDirty = false, fieldLeft = 0, fieldTop = 0;

  function shuffled(values) {
    const result = [...values];
    for (let i = result.length - 1; i > 0; i--) {
      const j = Math.floor(Math.random() * (i + 1));
      [result[i], result[j]] = [result[j], result[i]];
    }
    return result;
  }
  const clamp = (value, low, high) => Math.max(low, Math.min(Math.max(low, high), value));

  function scatterTargets() {
    const columns = Math.max(1, Math.round(Math.sqrt(cards.length * width / Math.max(height, 1))));
    const rows = Math.ceil(cards.length / columns);
    const cells = shuffled(Array.from({ length: columns * rows }, (_, i) => i));
    cards.forEach((card, i) => {
      const cell = cells[i];
      card.anchorX = clamp(((cell % columns) + .3 + Math.random() * .4) * width / columns - cardWidth / 2, 12, width - cardWidth - 12);
      card.anchorY = clamp((Math.floor(cell / columns) + .3 + Math.random() * .4) * height / rows - cardHeight / 2, 12, height - cardHeight - 12);
      card.phase = Math.random() * Math.PI * 2;
      card.speed = .24 + Math.random() * .32;
      card.rotation = (Math.random() - .5) * 18;
    });
  }

  function paint(card) {
    card.node.style.transform = `translate3d(${card.x.toFixed(2)}px, ${card.y.toFixed(2)}px, 0) rotate(${card.angle.toFixed(2)}deg)`;
  }

  function measure() {
    pointerDirty = true;
    width = document.documentElement.clientWidth;
    height = hero.getBoundingClientRect().height;
    hero.style.setProperty('--field-width', `${width}px`);
    cardWidth = width <= 650 ? 100 : 120;
    cardHeight = width <= 650 ? 70 : 80;
    const count = width <= 650 ? 16 : width <= 900 ? 32 : 48;
    if (cards.length !== count) {
      const names = shuffled(filenames);
      while (cards.length > count) cards.pop().node.remove();
      const used = new Set(cards.map(card => card.name));
      const available = names.filter(name => !used.has(name));
      while (cards.length < count) {
        const name = available.pop();
        const node = document.createElement('div');
        node.className = 'file-card';
        const heading = document.createElement('div');
        heading.className = 'file-card-heading';
        const icon = document.createElement('span');
        icon.className = 'file-card-icon';
        const label = document.createElement('span');
        label.className = 'file-card-name';
        label.textContent = name;
        heading.append(icon, label);
        const lines = document.createElement('div');
        lines.className = 'file-card-lines';
        for (let i = 0; i < 3; i++) lines.append(document.createElement('span'));
        node.append(heading, lines);
        node.style.setProperty('--file-opacity', (.25 + Math.random() * .1).toFixed(2));
        field.append(node);
        cards.push({ node, name, x: null, y: null, angle: 0 });
      }
    }
    scatterTargets();
    cards.forEach(card => {
      if (card.x === null || reducedMotion.matches) {
        card.x = card.anchorX;
        card.y = card.anchorY;
        card.angle = card.rotation;
      } else {
        card.x = clamp(card.x, 12, width - cardWidth - 12);
        card.y = clamp(card.y, 12, height - cardHeight - 12);
      }
      paint(card);
    });
    schedule();
  }

  function canAnimate() {
    return visible && !document.hidden && !reducedMotion.matches;
  }
  function schedule() {
    if (!frame && canAnimate()) frame = requestAnimationFrame(animate);
  }
  function pause() {
    cancelAnimationFrame(frame);
    frame = 0;
    lastTime = 0;
  }
  function reset() {
    if (stacked) scatterTargets();
    stacked = false;
    pointerActive = false;
    pointerDirty = false;
    hero.classList.remove('pointer-active');
    logo.style.removeProperty('--tilt-x');
    logo.style.removeProperty('--tilt-y');
    schedule();
  }

  function updatePointer() {
    const fieldRect = field.getBoundingClientRect();
    fieldLeft = fieldRect.left;
    fieldTop = fieldRect.top;
    // Grid columns stretch across the Hero. Use the visible content instead,
    // including text ranges so the headings' empty block width is excluded.
    const bounds = [...hoverElements].map(element => element.getBoundingClientRect());
    hoverText.forEach(element => {
      hoverRange.selectNodeContents(element);
      bounds.push(hoverRange.getBoundingClientRect());
    });
    const contentBounds = bounds.filter(rect => rect.width > 0 && rect.height > 0);
    const padding = 16;
    const inside = contentBounds.length > 0 &&
      mouseX >= Math.min(...contentBounds.map(rect => rect.left)) - padding &&
      mouseX <= Math.max(...contentBounds.map(rect => rect.right)) + padding &&
      mouseY >= Math.min(...contentBounds.map(rect => rect.top)) - padding &&
      mouseY <= Math.max(...contentBounds.map(rect => rect.bottom)) + padding;
    if (inside && !stacked) {
      stackX = mouseX - fieldRect.left + 20;
      stackY = mouseY - fieldRect.top + 20;
      stacked = true;
    } else if (!inside && stacked) {
      stacked = false;
      scatterTargets();
    }
  }

  function animate(time) {
    frame = 0;
    if (!canAnimate()) { lastTime = 0; return; }
    const dt = lastTime ? Math.min((time - lastTime) / 1000, .05) : 1 / 60;
    lastTime = time;
    elapsed += dt;
    const blend = 1 - Math.exp(-dt / .18);
    if (pointerActive && pointerDirty) {
      updatePointer();
      pointerDirty = false;
      const rect = hero.getBoundingClientRect();
      const x = mouseX - rect.left;
      const y = mouseY - rect.top;
      hero.style.setProperty('--glow-x', `${x}px`);
      hero.style.setProperty('--glow-y', `${y}px`);
      logo.style.setProperty('--tilt-x', `${(.5 - y / rect.height) * 5}deg`);
      logo.style.setProperty('--tilt-y', `${(x / rect.width - .5) * 5}deg`);
    }
    if (stacked) {
      const offset = (cards.length - 1) * 2;
      const x = clamp(mouseX - fieldLeft + 20, 12, width - cardWidth - offset - 12);
      const y = clamp(mouseY - fieldTop + 20, 12, height - cardHeight - offset - 12);
      const follow = 1 - Math.exp(-dt / .09);
      stackX += (x - stackX) * follow;
      stackY += (y - stackY) * follow;
    }
    cards.forEach((card, i) => {
      const phase = elapsed * card.speed + card.phase;
      const x = stacked ? stackX + i * 2 : clamp(card.anchorX + Math.sin(phase) * 22, 12, width - cardWidth - 12);
      const y = stacked ? stackY + i * 2 : clamp(card.anchorY + Math.cos(phase * .83) * 18, 12, height - cardHeight - 12);
      const angle = stacked ? 0 : card.rotation + Math.sin(phase * .7) * 4;
      card.x += (x - card.x) * blend;
      card.y += (y - card.y) * blend;
      card.angle += (angle - card.angle) * blend;
      paint(card);
    });
    schedule();
  }

  hero.addEventListener('pointermove', event => {
    if (!pointer.matches || reducedMotion.matches || event.pointerType !== 'mouse') { reset(); return; }
    mouseX = event.clientX;
    mouseY = event.clientY;
    pointerActive = true;
    pointerDirty = true;
    hero.classList.add('pointer-active');
    schedule();
  });
  layout.addEventListener('pointerleave', () => {
    if (stacked) { stacked = false; scatterTargets(); }
  });
  hero.addEventListener('pointerleave', reset);
  hero.addEventListener('pointercancel', reset);
  window.addEventListener('blur', reset);
  window.addEventListener('scroll', reset, { passive: true });
  pointer.addEventListener('change', reset);
  reducedMotion.addEventListener('change', () => { reset(); pause(); measure(); });
  document.addEventListener('visibilitychange', () => {
    reset();
    if (document.hidden) pause(); else schedule();
  });
  new ResizeObserver(measure).observe(hero);
  new IntersectionObserver(entries => {
    visible = entries[0].isIntersecting;
    if (visible) schedule(); else { reset(); pause(); }
  }).observe(hero);
  measure();
})();

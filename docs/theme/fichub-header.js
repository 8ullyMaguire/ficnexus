// Inject a FicNexus-native header into every docs page so /docs feels like
// part of the archive instead of a separate site. Mirrors the AO3-parity
// archive header: #900 red navbar with the site brand, white text links,
// #ddd hover boxes, and the same primary nav as the app.
(function () {
  var bar = document.createElement('div');
  bar.className = 'fnx-docs-nav';
  bar.setAttribute('role', 'navigation');
  bar.setAttribute('aria-label', 'FicNexus');
  bar.style.cssText =
    'background:#900 url("/images/red-ao3.png");padding:0;' +
    'box-shadow:inset 0 -6px 10px rgba(0,0,0,.35),1px 1px 3px -1px rgba(0,0,0,.25),inset 0 -1px 0 rgba(0,0,0,.85);' +
    'position:sticky;top:0;z-index:100;font-family:Georgia,"Times New Roman",serif;';

  var path = window.location.pathname
    .replace(/^\/docs\/?/, '').replace(/\.html.*$/, '') || 'index';

  // Contextual "Try it" deep links per docs page (unchanged behavior).
  var tryLinks = {
    'searching': ['/search', 'Try it: open Search'],
    'downloading': ['/', 'Try it: download a fic'],
    'bookmarks': ['/search', 'Try it: find a story to bookmark'],
    'ratings': ['/leaderboard', 'Try it: see the leaderboard'],
    'comments': ['/search', 'Try it: open a story'],
    'recommendations': ['/', 'Try it: see your recommendations'],
    'features': ['/roadmap', 'Try it: vote on features'],
    'faq': ['/', 'Try it: go home'],
  };

  var items = [
    ['/', 'Fandoms'],
    ['/requests', 'Requests'],
    ['/forum', 'Forum'],
    ['/search', 'Search'],
    ['/docs', 'Docs'],
  ];
  var links = items.map(function (i) {
    var active = (i[1] === 'Docs' && path !== '') ||
      (i[0] !== '/' && window.location.pathname.indexOf(i[0]) === 0);
    return '<a href="' + i[0] + '" data-nav="' + i[1] + '"' +
      (active ? ' data-active="1"' : '') +
      ' style="display:inline-block;padding:0.429em 0.75em;color:#fff;font-size:1em;' +
      'text-decoration:none;white-space:nowrap;line-height:1.5;' +
      (active ? 'background:rgba(255,255,255,.25);border-radius:0;' : '') +
      '">' + i[1] + '</a>';
  }).join('');

  var tryIt = tryLinks[path]
    ? '<a href="' + tryLinks[path][0] + '" data-try="1" style="margin-left:auto;display:inline-block;' +
      'padding:0.429em 0.75em;color:#fff;text-decoration:none;font-size:0.95em;white-space:nowrap;">' +
      '▶ ' + tryLinks[path][1] + '</a>'
    : '';

  bar.innerHTML =
    '<div style="max-width:1100px;margin:0 auto;padding:0 1rem;display:flex;align-items:center;">' +
    '<a href="/" style="font-size:1.286em;font-weight:700;color:#fff;text-decoration:none;' +
    'padding:0.3em 0.75em 0.3em 0;line-height:1.5;border-bottom:none;">FicNexus</a>' +
    links + tryIt + '</div>';

  document.body.insertBefore(bar, document.body.firstChild);

  // Hover = AO3 #ddd box with dark ink, matching the archive nav contract.
  var style = document.createElement('style');
  style.textContent =
    '.fnx-docs-nav a:hover{background:#ddd;color:#111;border-radius:0.25em;}' +
    '.fnx-docs-nav a[data-try]:hover{background:transparent;color:#fff;}' +
    // Hide mdBook's own top nav chrome so only the FicNexus bar shows.
    '.page-header,.right-buttons,.theme-popover,.menu-button,#sidebar-toggle-button{display:none!important;}' +
    '.content{margin-top:0!important;}';
  document.head.appendChild(style);
})();

use axum::response::Html;

/// The entire user interface: a URL box, a format picker, and the links that
/// come back. There is no SPA, no account, no reader — a fichub.net-style
/// exporter needs one form.
///
/// `GET /` renders this; the JS below talks to `/api/epub` and `/api/formats`.
pub async fn landing() -> Html<&'static str> {
    Html(INDEX_HTML)
}

const INDEX_HTML: &str = r##"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>ficnexus — download a fic</title>
<style>
  :root { color-scheme: dark; }
  body { margin:0; font:16px/1.5 system-ui, sans-serif; background:#14161a; color:#e6e6e6;
         display:flex; min-height:100vh; align-items:center; justify-content:center; }
  main { width:min(680px, 92vw); padding:2rem; }
  h1 { font-size:1.6rem; margin:0 0 .25rem; }
  p.sub { color:#9aa4b2; margin:0 0 1.5rem; }
  form { display:flex; gap:.5rem; flex-wrap:wrap; }
  input[type=url] { flex:1 1 320px; padding:.7rem .8rem; border-radius:8px; border:1px solid #2c313a;
                    background:#1b1f25; color:inherit; font-size:1rem; }
  select, button { padding:.7rem .8rem; border-radius:8px; border:1px solid #2c313a;
                   background:#1b1f25; color:inherit; font-size:1rem; cursor:pointer; }
  button { background:#2563eb; border-color:#2563eb; font-weight:600; }
  button:disabled { opacity:.6; cursor:progress; }
  #out { margin-top:1.5rem; }
  .card { background:#1b1f25; border:1px solid #2c313a; border-radius:10px; padding:1rem 1.2rem; }
  .card h2 { font-size:1.05rem; margin:0 0 .3rem; }
  .meta { color:#9aa4b2; font-size:.9rem; }
  ul.formats { list-style:none; padding:0; margin:.8rem 0 0; display:flex; gap:.6rem; flex-wrap:wrap; }
  ul.formats a { display:inline-block; padding:.45rem .8rem; border-radius:8px;
                 background:#243043; color:#cfe0ff; text-decoration:none; font-size:.9rem; }
  .err { color:#fca5a5; }
  footer { margin-top:2rem; color:#6b7280; font-size:.8rem; }
</style>
</head>
<body>
<main>
  <h1>ficnexus</h1>
  <p class="sub">Paste a story URL. Get an EPUB, HTML, MOBI, PDF, AZW3, TXT or Markdown file.</p>
  <form id="f">
    <input id="q" type="url" required placeholder="https://archiveofourown.org/works/…" autofocus>
    <select id="fmt">
      <option value="epub">EPUB</option>
      <option value="html">HTML</option>
      <option value="txt">TXT</option>
      <option value="md">Markdown</option>
      <option value="mobi">MOBI</option>
      <option value="pdf">PDF</option>
      <option value="azw3">AZW3</option>
      <option value="kepub">KEPUB</option>
      <option value="docx">DOCX</option>
      <option value="fb2">FB2</option>
    </select>
    <button id="go" type="submit">Download</button>
  </form>
  <div id="out"></div>
  <footer>
    API: <a href="/api/" style="color:#93b4ff">/api/</a> ·
    formats: <a href="/api/formats" style="color:#93b4ff">/api/formats</a> ·
    health: <a href="/api/health" style="color:#93b4ff">/api/health</a>
  </footer>
</main>
<script>
const f = document.getElementById('f');
const out = document.getElementById('out');
const go = document.getElementById('go');

f.addEventListener('submit', async (e) => {
  e.preventDefault();
  const q = document.getElementById('q').value.trim();
  const fmt = document.getElementById('fmt').value;
  if (!q) return;
  go.disabled = true;
  out.innerHTML = '<div class="card meta">Working… scraping and building the file.</div>';
  try {
    const r = await fetch('/api/epub?format=' + encodeURIComponent(fmt) + '&q=' + encodeURIComponent(q));
    const j = await r.json();
    if (!r.ok || j.err !== 0) {
      out.innerHTML = '<div class="card err">' + (j.msg || j.error || ('HTTP ' + r.status)) + '</div>';
      return;
    }
    const m = j.meta || {};
    const urls = j.urls || {};
    const links = Object.entries(urls)
      .map(([k, v]) => '<a href="' + v + '" download>' + k.toUpperCase() + '</a>')
      .join('');
    out.innerHTML =
      '<div class="card"><h2>' + (m.title || j.info || 'Work') + '</h2>' +
      '<div class="meta">' + (m.author ? 'by ' + m.author + ' · ' : '') +
        (m.chapters ? m.chapters + ' chapters · ' : '') +
        (m.words ? m.words + ' words · ' : '') + (m.status || '') + '</div>' +
      '<ul class="formats">' + links + '</ul></div>';
  } catch (err) {
    out.innerHTML = '<div class="card err">' + err + '</div>';
  } finally {
    go.disabled = false;
  }
});
</script>
</body>
</html>
"##;

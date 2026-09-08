# How to Download Fics

Downloading a story takes about ten seconds. Here's the complete guide,
including formats and troubleshooting.

## The basic flow

1. **Copy a story URL** from your browser's address bar. Any of these work:
   ```
   https://archiveofourown.org/works/12345678
   https://www.fanfiction.net/s/12345678/1/Story-Name
   https://forums.spacebattles.com/threads/story-name.1234567/
   ```
2. **Go to the Download tab** — it's the first item in the menu.
3. **Paste the URL** into the box labeled "Paste a fanfiction URL"
   (Ctrl+V on Windows/Linux, Cmd+V on Mac).
4. **Click Download.**

After a few seconds the story's info and download buttons appear. Pick your
format and you're done.

## Choosing a format

| Format | Best for |
|--------|----------|
| **EPUB** | Kindle, Kobo, Apple Books, Google Play Books, phones, tablets |
| **HTML** | Reading in a browser, any device |
| **MOBI** | Older Kindle devices |
| **PDF** | Printing, sharing |
| **AZW3** | Newer Kindles (better than MOBI) |
| **TXT** | Simple text, works everywhere |
| **Markdown** | Developers, note-taking apps |
| **KEPUB** | Kobo e-readers (enhanced EPUB) |
| **DOCX** | Word processors, editing |

**Tip:** EPUB is the most portable — it works on almost everything.
MOBI/PDF/AZW3 need Calibre on the server and may take longer.

## Choosing formats once, downloading everywhere

Set your preferred formats in **Settings → Download formats** (or
`GET /api/formats` for the canonical list of 9). Author and series
bulk-downloads then bundle exactly those formats — one work × one format
streams directly, anything bigger arrives as a ZIP with per-work folders.

## Batch download: all works by an author

Click the **Download all works** button on any author's page (or use
`GET /api/download/author?url=<author_profile_url>`). Works for:

- **AO3**: `https://archiveofourown.org/users/{name}`
- **XenForo** (QQ, SpaceBattles, SV): `https://forum.questionablequesting.com/members/{name}.{id}/`

The endpoint lists all works by that author, scrapes each one, exports in
your configured formats, and returns either a single file (1 work × 1
format) or a ZIP with per-work subdirectories.

**Progress streaming:** For large batches (60+ works), the SSE endpoint
`GET /api/download/author/stream?url=<url>` emits real-time progress
events (`progress`, `complete`, `error`) so the UI can show a progress
bar. The frontend `BatchAuthorDownload` component uses this automatically.

**Note:** XenForo author discovery requires login credentials for
NSFW-gated forums. Store your QQ/SB/SV credentials in **Settings →
Site Credentials** and the batch download will use them automatically.

## Dual-mode input: URL *or* "Title by Author"

The Download tab accepts two kinds of input: a story URL (the classic flow
above), or plain text like `The Long Way Home by amusewithaview on AO3`.
The find-fic parser splits title/author/site and searches the archive first
— handy when you remember the story but not the link. See [Find a fic by
name](./searching.md#find-a-fic-by-name).

## Pro tips

- **Bookmarklet**: drag the "FicHub ↗" link to your bookmarks bar. Then when
  you're on any story page, click it to jump straight to FicHub with the URL
  already filled in.
- **Multi-chapter stories**: FicHub grabs *all* chapters automatically.
- **Works offline**: downloaded files are self-contained — no internet needed
  to read them later.
- **Downloads are logged & rate-limited**: normal readers never notice, but
  the service rate-limits downloads per IP to keep bots out. If you're
  downloading many fics in a row, add a few seconds between them.

## Troubleshooting

**"Something went wrong"**
- Double-check the URL — it must be a valid, public story link.
- Some stories are restricted (locked to logged-in users on AO3).
- Try again in a minute — the source site might be busy.

**"Not available for download"**
- The story may be restricted by the author, or blocked by the source site.
- FicHub can only download public stories.

**"Unknown Title"**
- The source site may be blocking the request or busy. Wait a minute and retry.

---

*Next: [Finding Great Stories with Search](./searching.md)*

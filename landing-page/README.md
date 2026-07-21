# WinReclaim landing page

Static, dependency-free product site for WinReclaim.

## Local preview

From the repository root:

```powershell
cd landing-page
python -m http.server 4173
```

Open `http://localhost:4173`.

Opening `index.html` directly is not recommended because the Content Security Policy and absolute asset paths are designed for a web server.

## Deploy to Vercel

1. Import `amaansyed27/WinReclaim` into Vercel.
2. Set **Root Directory** to `landing-page`.
3. Keep **Framework Preset** as `Other`.
4. Leave the build command empty.
5. Leave the output directory empty.
6. Deploy.

`vercel.json` provides clean URLs, security headers and convenience redirects:

- `/download` → latest GitHub Release
- `/source` → GitHub repository

The page calls GitHub's public releases API in the browser and updates all Windows setup and MSI links to the newest release assets. If the API is unavailable or no release exists yet, the buttons fall back to the repository's latest-release page.

## Files

- `index.html` — product content and semantic page structure
- `styles.css` — responsive visual system and application mockup
- `script.js` — mobile navigation, reveal motion and latest-release resolution
- `vercel.json` — Vercel and security configuration
- `favicon.svg` — browser icon
- `social-preview.svg` — Open Graph preview artwork

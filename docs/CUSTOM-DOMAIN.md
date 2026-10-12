# Your own domain with Cloudflare

RustArena is built and published by GitHub Actions to GitHub Pages. Cloudflare is the registrar
(where you buy the name) and the DNS (which points the name at GitHub Pages). It can also count
visits with Web Analytics. Until you finish these steps, the site stays at
`https://antonyperez0.github.io/RustArena/`.

The domain is `rustarena.app`. It's registered on Cloudflare, so Cloudflare already runs its DNS.

`.app` addresses only work over HTTPS (browsers require it for the whole `.app` ending). So the new
address shows a security error until GitHub has issued the site's certificate in step 4, which
usually takes 15 to 60 minutes after DNS is set. The old address keeps working until step 3.

## 1. Verify the domain with GitHub (prevents takeovers)

1. On GitHub, open your profile picture > **Settings > Pages > Add a domain**, then enter `rustarena.app`.
2. GitHub shows a TXT record. In Cloudflare, open the domain > **DNS > Records > Add record** and add it:
   Type `TXT`, Name as shown (`_github-pages-challenge-AntonyPerez0`), Content as shown.
3. Back on GitHub, click **Verify**. It can take a few minutes for DNS to update.

## 2. Point the domain at GitHub Pages

In Cloudflare, **DNS > Records**, add these records. Set **Proxy status** to **DNS only** (grey cloud) for now:

| Type    | Name  | Content                  |
| ------- | ----- | ------------------------ |
| `A`     | `@`   | `185.199.108.153`        |
| `A`     | `@`   | `185.199.109.153`        |
| `A`     | `@`   | `185.199.110.153`        |
| `A`     | `@`   | `185.199.111.153`        |
| `AAAA`  | `@`   | `2606:50c0:8000::153`    |
| `AAAA`  | `@`   | `2606:50c0:8001::153`    |
| `AAAA`  | `@`   | `2606:50c0:8002::153`    |
| `AAAA`  | `@`   | `2606:50c0:8003::153`    |
| `CNAME` | `www` | `antonyperez0.github.io` |

Delete any other `A`, `AAAA` or `CNAME` records for `@` or `www`. Parked-domain records, for example, would conflict.

## 3. Tell the site its new address

The deploy workflow already sets `CUSTOM_DOMAIN: rustarena.app` (a repository variable named
`CUSTOM_DOMAIN` overrides it). Each deploy builds the site for the root of the domain, writes the
address into the sitemap, page metadata and share links, and publishes a `CNAME` file, which tells
GitHub Pages the domain to serve. Don't edit the `CNAME` file on the `gh-pages` branch by hand: each
deploy replaces that branch.

## 4. Turn on HTTPS

1. Open **Settings > Pages** in the repository. **Custom domain** shows `rustarena.app` with a DNS check.
   Wait until it says the DNS check succeeded. GitHub then gets a free certificate, which can
   take up to an hour.
2. Tick **Enforce HTTPS**.

Now `https://rustarena.app` serves the site, `www.rustarena.app` redirects to it, and the old
`antonyperez0.github.io/RustArena/...` addresses redirect to the same pages on the new domain.

## 5. Optional: Cloudflare extras

- **Visit counts (no cookies, no consent banner):** in Cloudflare, open **Analytics & Logs > Web Analytics >
  Add a site** and enter `rustarena.app`. Choose the manual JavaScript snippet option and copy the
  32-character `token` value. Add it as a repository variable named `CF_BEACON_TOKEN`, then run the
  workflow again.
- **Cloudflare's proxy (orange cloud):** this is not needed, because GitHub Pages already serves the
  site from a global CDN. If you want Cloudflare's caching or firewall anyway, finish step 4 first.
  Then set **SSL/TLS > Overview** to **Full** (never **Flexible**, which causes an endless redirect
  loop) and switch the records to **Proxied**. While proxied, GitHub's DNS check under
  Settings > Pages may show a warning. The site keeps working, because Cloudflare serves its own
  certificate to visitors.
- **Search engines:** add `https://rustarena.app` in [Google Search Console](https://search.google.com/search-console)
  (Cloudflare can add the verification record for you) and submit `https://rustarena.app/sitemap.xml`.

## Learners who started on the old address

Browsers keep progress separately for each site address, so progress saved on
`antonyperez0.github.io` doesn't follow to `rustarena.app` by itself. Once step 3 is done, the old
address redirects, so its saved progress can no longer be opened. If you (or anyone) already have
progress worth keeping, do this before step 3: on the old address, open **Profile**, then make a
transfer link or a private GitHub Gist. After the switch, open the link (or connect the same Gist)
on the new domain.

## Going back

Set `CUSTOM_DOMAIN: ""` in `.github/workflows/deploy.yml` (and delete any `CUSTOM_DOMAIN` variable), then push. The site returns to
`https://antonyperez0.github.io/RustArena/`. Also remove the custom domain under Settings > Pages.

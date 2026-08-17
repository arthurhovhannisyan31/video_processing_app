## Getting HTTPS working end-to-end (frontend on Vercel + backend on Hetzner)

Frontend is already live on Vercel behind Cloudflare (`https://www.videoprocessing.app`). The remaining piece is the backend: it currently serves plain HTTP, so the HTTPS frontend can't call it (mixed content / browser blocks it). This repo's `server-deploy.yml` workflow puts an nginx + Let's Encrypt TLS proxy in front of the backend so it's reachable at `https://api.videoprocessing.app`. It builds/pushes the image, copies `docker-compose.prod.yml` + `configs/nginx/backend.conf` to the host, reloads nginx, writes `.env`, restarts containers, runs a smoke test — fully automatic once the host is ready. But nginx isn't installed there yet, so the first run will fail on the nginx reload step until these are done by hand:

1. **DNS** — add a record for `api.videoprocessing.app` in Cloudflare, pointing to the Hetzner host IP:
   - Type **A**, name `api`, value = Hetzner IP.
   - Set it to **DNS only** (grey cloud, not orange/proxied). If Cloudflare proxies it, certbot's HTTP validation (port 80, standalone mode) will hit Cloudflare's edge instead of the Hetzner host and fail. Can switch it to proxied afterward once the cert exists, but simplest is to leave it DNS-only permanently for an API host.
2. **Run the bootstrap script on the host** (installs nginx + certbot, opens firewall ports, issues the initial TLS cert — idempotent, safe to re-run):
   ```
   scp configs/scripts/server-bootstrap.sh root@<host>:~/
   ssh root@<host> "bash ~/server-bootstrap.sh"
   ```
3. **GH vars** (repo → Settings → Environments → production) — check:
   - `BACKEND_CORS_ORIGINS` includes `https://www.videoprocessing.app`
4. **Vercel env var** (this is on Vercel's dashboard, not in this repo) — set `NEXT_PUBLIC_BACKEND_BASE_URL=https://api.videoprocessing.app/api` on the frontend project, then redeploy it so the build picks it up.

Once these 4 steps are done, backend deploys are triggered locally with:

```
./configs/scripts/deploy-prod.sh          # full check + build + deploy
./configs/scripts/deploy-prod.sh skip     # skip build, redeploy current image (rollback-friendly)
```

It runs preflight checks (main branch, clean tree, `cargo sqlx prepare`, clippy, audit, tests), tags the commit, then triggers `server-deploy.yml` via `gh workflow run`. The workflow itself has a smoke test that fails the run if `/api/health` doesn't respond after deploy.

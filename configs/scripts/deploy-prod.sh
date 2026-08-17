#!/bin/bash
# Usage:
#   ./configs/scripts/deploy-prod.sh                    # full check + deploy
#   ./configs/scripts/deploy-prod.sh skip                # skip build, redeploy current image
#   ./configs/scripts/deploy-prod.sh --skip-preflight     # ignore local uncommitted changes
set -e

REPO_ROOT="$(git rev-parse --show-toplevel)"
. "${REPO_ROOT}/configs/bash/colors"

FORCE=false
ARGS=()
for arg in "$@"; do
    case "$arg" in
        --skip-preflight) FORCE=true ;;
        *)                ARGS+=("$arg") ;;
    esac
done

printf "🔍 Pre-flight checks...\n"
BRANCH=$(git rev-parse --abbrev-ref HEAD)
if [ "$BRANCH" != "main" ]; then
    printf "${RED}❌ Must be on main branch (currently on '%s')${NC}\n" "$BRANCH"
    exit 1
fi
if ! git diff --quiet || ! git diff --cached --quiet; then
    if [ "$FORCE" = "true" ]; then
        printf "${YELLOW}⚠️  Uncommitted changes — deploying last pushed commit (--skip-preflight)${NC}\n"
    else
        printf "${RED}❌ Uncommitted changes. Commit or stash before deploying.${NC}\n"
        exit 1
    fi
fi
printf "${GREEN}✅ On main, working tree is clean${NC}\n"

printf "🗄️  Running cargo sqlx prepare...\n"
(cd "${REPO_ROOT}/modules/backend" && cargo sqlx prepare)
if ! git diff --quiet; then
    printf "📝 .sqlx files changed — committing and pushing...\n"
    git add modules/backend/.sqlx/
    git commit -m "chore: update .sqlx query cache"
    git push origin main
fi
printf "${GREEN}✅ SQLx offline data is up to date${NC}\n"

printf "📦 Checking frontend lockfile...\n"
(cd "${REPO_ROOT}/modules/frontend" && yarn install --frozen-lockfile) || {
    printf "${RED}❌ modules/frontend/yarn.lock is stale — run 'yarn install' and commit${NC}\n"
    exit 1
}
printf "${GREEN}✅ Lockfile up to date${NC}\n"

printf "🔷 Clippy / Audit...\n"
(cd "${REPO_ROOT}/modules/backend" && cargo clippy --all-features --all-targets --quiet)
(cd "${REPO_ROOT}/modules/backend" && cargo audit)
printf "${GREEN}✅ Clippy, Audit OK${NC}\n"

printf "🧪 Running backend tests...\n"
(cd "${REPO_ROOT}/modules/backend" && cargo test)
printf "${GREEN}✅ Tests passed${NC}\n"

SKIP_BUILD="${ARGS[0]:-false}"
if [ "$SKIP_BUILD" = "skip" ]; then
    SKIP_BUILD="true"
fi

TAG_NAME="deploy_prod_$(date +'%Y/%m/%d_%Hh%Mm%Ss')"
git tag "$TAG_NAME"
git push origin "$TAG_NAME"

printf "🚀 Triggering deployment...\n"
gh workflow run server-deploy.yml -f skip_build="$SKIP_BUILD"

printf "${GREEN}✅ Deploy workflow triggered!${NC}\n"
echo "   Tag:        $TAG_NAME"
echo "   skip_build: $SKIP_BUILD"
echo "   Track at:   https://github.com/$(gh repo view --json nameWithOwner -q .nameWithOwner)/actions"

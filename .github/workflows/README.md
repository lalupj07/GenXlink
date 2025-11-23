# GitHub Actions Workflows

## Setup Required Secrets

To enable automatic deployment, configure these secrets in your GitHub repository:

### Railway Deployment
- `RAILWAY_TOKEN` - Your Railway API token
  - Get from: https://railway.app/account/tokens

### Fly.io Deployment  
- `FLY_API_TOKEN` - Your Fly.io API token
  - Get from: `flyctl auth token`

## How to Add Secrets

1. Go to your GitHub repository
2. Navigate to Settings → Secrets and variables → Actions
3. Click "New repository secret"
4. Add the secret name and value
5. Click "Add secret"

## Workflow Files

- `deploy.yml` - Main deployment workflow
  - Builds and tests the project
  - Deploys to Railway (if token configured)
  - Deploys to Fly.io (if token configured)

## Note on IDE Warnings

The IDE may show warnings about `secrets.RAILWAY_TOKEN` and `secrets.FLY_API_TOKEN` being invalid.
These are false positives - `secrets` is a valid GitHub Actions context that's available at runtime.

The workflow handles missing secrets gracefully by checking if they're configured before attempting deployment.

## Active Workflows

### 1. Validate Project
- **File:** `validate.yml`
- **Trigger:** Push to main branch
- **Purpose:** Validates project structure and documentation
- **Status:** ✅ Active

### 2. Pages Build and Deployment
- **Managed by:** GitHub (automatic)
- **Purpose:** Deploys documentation to GitHub Pages
- **URL:** https://lalupj07.github.io/GenXlink/
- **Status:** ✅ Active

## Deployment

Deployment is done **manually** using Railway CLI:

```bash
railway login
railway init
railway up
```

This approach gives you more control and avoids CI/CD complexity for now.

## Future Enhancements

When ready, you can add:
- Automated Rust builds
- Automated testing
- Automated deployment to Railway/Fly.io
- Release automation

For now, manual deployment via Railway CLI is simpler and more reliable.

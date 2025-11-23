# GitHub Actions Workflows

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

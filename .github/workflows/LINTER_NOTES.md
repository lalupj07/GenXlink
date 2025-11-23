# GitHub Actions Linter Notes

## About IDE Warnings

### "Context access might be invalid" Warnings

You may see warnings like:
- `Context access might be invalid: RAILWAY_TOKEN`
- `Context access might be invalid: FLY_API_TOKEN`

**These are FALSE POSITIVES.**

### Why These Warnings Appear

1. **Static Analysis**: IDE uses YAML linter without GitHub Actions context knowledge
2. **Limited Understanding**: Linter doesn't recognize GitHub's runtime environment
3. **False Positives**: The code is correct, but IDE can't verify it

### Reality vs IDE

| Aspect | IDE Thinks | Reality |
|--------|------------|---------|
| `secrets` context | Unknown | ✅ Valid GitHub Actions context |
| `secrets.RAILWAY_TOKEN` | Invalid | ✅ Works when secret is added |
| `secrets.FLY_API_TOKEN` | Invalid | ✅ Works when secret is added |
| Will it work? | Unknown | ✅ Yes, perfectly |

### Valid GitHub Actions Contexts

These are all valid and will work at runtime:
- `secrets.*` - Repository secrets
- `github.*` - Workflow information
- `env.*` - Environment variables
- `job.*` - Job information
- `steps.*` - Step information
- `runner.*` - Runner information

### Official Documentation

- [Using Secrets in GitHub Actions](https://docs.github.com/en/actions/security-guides/using-secrets-in-github-actions)
- [Contexts and Expression Syntax](https://docs.github.com/en/actions/learn-github-actions/contexts)

### What To Do

**IGNORE these warnings.** They are:
- Not actual errors
- IDE limitations
- Safe to ignore
- Will work correctly on GitHub

### Our Workflow Handles Missing Secrets

Our code checks if secrets exist before using them:

```yaml
# Example from deploy.yml
if [ -z "${{ secrets.RAILWAY_TOKEN }}" ]; then
  echo "Token not configured, skipping deployment"
  echo "skip=true" >> $GITHUB_OUTPUT
else
  echo "skip=false" >> $GITHUB_OUTPUT
fi
```

This ensures graceful handling when secrets aren't configured.

---

**Summary: These warnings are IDE limitations, not code problems. The workflow is correct and will work perfectly.**

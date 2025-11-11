# 🏷️ tag_checker

**Check for the most recent git tag** based on a prefix and optional prerelease mode.  

---

## 📖 Description

The `tag_checker` GitHub Action retrieves the most recent semver-formatted git tag in a repository. It supports filtering by a specific tag prefix (e.g. `v` or `release-`) and optionally includes prerelease tags (e.g. `v1.0.0-prerelease.1`). This is useful for automated versioning, release workflows, or CI pipelines that depend on the latest tag.

The tags searched are filtered based on the provided `release-branch`. If the checked-out branch matches the `release-branch`, the Action will only search for non-prelease tags. Otherwise, it will include prerelease tags based on the provided `prerelease-suffix`.

---

## ⚙️ Inputs

| Name | Description | Required | Default |
|------|--------------|-----------|----------|
| `release-branch` | The branch to check for the latest tag. | ✅ Yes | — |
| `tag-prefix` | The prefix of the semver tag to check for (e.g. `v` or `release-`). | ❌ No | `''` |
| `prerelease-suffix` | The suffix used to identify prerelease tags (e.g. `beta`, `rc`, `alpha`). | ❌ No | `'prerelease'` |
| `token` | The GitHub token to use for downloading the action binary, defaults to workflow token. | ❌ No | `${{ github.token }}` |
| `working-directory` | The working directory to run the tag check in. | ❌ No | `'.'` |

---

## 🧾 Outputs

| Name | Description |
|------|--------------|
| `latest_tag` | The latest tag found matching the given filters. |

## Example usage

###
```yaml
steps:
  - name: Get latest tag
    id: tag
    uses: waikato-ahuora-smart-energy-systems/ci-actions@v0.1.6
    with:
      release-branch: main
      tag-prefix: v

  - name: Print latest tag
    run: echo "Latest tag is ${{ steps.tag.outputs.latest_tag }}"

  - name: Get latest prerelease tag
    id: tag-prerelease
    uses: waikato-ahuora-smart-energy-systems/ci-actions@v0.1.6
    with:
      release-branch: main
      tag-prefix: v
      prerelease-suffix: beta
```

## 🧰 Notes

The Action must be run in a checked-out repository (make sure to use actions/checkout@v4 before running it).

- tag_prefix allows filtering by patterns like v1., release-, etc.
- Works seamlessly in monorepos using working-directory for subdirectory-based tagging.

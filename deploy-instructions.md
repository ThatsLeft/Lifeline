# GitHub Pages Deployment Instructions

## Step 1: Initialize Git Repository (if not already done)

```bash
cd /home/thatsright/Work/Forte/lifeline
git init
git add .
git commit -m "Initial commit: Lifeline Timeline application"
```

## Step 2: Create GitHub Repository

1. Go to https://github.com/new
2. Repository name: `lifeline` (or any name you prefer)
3. Description: "Interactive timeline with cosmic animations"
4. **Important**: Do NOT initialize with README, .gitignore, or license (we already have these)
5. Click "Create repository"

## Step 3: Push to GitHub

Replace `<your-username>` with your GitHub username:

```bash
git remote add origin https://github.com/<your-username>/lifeline.git
git branch -M main
git push -u origin main
```

## Step 4: Enable GitHub Pages

1. Go to your repository on GitHub
2. Click "Settings" tab
3. Click "Pages" in the left sidebar
4. Under "Build and deployment":
   - Source: Select **"GitHub Actions"**
5. Click "Save"

## Step 5: Trigger Deployment

The deployment will start automatically on your next push, or you can:

1. Go to "Actions" tab in your repository
2. Click "Deploy to GitHub Pages" workflow
3. Click "Run workflow" → "Run workflow"

## Step 6: Access Your App

After the workflow completes (usually 2-3 minutes):

Your app will be live at: `https://<your-username>.github.io/lifeline`

## Troubleshooting

### If the page shows 404:
1. Wait a few minutes for GitHub to provision the site
2. Check that GitHub Pages is set to "GitHub Actions" source
3. Verify the workflow completed successfully in the Actions tab

### If the workflow fails:
1. Check the Actions tab for error messages
2. Ensure you have Actions enabled for your repository
3. Verify all files were committed and pushed

### If assets don't load:
- The workflow automatically sets the correct `public-url` based on your repository name
- Clear browser cache and try again

## Making Updates

Every time you push to the `main` branch, GitHub Actions will automatically:
1. Build the WASM version
2. Deploy to GitHub Pages
3. Update your live site

```bash
# Make your changes
git add .
git commit -m "Update: describe your changes"
git push
```

## Local Development

While developing, test locally with:

```bash
# Web version with hot reload
trunk serve
# Then open http://127.0.0.1:8080

# Or native version
cargo run
```

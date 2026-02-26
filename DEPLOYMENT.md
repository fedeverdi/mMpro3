# Deployment Guide

## Vercel Deployment

MMpro3 can be deployed to Vercel as a web application (without Electron features).

### Prerequisites

1. A [Vercel account](https://vercel.com)
2. [Vercel CLI](https://vercel.com/cli) installed (optional, for local testing)

### Quick Deploy

#### Option 1: GitHub Integration (Recommended)

1. Push your code to GitHub
2. Go to [Vercel Dashboard](https://vercel.com/dashboard)
3. Click "Add New Project"
4. Import your GitHub repository
5. Vercel will automatically detect the configuration from `vercel.json`
6. Click "Deploy"

#### Option 2: Vercel CLI

```bash
# Install Vercel CLI (if not already installed)
npm i -g vercel

# Login to Vercel
vercel login

# Deploy
vercel
```

### Build Commands

The project includes several build commands for different versions:

```bash
# Build full version (default)
npm run build:web
npm run build:web:full

# Build medium version
npm run build:web:medium

# Build demo version
npm run build:web:demo
```

### Environment Variables

Set these in your Vercel project settings:

- `VITE_BUILD_MODE`: Set to `demo`, `medium`, or `full` (default: `full`)

### Build Configuration

The web build uses `vite.web.config.mjs` which:
- Outputs to `dist/` directory
- Uses `index.html` as entry point
- Includes all assets from `public/` folder
- Configured for SPA routing

### Local Preview

To test the production build locally:

```bash
# Build
npm run build:web

# Preview
npm run preview:web
```

### Differences from Electron Version

The web version deployed to Vercel:
- ✅ All audio features work
- ✅ Tone.js audio engine
- ✅ All UI features
- ✅ Browser audio API for microphone input
- ❌ No file system access (uses browser storage)
- ❌ No native file dialogs (uses browser file picker)
- ❌ No Electron-specific features

### Custom Domain

To add a custom domain:

1. Go to your Vercel project settings
2. Navigate to "Domains"
3. Add your custom domain
4. Follow DNS configuration instructions

### Deployment Modes

Configure build mode in Vercel:

1. Go to Project Settings → Environment Variables
2. Add `VITE_BUILD_MODE` with value:
   - `demo` - Limited features
   - `medium` - Intermediate features
   - `full` - All features (default)

### Troubleshooting

**Build fails:**
- Check that all dependencies are in `package.json`
- Verify Node.js version (18.x or higher recommended)
- Check build logs in Vercel dashboard

**Audio doesn't work:**
- Ensure HTTPS is enabled (Vercel provides this by default)
- Check browser console for errors
- Verify browser supports Web Audio API

**File storage issues:**
- Web version uses IndexedDB for storage
- Consider storage limits (~50MB typical browser limit)
- Large audio files may need external storage solution

### Performance Optimization

The vercel.json includes:
- Asset caching (1 year for immutable assets)
- SPA routing configuration
- Optimized headers

### Analytics

To enable Vercel Analytics:

1. Go to your project in Vercel
2. Navigate to "Analytics" tab
3. Enable Web Analytics
4. No code changes needed

## Other Platforms

### Netlify

Similar to Vercel, use:
```bash
npm run build:web
```

Configure in `netlify.toml`:
```toml
[build]
  command = "npm run build:web"
  publish = "dist"

[[redirects]]
  from = "/*"
  to = "/index.html"
  status = 200
```

### GitHub Pages

```bash
# Build with base path
vite build --config vite.web.config.mjs --base=/your-repo-name/

# Deploy to gh-pages branch
# (use gh-pages package or GitHub Actions)
```

## Support

For deployment issues:
- Check [Vercel Documentation](https://vercel.com/docs)
- Review build logs in Vercel dashboard
- Check browser console for runtime errors

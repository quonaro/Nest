# Nest Documentation

This directory contains the Vue.js documentation site for Nest.

## Development

```bash
# Install dependencies
npm install

# Start development server
npm run dev

# Build for production
npm run build

# Preview production build
npm run preview
```

## Deployment

The documentation is automatically built and deployed to the `docs-build` branch when changes are pushed to `docs-landing` branch.

The GitHub Actions workflow will:
1. Build the Vue.js application
2. Deploy the built files to the `docs-build` branch
3. The `docs-build` branch can be used for GitHub Pages or any static hosting

## Structure

- `src/views/` - Documentation pages
- `src/components/` - Reusable components
- `src/router/` - Vue Router configuration
- `src/assets/` - Styles and assets


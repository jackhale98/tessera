# Quick Start Guide - Tessera Frontend

## Initial Setup

```bash
# Install dependencies
npm install

# Run tests to verify everything works
npm run test

# Start development server
npm run dev
```

## Expected Output

### Tests
```
✓ 31 tests passing
✓ 5 test files
```

### Dev Server
- Opens at: http://localhost:1420
- Hot reload enabled
- Should show polished UI with:
  - Dark sidebar
  - Clean topbar
  - Dashboard with metrics
  - Smooth animations

## Quick Commands

```bash
# Development
npm run dev              # Web dev server
npm run tauri dev        # Tauri desktop app

# Testing
npm run test             # Watch mode
npm run test:ui          # Visual test UI
npm run test:coverage    # Coverage report

# Building
npm run build            # Production build
npm run tauri build      # Build desktop app
```

## First Time Using?

1. Start dev server: `npm run dev`
2. Open http://localhost:1420
3. Click "Project Management" in sidebar
4. Click "New Entity" to see form modal
5. Explore other modules

## File Structure

```
src/
├── components/
│   ├── layout/         # Sidebar, TopBar, Layout
│   ├── ui/             # Reusable UI components
│   ├── forms/          # Entity forms
│   └── views/          # Page views
├── lib/api.ts          # Tauri API client
├── stores/             # Zustand state
├── types/              # TypeScript types
└── index.css           # Tailwind + custom styles
```

## Key Features

- ✅ Modern, clean UI
- ✅ Dark themed sidebar
- ✅ Type-safe API integration
- ✅ Comprehensive tests
- ✅ Form validation
- ✅ State management
- ✅ Responsive design

## Need Help?

- Check `FIXES_APPLIED.md` for troubleshooting
- Check `FRONTEND_README.md` for detailed docs
- Check `FRONTEND_IMPLEMENTATION_SUMMARY.md` for architecture

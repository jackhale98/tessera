# Tessera Frontend Implementation

## Overview

The Tessera front-end has been systematically built using React, TypeScript, Tailwind CSS, and Tauri, following the design specifications from the GUI example while adapting it for the Tessera product lifecycle management system.

## Project Structure

```
src/
├── components/
│   ├── layout/
│   │   ├── Layout.tsx       # Main layout wrapper
│   │   ├── Sidebar.tsx      # Left sidebar with module navigation
│   │   └── TopBar.tsx       # Top bar with search and actions
│   ├── ui/
│   │   ├── Badge.tsx        # Status and type badges
│   │   ├── Card.tsx         # Reusable card component
│   │   └── Table.tsx        # Table components (Table, TableHeader, TableBody, etc.)
│   └── views/
│       ├── Dashboard.tsx         # Main dashboard with metrics
│       ├── EntityTableView.tsx   # Generic entity table view
│       └── ModuleView.tsx        # Module view wrapper
├── lib/
│   └── api.ts               # Tauri API client with type-safe wrappers
├── stores/
│   ├── useEntityStore.ts    # Zustand store for entity management
│   └── useUIStore.ts        # Zustand store for UI state
├── types/
│   └── index.ts             # TypeScript types matching Rust backend
├── App.tsx                  # Main application component
├── main.tsx                 # Application entry point
└── index.css                # Global styles with Tailwind

Configuration:
├── tailwind.config.js       # Tailwind CSS configuration
├── vite.config.ts           # Vite configuration with path aliases
└── tsconfig.json            # TypeScript configuration
```

## Key Features Implemented

### 1. Type-Safe Backend Integration

All Rust backend entity types have been mirrored in TypeScript:
- Task, Milestone, Resource, Calendar, Baseline
- Requirement, Hazard, Risk, RiskControl
- Assembly, Component, Feature, Mate, Stackup, Supplier, Quote
- Verification, Validation, Manufacturing

The API client (`src/lib/api.ts`) provides type-safe wrappers around all Tauri commands for CRUD operations and calculations.

### 2. State Management

**Entity Store** (`useEntityStore`):
- Manages state for tasks, requirements, risks, components, and assemblies
- Provides async actions for create, read, update, delete operations
- Handles loading states and errors
- Integrates directly with the Tauri backend

**UI Store** (`useUIStore`):
- Manages navigation state (active module, view mode)
- Handles search queries and filters
- Controls sidebar collapse state

### 3. Layout Components

**Sidebar**:
- Dark theme matching the GUI example
- Module navigation (Dashboard, Project Management, Requirements, Risk, Design, V&V, Manufacturing)
- Git branch indicator
- Settings access

**TopBar**:
- Context-aware title
- Global search
- Notification bell (with indicator)
- Favorites
- "New Entity" button (shown in module views)

**Layout**:
- Responsive flex layout
- Overflow handling for content areas

### 4. Reusable UI Components

**Badge**:
- Status badges (Draft, PendingApproval, Approved, Released)
- Type badges (User, System, Design, Software, Safety)
- Customizable color schemes

**Card**:
- Consistent card styling
- Optional title and icon
- Used for dashboard metrics, warnings, activity

**Table Components**:
- Table, TableHeader, TableBody, TableRow, TableHead, TableCell
- Hover effects
- Responsive design
- Clickable rows

### 5. Views

**Dashboard**:
- Metric cards (Project completion, estimated end date, open risks, requirements count)
- Warnings & issues panel
- Recent activity panel
- Welcome message for new users

**EntityTableView**:
- Generic table view for tasks, requirements, and risks
- Customized columns based on entity type
- Progress bars for tasks
- Risk score visualization with color coding
- Status badges

**ModuleView**:
- Wraps entity tables with view controls
- Filter and export buttons (ready for implementation)
- Supports table view mode (graph, matrix, chart modes can be added)

### 6. Styling

**Tailwind Configuration**:
- Custom color palette using slate shades
- Custom font families (Inter for sans, Fira Code for mono)
- Extended theme with Tessera-specific styles

**Global Styles** (`index.css`):
- Base styles for common elements
- Component classes (btn, card, input, table-container)
- Utility classes (scrollbar-thin for custom scrollbars)
- Responsive and accessible styling

## Running the Application

```bash
# Install dependencies
npm install

# Run in development mode
npm run dev

# Build for production
npm run build

# Run Tauri in development
npm run tauri dev

# Build Tauri application
npm run tauri build
```

## Next Steps for Development

### 1. Form Components
Create entity creation/editing forms using:
- React Hook Form for form state management
- Zod for validation (matching Rust backend schemas)
- Dynamic form generation based on entity type

### 2. Enhanced Views
- **Graph View**: Entity relationship visualization using reactflow or vis-network
- **Matrix View**: Traceability matrix for requirements and design
- **Chart View**: Gantt charts for project management, cost analysis charts for BOM

### 3. Risk Management Enhancements
- Risk matrix heatmap visualization
- Interactive risk scoring
- Control assignment interface
- FMEA export functionality

### 4. Design Module
- BOM table with cost breakdowns
- Tolerance analysis results visualization (histograms, distribution plots)
- Stackup editor with feature contribution
- Assembly tree view

### 5. Filtering and Search
- Advanced filtering UI
- Full-text search integration
- Filter persistence
- Saved filter presets

### 6. Real-time Updates
- WebSocket/polling for multi-user scenarios
- Git change notifications
- Comment notifications

### 7. Testing
Create tests using Vitest and React Testing Library:
```bash
# Run tests
npm run test

# Run tests with coverage
npm run test:coverage
```

Test coverage should include:
- Component rendering
- User interactions
- API integration
- State management
- Error handling

### 8. Accessibility
- Keyboard navigation
- ARIA labels
- Focus management
- Screen reader support

### 9. Performance Optimization
- Virtual scrolling for large entity lists
- Lazy loading for module views
- Memoization of expensive calculations
- Code splitting

## Design Principles

1. **Modularity**: Components are small, focused, and reusable
2. **Type Safety**: Full TypeScript coverage with strict mode
3. **Maintainability**: Clear folder structure and naming conventions
4. **Scalability**: Easy to add new entity types and views
5. **Consistency**: Follows the GUI example's design patterns
6. **Accessibility**: Semantic HTML and ARIA attributes
7. **Performance**: Efficient rendering and state management

## API Integration Notes

All backend commands are accessed through the `api` object:

```typescript
import { api } from '@/lib/api';

// Create a task
const task = await api.task.create({
  name: 'New Task',
  description: 'Task description',
  // ...
});

// List requirements
const requirements = await api.requirement.list();

// Run critical path calculation
const result = await api.calculation.criticalPath();
```

## Customization

### Adding a New Module

1. Add the module to `Sidebar.tsx` modules array
2. Create a view component in `src/components/views/`
3. Add routing logic in `App.tsx`
4. Implement entity-specific table columns in `EntityTableView.tsx`
5. Add API methods if needed in `src/lib/api.ts`

### Styling

The app uses Tailwind CSS with custom configuration. To modify:
- Edit `tailwind.config.js` for theme changes
- Use `src/index.css` for global component classes
- Apply Tailwind classes directly in components

### State Management

To add new entity types to the store:
1. Add TypeScript type in `src/types/index.ts`
2. Add state and actions in `useEntityStore.ts`
3. Add API methods in `src/lib/api.ts`

## Troubleshooting

### Build Issues
- Ensure all dependencies are installed: `npm install`
- Clear cache: `rm -rf node_modules package-lock.json && npm install`
- Check TypeScript errors: `npx tsc --noEmit`

### Tauri Issues
- Ensure Rust toolchain is installed
- Check Tauri prerequisites: `npm run tauri info`
- Rebuild Tauri: `npm run tauri build -- --debug`

### Type Errors
- Ensure types in `src/types/index.ts` match Rust backend exactly
- Check that all required fields are provided in API calls
- Use TypeScript strict mode to catch errors early

## Contributing

When adding new features:
1. Follow the existing folder structure
2. Use TypeScript with strict mode
3. Create reusable components when possible
4. Add types for all props and function parameters
5. Test thoroughly with the Rust backend
6. Update this README with significant changes

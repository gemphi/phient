# puijs (`puijs`)

> _Enterprise React 19 Component Library & Data Engine modeled after **Palantir Blueprint** (`@blueprintjs/core`, `@blueprintjs/table`)._

[![npm version](https://img.shields.io/badge/npm-@phiace/puijs-red.svg)](https://www.npmjs.com/package/@phiace/puijs)
[![React](https://img.shields.io/badge/React-18%20%26%2019-blue.svg)](package.json)
[![Architecture](https://img.shields.io/badge/Architecture-Palantir%20Blueprint-indigo.svg)](ARCHITECTURE.md)
[![License](https://img.shields.io/badge/License-MIT-purple.svg)](LICENSE)

---

## 1. Quick Start

### Installation
```bash
npm install @phiace/puijs lucide-react
```

### Import Styles & Wrap Root Provider
```tsx
import '@phiace/puijs/styles';
import '@phiace/puijs/tokens';
import { PuiProvider } from '@phiace/puijs';

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <PuiProvider defaultTheme="system" defaultBrand="foundry">
      {children}
    </PuiProvider>
  );
}
```

---

## 2. Core Modules

| Module | Exports & Capabilities |
|:---|:---|
| **Presentation Primitives** | `Button`, `Tag`, `Badge`, `Callout`, `Card`, `Table`, `Tree`, `Dialog`, `Drawer`, `Toast` |
| **Layout Primitives** | `Page`, `Section`, `Stack`, `Grid`, `Container`, `Row`, `Col` |
| **Data Display** | `Table`, `Tree`, `Card`, `CodeBlock`, `ThreeDMode` |
| **Commerce Suite** | `Price`, `ProductCard`, `ProductGrid`, `CartDrawer`, `CartItem`, `CartSummary`, `OrderCard`, `FacetFilter` |
| **Reactive Store Layer** | `createStore`, `useStore` (`useSyncExternalStore`), `cartStore`, `wishlistStore`, `persist`, `logger` |
| **Service Layer & DI** | `ServiceContainer`, `BaseService`, `EventEmitter`, `useService`, `TelemetryService`, `CartService` |
| **API & Streaming Engine**| `ApiClient`, `QueryCache`, `StreamClient`, `useQuery`, `useMutation`, `useStream`, `useAPI` |

---

## 3. Usage Examples

### Pure UI & Commerce
```tsx
import { ProductCard, Price, useCart } from '@phiace/puijs';

export const ProductListing = ({ product }: { product: any }) => {
  const cart = useCart();
  return (
    <ProductCard
      id={product.id}
      title={product.title}
      price={product.price}
      image={product.image}
      onAddToCart={() => cart.addItem(product)}
    />
  );
};
```

### Layout & Flex Sharing
```tsx
import { Section, Stack } from '@phiace/puijs';

<Section variant="page">
  <Stack fill>
    <Stack>Auto-sized child</Stack>
    <Stack fill>Remaining space</Stack>
  </Stack>
</Section>
```

```tsx
import { createStore, useStore } from '@phiace/puijs';

const counterStore = createStore((set, get) => ({
  count: 0,
  inc: () => set({ count: get().count + 1 }),
}));

export const Counter = () => {
  const { count, inc } = useStore(counterStore);
  return <button onClick={inc}>Count: {count}</button>;
};
```

---

## 4. Documentation & Applications
- **[`apps/pui-book`](apps/pui-book/)** (`puijs.com`): Interactive docs, live code playground, and component specs.
- **[`apps/demo-app`](apps/demo-app/)**: Enterprise operational cockpit with live ontology tables and telemetry.
- **[`apps/landing-app`](apps/landing-app/)**: Design system feature tour and theme switcher.

---

## License
MIT © [GemPhi](https://github.com/gemphi)

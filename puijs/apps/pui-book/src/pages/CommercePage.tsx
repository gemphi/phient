'use client';

import React from 'react';
import { Stack, Title, Text, Divider, Callout } from '@pui/components';
import { PageShell } from '../layouts/PageShell';
import { CommerceExample } from '../examples/CommerceExample';
import { ShoppingCart } from 'lucide-react';

export const CommercePage: React.FC = () => {
  return (
    <PageShell
      title="Commerce Suite"
      description="Industrial, high-performance commerce primitives and reactive storefront flows for products, carts, pricing, and checkout."
    >
      <Stack direction="column" gap={4}>
        <Callout intent="primary" title="Pure Decoupled Commerce Primitives" icon={<ShoppingCart size={18} />}>
          Components are 100% decoupled from backend schemas. All state (cart, wishlist, catalog, currency) is orchestrated reactively through PUI micro-stores and services.
        </Callout>

        <Stack direction="column" gap={2}>
          <Title level={4}>Interactive Storefront & Cart Drawer</Title>
          <Text variant="muted">
            Add items, toggle wishlist, filter categories, and inspect live cart synchronization.
          </Text>
        </Stack>

        <CommerceExample />

        <Divider />
      </Stack>
    </PageShell>
  );
};

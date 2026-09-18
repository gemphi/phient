'use client';

import React, { useState } from 'react';
import {
  ProductGrid,
  CartDrawer,
  Price,
  Button,
  Stack,
  Card,
  CardBody,
  Callout,
  useCart,
  useWishlist,
  MOCK_PRODUCTS,
} from '@pui/components';
import { ShoppingBag, Sparkles, CheckCircle2 } from 'lucide-react';

export const CommerceExample: React.FC = () => {
  const cart = useCart();
  const wishlist = useWishlist();
  const [selectedCategory, setSelectedCategory] = useState<string>('All');
  const [checkoutSuccess, setCheckoutSuccess] = useState<string | null>(null);

  const filtered =
    selectedCategory === 'All'
      ? MOCK_PRODUCTS
      : MOCK_PRODUCTS.filter((p) => p.category === selectedCategory);

  const productCardProps = filtered.map((p) => ({
    id: p.id,
    title: p.title,
    category: p.category,
    price: p.price,
    salePrice: p.salePrice,
    image: p.image,
    inWishlist: wishlist.hasItem(p.id),
    onAddToCart: () => cart.addItem({ id: p.id, title: p.title, price: p.salePrice || p.price, image: p.image }),
    onToggleWishlist: () => wishlist.toggleItem({ id: p.id, title: p.title, price: p.salePrice || p.price, image: p.image }),
  }));

  const handleCheckout = () => {
    const orderId = `ORD-${Date.now().toString(36).toUpperCase()}`;
    cart.clearCart();
    cart.setOpen(false);
    setCheckoutSuccess(orderId);
  };

  return (
    <Stack direction="column" gap={4}>
      <Card>
        <CardBody>
        <Stack direction="row" justify="space-between" align="center">
          <Stack direction="row" gap={2}>
            {['All', 'Hardware', 'Systems', 'Sensors'].map((cat) => (
              <Button
                key={cat}
                variant={selectedCategory === cat ? 'primary' : 'secondary'}
                size="sm"
                onClick={() => setSelectedCategory(cat)}
              >
                {cat}
              </Button>
            ))}
          </Stack>

          <Button
            variant="primary"
            size="md"
            icon={<ShoppingBag size={16} />}
            onClick={() => cart.setOpen(true)}
          >
            Cart ({cart.getTotalCount()}) - <Price amount={cart.getTotalPrice()} />
          </Button>
        </Stack>
        </CardBody>
      </Card>

      {checkoutSuccess && (
        <Callout intent="success" title="Checkout Successful" icon={<CheckCircle2 size={18} />}>
          Order <strong>#{checkoutSuccess}</strong> was dispatched and charged. Reactive store state synchronized.
        </Callout>
      )}

      <ProductGrid products={productCardProps} />

      <CartDrawer
        isOpen={cart.isOpen}
        onClose={() => cart.setOpen(false)}
        items={cart.items.map((i) => ({
          ...i,
          onUpdateQuantity: (id, q) => cart.updateQuantity(id, q),
          onRemove: (id) => cart.removeItem(id),
        }))}
        onCheckout={handleCheckout}
      />
    </Stack>
  );
};

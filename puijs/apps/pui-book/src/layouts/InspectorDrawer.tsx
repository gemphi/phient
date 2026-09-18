'use client';

import React from 'react';
import { Drawer, FormGroup, Select, Switch, Stack, Title, Text, Tag, usePuiTheme } from '@pui/components';

interface InspectorDrawerProps {
  isOpen: boolean;
  onClose: () => void;
}

export const InspectorDrawer: React.FC<InspectorDrawerProps> = ({ isOpen, onClose }) => {
  const { theme, setTheme, themeStyle, setThemeStyle, brandId, setBrandId, brands, isDark } = usePuiTheme();

  return (
    <Drawer isOpen={isOpen} onClose={onClose} title="PUI Global Inspector" position="right">
      <Stack direction="column" gap={5} padding={20}>
        <Stack direction="column" gap={1}>
          <Title level={5}>Live Design System Controls</Title>
          <Text variant="sm" color="secondary">
            Mutates root CSS custom properties in real-time.
          </Text>
        </Stack>

        <FormGroup label="Active Brand Palette">
          <Select value={brandId} onChange={(e) => setBrandId(e.target.value)}>
            {brands.map((b) => (
              <option key={b.id} value={b.id}>{b.name}</option>
            ))}
          </Select>
        </FormGroup>

        <FormGroup label="Surface Style Mode">
          <Select value={themeStyle} onChange={(e) => setThemeStyle(e.target.value as any)}>
            <option value="flat">Flat (Default)</option>
            <option value="elevated">Elevated Shadows</option>
            <option value="glass">Glassmorphism</option>
            <option value="gradient">Subtle Gradient</option>
          </Select>
        </FormGroup>

        <FormGroup label="Dark Mode Theme">
          <Switch
            label="Enable Foundry Dark Mode"
            checked={isDark}
            onChange={(e) => setTheme(e.target.checked ? 'dark' : 'light')}
          />
        </FormGroup>

        <Stack direction="row" gap={2} wrap="wrap">
          <Tag intent="primary">puijs.com</Tag>
          <Tag intent="success">Decoupled Architecture</Tag>
        </Stack>
      </Stack>
    </Drawer>
  );
};
